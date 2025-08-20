use gio::ApplicationFlags;
use glib::ExitCode;
use gtk::prelude::*;
use samod::DocumentId;
use std::str::FromStr;

mod app_state;
mod document_loader;
mod sync;

use app_state::AppState;
use document_loader::DocumentLoader;

const APP_ID: &str = "xyz.patternist.glyphcaster";

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        // .with_thread_ids(true)
        // .with_thread_names(true)
        .with_level(true)
        // .with_target(false)
        .init();

    let application = adw::Application::new(Some(APP_ID), ApplicationFlags::HANDLES_COMMAND_LINE);
    application.connect_command_line(move |app, cli| {
        let doc_id = if cli.arguments().len() > 1 {
            let Some(automerge_url) = cli.arguments().get(1).cloned() else {
                eprintln!("No automerge URL provided");
                return ExitCode::FAILURE;
            };
            let Some(automerge_url) = automerge_url.to_str() else {
                eprintln!("automerge URL was not a valid UTF-8 string");
                return ExitCode::FAILURE;
            };
            let Some(doc_id) = automerge_url.trim().strip_prefix("automerge:") else {
                eprintln!("automerge URL doesn't have an 'automerge:' prefix");
                return ExitCode::FAILURE;
            };
            match DocumentId::from_str(doc_id) {
                Ok(doc_id) => Some(doc_id),
                Err(e) => {
                    eprintln!("Invalid document ID {doc_id}: {e}");
                    return ExitCode::FAILURE;
                }
            }
        } else {
            None
        };

        let node_id = if let Some(node_id) = cli.arguments().get(2).cloned() {
            let Some(node_id) = node_id.to_str() else {
                eprintln!("node ID was not a valid UTF-8 string");
                return ExitCode::FAILURE;
            };
            match iroh::NodeId::from_str(node_id.trim()) {
                Err(e) => {
                    eprintln!("Invalid node ID {node_id}: {e}");
                    return ExitCode::FAILURE;
                }
                Ok(node_id) => Some(node_id),
            }
        } else {
            None
        };

        let app_state = AppState::new(app, doc_id, node_id);

        // Show the window
        app_state.window.present();

        // Start the async loading process
        DocumentLoader::start_loading(app_state);

        ExitCode::SUCCESS
    });

    application.run();
}
