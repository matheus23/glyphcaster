use gio::ApplicationFlags;
use gtk::prelude::*;
use samod::DocumentId;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

mod app_state;
mod document_loader;
mod sync;

use app_state::AppState;
use document_loader::DocumentLoader;

const APP_ID: &str = "xyz.patternist.rust-essay-editor";

fn build_ui(application: &gtk::Application, doc_id: Option<DocumentId>) {
    let app_state = Rc::new(RefCell::new(AppState::new(application, doc_id)));

    // Show the window
    app_state.borrow().window.present();

    // Start the async loading process
    DocumentLoader::start_loading(app_state);
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_level(true)
        // .with_target(false)
        .init();

    let application = gtk::Application::new(Some(APP_ID), ApplicationFlags::HANDLES_COMMAND_LINE);
    application.connect_command_line(move |app, cli| {
        let doc_id = if cli.arguments().len() > 1 {
            let Some(doc_id_os_str) = cli.arguments().get(1).cloned() else {
                eprintln!("No document ID provided");
                return 1;
            };
            let Some(doc_id_str) = doc_id_os_str.to_str() else {
                eprintln!("document ID was not a valid UTF-8 string");
                return 1;
            };
            match DocumentId::from_str(doc_id_str.trim()) {
                Ok(doc_id) => Some(doc_id),
                Err(e) => {
                    eprintln!("Invalid document ID {}: {}", doc_id_str, e);
                    return 1;
                }
            }
        } else {
            None
        };

        build_ui(app, doc_id);
        0
    });

    application.run();
}
