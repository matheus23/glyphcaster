use std::str::FromStr;

use crate::app_state::AppState;
use crate::sync::TextSynchronizer;
use anyhow::Context as _;
use automerge::transaction::Transactable;
use automerge::{Automerge, AutomergeError, ObjType, ROOT, ReadDoc};
use gtk::glib;
use iroh::Watcher;
use iroh_automerge_repo::IrohRepo;
use samod::{DocHandle, PeerId};
use sourceview5::prelude::*;

pub struct DocumentLoader {
    app_state: AppState,
}

impl DocumentLoader {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    pub async fn load_document(
        &mut self,
    ) -> Result<(sourceview5::Buffer, DocHandle, iroh::protocol::Router), Box<dyn std::error::Error>>
    {
        let rt = &self.app_state.rt;
        let iroh_secret = self.app_state.iroh_secret.clone();

        self.update_progress("Initializing iroh", 0.1).await;

        let endpoint = rt
            .spawn(async {
                let secret_key =
                    iroh_secret.and_then(|key_hex| match iroh::SecretKey::from_str(&key_hex) {
                        Ok(key) => Some(key),
                        Err(_) => {
                            tracing::warn!("invalid IROH_SECRET provided: not valid hex");
                            None
                        }
                    });

                let secret_key = match secret_key {
                    Some(key) => {
                        tracing::info!("Using existing key: {}", key.public());
                        key
                    }
                    None => {
                        tracing::info!("Generating new key");
                        let mut rng = rand::rngs::OsRng;
                        let secret_key = iroh::SecretKey::generate(&mut rng);
                        tracing::info!(
                            "Set env var for persistent Node ID: export IROH_SECRET={}",
                            data_encoding::HEXLOWER.encode(&secret_key.to_bytes())
                        );
                        secret_key
                    }
                };

                let endpoint = iroh::Endpoint::builder()
                    .discovery_n0()
                    .secret_key(secret_key)
                    .bind()
                    .await?;

                endpoint.home_relay().initialized().await;

                anyhow::Ok(endpoint)
            })
            .await??;

        self.update_progress("Initializing samod", 0.2).await;

        let samod = rt
            .spawn({
                let endpoint = endpoint.clone();
                async move {
                    samod::Samod::build_tokio()
                        .with_peer_id(PeerId::from_string(endpoint.node_id().to_string()))
                        .with_storage(samod::storage::TokioFilesystemStorage::new("./data"))
                        .load()
                        .await
                }
            })
            .await?;

        self.update_progress("Starting to serve over iroh", 0.3)
            .await;

        let (proto, router) = rt
            .spawn_blocking({
                let endpoint = endpoint.clone();
                let samod = samod.clone();
                || {
                    let proto = IrohRepo::new(endpoint.clone(), samod);
                    let router = iroh::protocol::Router::builder(endpoint)
                        .accept(IrohRepo::SYNC_ALPN, proto.clone())
                        .spawn();
                    (proto, router)
                }
            })
            .await?;

        if let Some(node_id) = self.app_state.node_id.clone() {
            self.update_progress("Connecting to remote node", 0.4).await;

            tracing::info!(%node_id, "Starting continuous sync");
            rt.spawn(async move { proto.sync_with(node_id).await });

            samod
                .when_connected(PeerId::from_string(node_id.to_string()))
                .await?;

            tracing::info!(%node_id, "Connected");
        }

        self.update_progress("Loading document...", 0.5).await;

        let handle = if let Some(doc_id) = self.app_state.document_id.clone() {
            samod
                .find(doc_id.clone())
                .await?
                .context(format!("couldn't find document with document ID {doc_id}"))?
        } else {
            let mut doc = Automerge::new();
            doc.transact::<_, _, AutomergeError>(|tx| {
                let text_id = tx.put_object(ROOT, "content", ObjType::Text)?;
                tx.splice_text(&text_id, 0, 0, "# Untitled")?;
                Ok(())
            })
            .unwrap();
            samod.create(doc).await?
        };

        let content = handle.with_document(|doc| {
            let (value, id) = doc.get(automerge::ROOT, "content").unwrap().unwrap();
            match value {
                automerge::Value::Object(automerge::ObjType::Text) => {
                    let text = doc.text(id).unwrap();
                    text.to_string()
                }
                _ => panic!("content should be a text object"),
            }
        });

        println!(
            "Connect using automerge:{} {}",
            handle.document_id(),
            endpoint.node_id()
        );

        self.update_progress("Setting up editor...", 0.9).await;

        let buffer = self.create_markdown_buffer(content).await?;

        // Final step
        self.update_progress("Ready!", 1.0).await;
        glib::timeout_future(std::time::Duration::from_millis(200)).await;

        Ok((buffer, handle, router))
    }

    async fn update_progress(&self, message: &str, progress: f64) {
        self.app_state
            .update_loading_status(message, Some(progress));

        // Small delay to make the UI updates visible
        glib::timeout_future(std::time::Duration::from_millis(50)).await;
    }

    async fn create_markdown_buffer(
        &self,
        initial_content: String,
    ) -> Result<sourceview5::Buffer, Box<dyn std::error::Error>> {
        let buffer = sourceview5::Buffer::new(None);
        buffer.set_highlight_syntax(true);

        // Set up markdown language
        if let Some(ref language) = sourceview5::LanguageManager::new().language("markdown") {
            buffer.set_language(Some(language));
        } else {
            eprintln!("Warning: Markdown language definition not found");
        }

        // Set up syntax highlighting theme
        if let Some(ref scheme) = sourceview5::StyleSchemeManager::new().scheme("solarized-light") {
            buffer.set_style_scheme(Some(scheme));
        } else {
            // Fallback to default theme
            if let Some(ref scheme) = sourceview5::StyleSchemeManager::new().scheme("classic") {
                buffer.set_style_scheme(Some(scheme));
            }
        }

        buffer.set_text(&initial_content);

        Ok(buffer)
    }

    pub fn start_loading(app_state: AppState) {
        let mut loader = DocumentLoader::new(app_state);

        glib::MainContext::default().spawn_local(async move {
            let (buffer, doc_handle, router) = match loader.load_document().await {
                Err(e) => {
                    loader.app_state.show_error(&e.to_string());
                    return;
                }
                Ok((buffer, doc_handle, router)) => (buffer, doc_handle, router),
            };

            let doc_id = doc_handle.document_id();

            loader
                .app_state
                .update_document_id(&doc_id, router.endpoint().node_id());
            loader.app_state.setup_editor(&buffer);
            loader.app_state.show_editor();

            // Set up bidirectional synchronization
            let sync = TextSynchronizer::new(doc_handle, buffer);
            sync.start();

            // Keep polling remote infos until window is closed
            let window = loader.app_state.window.clone();
            loop {
                // Check if window is still visible/mapped
                if !window.is_visible() {
                    tracing::info!("Window closed, stopping remote info polling");
                    let _ = router.shutdown().await;
                    break;
                }

                let remote_infos: Vec<_> = router
                    .endpoint()
                    .remote_info_iter()
                    .filter(|info| {
                        match info.last_used {
                            Some(duration) => duration.as_secs_f64() <= 2.0,
                            None => true, // Include peers that have never been used (they're new)
                        }
                    })
                    .collect();

                for info in &remote_infos {
                    let node_id = info.node_id;
                    let is_active = info.has_send_address();
                    let (connection_type, connection_details) = match &info.conn_type {
                        iroh::endpoint::ConnectionType::Direct(addr) => {
                            let ip_version = if addr.is_ipv4() { "IPv4" } else { "IPv6" };
                            ("Direct", format!("({})", ip_version))
                        },
                        iroh::endpoint::ConnectionType::Relay(relay_url) => {
                            ("Relay", format!("({})", relay_url))
                        },
                        iroh::endpoint::ConnectionType::Mixed(addr, relay_url) => {
                            let ip_version = if addr.is_ipv4() { "IPv4" } else { "IPv6" };
                            ("Mixed", format!("({} + {})", ip_version, relay_url))
                        },
                        iroh::endpoint::ConnectionType::None => ("None", String::new()),
                    };
                    let latency = info
                        .latency
                        .map(|d| format!("{:.1}ms", d.as_secs_f64() * 1000.0))
                        .unwrap_or_else(|| "Unknown".to_string());
                    let last_used = info
                        .last_used
                        .map(|d| format!("{:.1}s ago", d.as_secs_f64()))
                        .unwrap_or_else(|| "Never".to_string());

                    tracing::info!(
                        "Remote peer: {} | Active: {} | Type: {}{} | Latency: {} | Last used: {}",
                        node_id,
                        is_active,
                        connection_type,
                        connection_details,
                        latency,
                        last_used
                    );
                }

                if remote_infos.is_empty() {
                    tracing::info!("No recently active remote peers");
                }

                // Sleep for a bit before next poll using glib
                glib::timeout_future(std::time::Duration::from_secs(5)).await;
            }
        });
    }
}
