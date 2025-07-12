use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

mod app_state;
mod document_loader;
mod sync;

use app_state::AppState;
use document_loader::DocumentLoader;

const APP_ID: &str = "xyz.patternist.rust-essay-editor";

fn build_ui(application: &gtk::Application) {
    let app_state = Rc::new(RefCell::new(AppState::new(application)));

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
    let application = gtk::Application::new(Some(APP_ID), Default::default());
    application.connect_activate(build_ui);

    application.run();
}
