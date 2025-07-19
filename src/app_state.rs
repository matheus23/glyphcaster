use gtk::{glib, prelude::*};
use samod::DocumentId;
use sourceview5::prelude::*;

pub struct LoadingPageWidgets {
    pub container: gtk::Box,
    pub label: gtk::Label,
    pub spinner: gtk::Spinner,
    pub progress_bar: gtk::ProgressBar,
}

#[derive(Clone)]
pub struct AppState {
    pub document_id: Option<DocumentId>,
    pub window: gtk::ApplicationWindow,
    pub main_stack: gtk::Stack,
    pub loading_page: gtk::Box,
    pub editor_page: gtk::Box,
    pub loading_label: gtk::Label,
    pub loading_spinner: gtk::Spinner,
    pub progress_bar: gtk::ProgressBar,
}

impl AppState {
    pub fn new(application: &gtk::Application, doc_id: Option<DocumentId>) -> Self {
        let window = gtk::ApplicationWindow::new(application);
        window.set_title(Some("Rusty Essay Editor"));
        window.set_default_size(800, 600);

        // Create main stack to switch between loading and editor
        let main_stack = gtk::Stack::new();
        main_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        main_stack.set_transition_duration(300);

        // Create loading page
        let LoadingPageWidgets {
            container: loading_page,
            label: loading_label,
            spinner: loading_spinner,
            progress_bar,
        } = Self::create_loading_page();

        // Create editor page
        let editor_page = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        // Add pages to stack
        main_stack.add_named(&loading_page, Some("loading"));
        main_stack.add_named(&editor_page, Some("editor"));

        // Show loading page initially
        main_stack.set_visible_child_name("loading");

        window.set_child(Some(&main_stack));

        Self {
            document_id: doc_id,
            window,
            main_stack,
            loading_page,
            editor_page,
            loading_label,
            loading_spinner,
            progress_bar,
        }
    }

    fn create_loading_page() -> LoadingPageWidgets {
        let loading_page = gtk::Box::new(gtk::Orientation::Vertical, 20);
        loading_page.set_halign(gtk::Align::Center);
        loading_page.set_valign(gtk::Align::Center);
        loading_page.set_margin_top(50);
        loading_page.set_margin_bottom(50);
        loading_page.set_margin_start(50);
        loading_page.set_margin_end(50);

        let loading_spinner = gtk::Spinner::new();
        loading_spinner.set_size_request(48, 48);
        loading_spinner.start();

        let loading_label = gtk::Label::new(Some("Initializing Samod..."));
        loading_label.set_markup("<span size='large'>Initializing Samod...</span>");

        let progress_bar = gtk::ProgressBar::new();
        progress_bar.set_size_request(300, -1);
        progress_bar.set_show_text(true);

        loading_page.append(&loading_spinner);
        loading_page.append(&loading_label);
        loading_page.append(&progress_bar);

        LoadingPageWidgets {
            container: loading_page,
            label: loading_label,
            spinner: loading_spinner,
            progress_bar,
        }
    }

    pub fn update_loading_status(&self, message: &str, progress: Option<f64>) {
        self.loading_label
            .set_markup(&format!("<span size='large'>{}</span>", message));

        if let Some(progress) = progress {
            self.progress_bar.set_fraction(progress);
            self.progress_bar
                .set_text(Some(&format!("{:.0}%", progress * 100.0)));
        } else {
            self.progress_bar.pulse();
        }
    }

    pub fn show_editor(&self) {
        self.loading_spinner.stop();
        self.main_stack.set_visible_child_name("editor");
    }

    pub fn setup_editor(&self, buffer: &sourceview5::Buffer) {
        // Clear any existing children
        while let Some(child) = self.editor_page.first_child() {
            self.editor_page.remove(&child);
        }

        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let scroll = gtk::ScrolledWindow::builder()
            .vscrollbar_policy(gtk::PolicyType::External)
            .build();

        let view = sourceview5::View::with_buffer(buffer);
        view.set_monospace(true);
        view.set_background_pattern(sourceview5::BackgroundPatternType::Grid);
        view.set_show_line_numbers(true);
        view.set_highlight_current_line(true);
        view.set_tab_width(4);
        view.set_hexpand(true);
        view.set_auto_indent(true);
        view.set_insert_spaces_instead_of_tabs(true);
        view.set_smart_backspace(true);
        view.set_smart_home_end(sourceview5::SmartHomeEndType::Before);

        scroll.set_child(Some(&view));
        container.append(&scroll);

        let map = sourceview5::Map::new();
        map.set_view(&view);
        container.append(&map);

        self.editor_page.append(&container);
    }

    pub fn show_error(&self, error_message: &str) {
        self.loading_spinner.stop();
        self.loading_label.set_markup(&format!(
            "<span size='large' color='red'>Error: {}</span>",
            glib::markup_escape_text(error_message)
        ));
        self.progress_bar.set_fraction(0.0);
        self.progress_bar.set_text(Some("Failed"));
    }

    pub fn retry_loading<F>(&self, retry_callback: F)
    where
        F: Fn() + 'static,
    {
        // Reset loading state
        self.loading_spinner.start();
        self.progress_bar.set_fraction(0.0);
        self.progress_bar.set_text(Some(""));
        self.main_stack.set_visible_child_name("loading");

        // Add a retry button to the loading page
        let retry_button = gtk::Button::with_label("Retry");
        retry_button.connect_clicked(move |_| {
            retry_callback();
        });

        self.loading_page.append(&retry_button);
    }
}
