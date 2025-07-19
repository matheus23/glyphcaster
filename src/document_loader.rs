use crate::app_state::AppState;
use crate::sync::TextSynchronizer;
use automerge::transaction::Transactable;
use automerge::{Automerge, AutomergeError, ObjType, ROOT, ReadDoc};
use gtk::glib;
use samod::{ConnDirection, DocHandle};
use sourceview5::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct DocumentLoader {
    app_state: Rc<RefCell<AppState>>,
}

impl DocumentLoader {
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        Self { app_state }
    }

    pub async fn load_document(
        &self,
    ) -> Result<(sourceview5::Buffer, DocHandle), Box<dyn std::error::Error>> {
        // Step 1: Initialize Samod
        self.update_progress("Initializing Samod...", 0.1).await;

        let samod = samod::Samod::build_gio()
            .with_storage(samod::storage::GioFilesystemStorage::new("./data"))
            .load()
            .await;

        // Step 1: Initialize Samod
        self.update_progress("Connecting to sync server", 0.1).await;

        let (conn, _) = async_tungstenite::gio::connect_async("wss://sync.automerge.org")
            .await
            .unwrap();

        let conn = samod.connect_tungstenite(conn, ConnDirection::Outgoing);
        glib::spawn_future(async move {
            let result = conn.await;
            tracing::info!(?result, "Samod connection finished");
        });

        samod
            .when_connected("storage-server-sync-automerge-org".into())
            .await
            .unwrap();

        // Step 2: Load the document
        self.update_progress("Loading document...", 0.5).await;

        let handle = if let Some(doc_id) = self.app_state.borrow().document_id.clone() {
            samod.find(doc_id).await.unwrap().unwrap()
        } else {
            let mut doc = Automerge::new();
            doc.transact::<_, _, AutomergeError>(|tx| {
                let text_id = tx.put_object(ROOT, "content", ObjType::Text)?;
                tx.splice_text(&text_id, 0, 0, "# Untitled")?;
                Ok(())
            })
            .unwrap();
            samod.create(doc).await.unwrap()
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

        // Step 3: Setup the buffer
        self.update_progress("Setting up editor...", 0.9).await;

        let buffer = self.create_markdown_buffer(content).await?;

        // Final step
        self.update_progress("Ready!", 1.0).await;
        glib::timeout_future(std::time::Duration::from_millis(200)).await;

        Ok((buffer, handle))
    }

    async fn update_progress(&self, message: &str, progress: f64) {
        let state = self.app_state.borrow();
        state.update_loading_status(message, Some(progress));

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

    pub fn start_loading(app_state: Rc<RefCell<AppState>>) {
        let loader = DocumentLoader::new(app_state.clone());

        glib::MainContext::default().spawn_local(async move {
            match loader.load_document().await {
                Ok((buffer, doc_handle)) => {
                    // Get the document ID from the handle
                    let doc_id = doc_handle.document_id();

                    {
                        let mut state = app_state.borrow_mut();
                        // Update the document ID in app state
                        state.document_id = Some(doc_id.clone());
                        // Store the document handle
                        *state.doc_handle.borrow_mut() = Some(doc_handle.clone());
                    }

                    {
                        let state = app_state.borrow();
                        // Update the UI with document ID
                        state.update_document_id(&doc_id);
                        state.setup_editor(&buffer);
                        state.show_editor();
                    }

                    // Set up bidirectional synchronization
                    let sync = TextSynchronizer::new(doc_handle, buffer);
                    sync.start();
                }
                Err(e) => {
                    let state = app_state.borrow();
                    state.show_error(&e.to_string());
                }
            }
        });
    }
}
