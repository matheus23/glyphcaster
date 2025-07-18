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

fn build_ui(application: &gtk::Application, doc_id: DocumentId) {
    let app_state = Rc::new(RefCell::new(AppState::new(application, doc_id)));

    // Show the window
    app_state.borrow().window.present();

    // Start the async loading process
    DocumentLoader::start_loading(app_state);
}

fn main() {
    // Read the document URL from stdin
    // let args: Vec<String> = std::env::args().collect();
    // let document_id_str = if args.len() > 1 {
    //     args[1].clone()
    // } else {
    //     eprintln!("no docuemnt ID specified");
    //     return;
    // };
    // let doc_id = match DocumentId::from_str(&document_id_str) {
    //     Ok(url) => url,
    //     Err(e) => {
    //         eprintln!("Invalid document id: {}", e);
    //         return;
    //     }
    // };

    let doc_id: DocumentId = "p8dpAaexjrpx2JFKbg5Z3a4NQyN".parse().unwrap();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_level(true)
        // .with_target(false)
        .init();

    // let application = gtk::Application::new(Some(APP_ID), ApplicationFlags::HANDLES_OPEN);
    let application = gtk::Application::new(Some(APP_ID), Default::default());
    application.connect_activate({
        let doc_id = doc_id.clone();
        move |app| build_ui(app, doc_id.clone())
    });

    application.run();
}
