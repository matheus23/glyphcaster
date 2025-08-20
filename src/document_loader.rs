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
        let rt = self.app_state.rt.clone();

        self.update_progress("Initializing iroh", 0.1).await;

        let endpoint = rt
            .spawn(async {
                // let secret_key = load_secret_key("./grammancy/keypair").await?;

                let endpoint = iroh::Endpoint::builder()
                    .discovery_n0()
                    // .secret_key(secret_key)
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
                        // Filesystem storage doesn't work right now
                        // .with_storage(samod::storage::TokioFilesystemStorage::new("./data"))
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
            match loader.load_document().await {
                Ok((buffer, doc_handle, router)) => {
                    // Get the document ID from the handle
                    let doc_id = doc_handle.document_id();

                    // Update the document ID in app state
                    loader.app_state.document_id.replace(doc_id.clone());
                    // Store the document handle
                    loader.app_state.doc_handle.replace(doc_handle.clone());

                    // Update the UI with document ID
                    loader
                        .app_state
                        .update_document_id(&doc_id, router.endpoint().node_id());
                    loader.app_state.setup_editor(&buffer);
                    loader.app_state.show_editor();

                    // Set up bidirectional synchronization
                    let sync = TextSynchronizer::new(doc_handle, buffer, router);
                    sync.start();
                    // Don't drop the router or other things
                    futures::future::pending().await
                }
                Err(e) => {
                    loader.app_state.show_error(&e.to_string());
                }
            }
        });
    }
}

/// Loads a [`SecretKey`] from the provided file, or stores a newly generated one
/// at the given location.
#[allow(unused)]
async fn load_secret_key(
    key_path: impl Into<std::path::PathBuf>,
) -> anyhow::Result<iroh::SecretKey> {
    use iroh::SecretKey;
    use tokio::io::AsyncWriteExt;

    let key_path = key_path.into();
    if key_path.exists() {
        let keystr = tokio::fs::read(key_path).await?;

        let ser_key = ssh_key::private::PrivateKey::from_openssh(keystr)?;
        let ssh_key::private::KeypairData::Ed25519(kp) = ser_key.key_data() else {
            anyhow::bail!("invalid key format");
        };
        let secret_key = SecretKey::from_bytes(&kp.private.to_bytes());
        Ok(secret_key)
    } else {
        let secret_key = SecretKey::generate(rand::rngs::OsRng);
        let ckey = ssh_key::private::Ed25519Keypair {
            public: secret_key.public().public().into(),
            private: secret_key.secret().into(),
        };
        let ser_key =
            ssh_key::private::PrivateKey::from(ckey).to_openssh(ssh_key::LineEnding::default())?;

        // Try to canonicalize if possible
        let key_path = key_path.canonicalize().unwrap_or(key_path);
        let key_path_parent = key_path.parent().ok_or_else(|| {
            anyhow::anyhow!("no parent directory found for '{}'", key_path.display())
        })?;
        tokio::fs::create_dir_all(&key_path_parent).await?;

        // write to tempfile
        let (file, temp_file_path) = tempfile::NamedTempFile::new_in(key_path_parent)
            .context("unable to create tempfile")?
            .into_parts();
        let mut file = tokio::fs::File::from_std(file);
        file.write_all(ser_key.as_bytes())
            .await
            .context("unable to write keyfile")?;
        file.flush().await?;
        drop(file);

        // move file
        tokio::fs::rename(temp_file_path, key_path)
            .await
            .context("failed to rename keyfile")?;

        Ok(secret_key)
    }
}
