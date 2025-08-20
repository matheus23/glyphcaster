use gtk::{glib, prelude::*};
use samod::DocumentId;
use sourceview5::prelude::*;

pub struct LoadingPageWidgets {
    pub container: gtk::Box,
    pub label: gtk::Label,
    pub spinner: gtk::Spinner,
    pub progress_bar: gtk::ProgressBar,
}

pub struct AppState {
    pub rt: tokio::runtime::Runtime,
    pub document_id: Option<DocumentId>,
    pub node_id: Option<iroh::NodeId>,
    pub iroh_secret: Option<String>,
    pub window: gtk::ApplicationWindow,
    pub main_stack: gtk::Stack,
    #[allow(unused)]
    pub loading_page: gtk::Box,
    pub editor_page: gtk::Box,
    pub header_bar: gtk::HeaderBar,
    pub doc_id_label: gtk::Label,
    pub copy_button: gtk::Button,
    pub loading_label: gtk::Label,
    pub loading_spinner: gtk::Spinner,
    pub progress_bar: gtk::ProgressBar,
    pub side_pane: Option<gtk::Box>,
}

impl AppState {
    pub fn new(
        application: &adw::Application,
        doc_id: Option<DocumentId>,
        node_id: Option<iroh::NodeId>,
        iroh_secret: Option<String>,
    ) -> Self {
        let window = gtk::ApplicationWindow::new(application);
        window.set_title(Some("Glyphcaster"));
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

        // Create editor page with vertical orientation to include header
        let editor_page = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Create header bar for document info
        let header_bar = gtk::HeaderBar::new();
        header_bar.set_show_title_buttons(false);

        // Create a box to hold the document ID label and copy button
        let doc_id_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        doc_id_box.set_halign(gtk::Align::Center);

        // Create label for document ID
        let doc_id_label = gtk::Label::new(Some("Document ID: Loading..."));
        doc_id_label.set_css_classes(&["subtitle"]);
        doc_id_label.set_selectable(true);
        doc_id_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);

        // Create copy button
        let copy_button = gtk::Button::from_icon_name("edit-copy-symbolic");
        copy_button.set_tooltip_text(Some("Copy Document ID"));
        copy_button.set_has_frame(false);
        copy_button.set_sensitive(false); // Disabled until ID is loaded

        doc_id_box.append(&doc_id_label);
        doc_id_box.append(&copy_button);

        header_bar.set_title_widget(Some(&doc_id_box));

        editor_page.append(&header_bar);

        // Add pages to stack
        main_stack.add_named(&loading_page, Some("loading"));
        main_stack.add_named(&editor_page, Some("editor"));

        // Show loading page initially
        main_stack.set_visible_child_name("loading");

        window.set_child(Some(&main_stack));

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");

        Self {
            rt,
            document_id: doc_id,
            node_id,
            iroh_secret,
            window,
            main_stack,
            loading_page,
            editor_page,
            header_bar,
            doc_id_label,
            copy_button,
            loading_label,
            loading_spinner,
            progress_bar,
            side_pane: None,
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

    pub fn setup_editor(&mut self, buffer: &sourceview5::Buffer) {
        // Create a new container for the editor content
        let main_container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        main_container.set_vexpand(true);
        main_container.set_hexpand(true);

        // Create the editor area (left side)
        let editor_container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        editor_container.set_vexpand(true);
        editor_container.set_hexpand(true);

        let scroll = gtk::ScrolledWindow::builder()
            .vscrollbar_policy(gtk::PolicyType::External)
            .hexpand(true)
            .vexpand(true)
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
        editor_container.append(&scroll);

        let map = sourceview5::Map::new();
        map.set_view(&view);
        editor_container.append(&map);

        // Add editor container to main container
        main_container.append(&editor_container);

        // Create the side pane (right side)
        let side_pane = gtk::Box::new(gtk::Orientation::Vertical, 8);
        side_pane.set_width_request(250);
        side_pane.set_vexpand(true);
        side_pane.set_margin_top(8);
        side_pane.set_margin_bottom(8);
        side_pane.set_margin_start(8);
        side_pane.set_margin_end(8);
        side_pane.add_css_class("sidebar");

        // Add a placeholder label for iroh peers
        let placeholder_label = gtk::Label::new(Some("No Active Iroh Peers"));
        placeholder_label.set_halign(gtk::Align::Center);
        placeholder_label.set_valign(gtk::Align::Start);
        placeholder_label.set_margin_top(16);
        placeholder_label.add_css_class("dim-label");

        side_pane.append(&placeholder_label);

        // Add side pane to main container
        main_container.append(&side_pane);

        // Remove any existing editor content (but keep the header bar)
        let mut child = self.editor_page.first_child();
        while let Some(widget) = child {
            let next = widget.next_sibling();
            if &widget != &self.header_bar {
                self.editor_page.remove(&widget);
            }
            child = next;
        }

        // Store reference to side pane for later updates
        self.side_pane = Some(side_pane.clone());

        // Add the new main container
        self.editor_page.append(&main_container);
    }

    pub fn update_document_id(&self, doc_id: &DocumentId, node_id: iroh::NodeId) {
        let doc_id_string = doc_id.to_string();
        let connection_string = format!("automerge:{doc_id_string} {node_id}");
        self.doc_id_label
            .set_text(&format!("Connect using: {connection_string}"));
        self.doc_id_label.set_tooltip_text(Some(&format!(
            "Full connnection string: {connection_string}"
        )));

        // Enable the copy button and set up its click handler
        self.copy_button.set_sensitive(true);

        let window = self.window.clone();
        self.copy_button.connect_clicked(move |_| {
            // Copy to clipboard
            let display = gtk::prelude::WidgetExt::display(&window);
            let clipboard = display.clipboard();
            clipboard.set_text(&connection_string);

            // Show a toast or notification (optional)
            println!("Connection string copied to clipboard: {connection_string}");
        });
    }

    pub fn update_remote_peers(&self, peer_infos: Vec<iroh::endpoint::RemoteInfo>) {
        if let Some(ref side_pane) = self.side_pane {
            // Clear existing content except the first child (placeholder)
            let mut child = side_pane.first_child();
            let mut children_to_remove = Vec::new();
            let mut first = true;
            
            while let Some(widget) = child {
                if !first {
                    children_to_remove.push(widget.clone());
                }
                first = false;
                child = widget.next_sibling();
            }
            
            for widget in children_to_remove {
                side_pane.remove(&widget);
            }
            
            // Update the placeholder or add peer info
            if peer_infos.is_empty() {
                if let Some(placeholder) = side_pane.first_child().and_then(|w| w.downcast::<gtk::Label>().ok()) {
                    placeholder.set_text("No Active Iroh Peers");
                }
            } else {
                if let Some(placeholder) = side_pane.first_child().and_then(|w| w.downcast::<gtk::Label>().ok()) {
                    placeholder.set_text("Connected Iroh Peers");
                }
                
                for info in peer_infos {
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
                    
                    let latency = info.latency
                        .map(|d| format!("{:.1}ms", d.as_secs_f64() * 1000.0))
                        .unwrap_or_else(|| "Unknown".to_string());
                    
                    // Create a container for each peer
                    let peer_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
                    peer_box.add_css_class("card");
                    
                    // Node ID header with icon
                    let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                    
                    let node_icon = gtk::Label::new(Some("🔗"));
                    header_box.append(&node_icon);
                    
                    let node_id_label = gtk::Label::new(Some(&format!(
                        "{}...", 
                        info.node_id.to_string().chars().take(12).collect::<String>()
                    )));
                    node_id_label.set_halign(gtk::Align::Start);
                    node_id_label.add_css_class("heading");
                    node_id_label.set_selectable(true);
                    header_box.append(&node_id_label);
                    
                    peer_box.append(&header_box);
                    
                    // Connection info
                    let conn_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                    
                    let conn_icon = match connection_type {
                        "Direct" => "🔌",
                        "Relay" => "📡", 
                        "Mixed" => "🔀",
                        _ => "❌"
                    };
                    let conn_icon_label = gtk::Label::new(Some(conn_icon));
                    conn_box.append(&conn_icon_label);
                    
                    let conn_label = gtk::Label::new(Some(&format!("{}{}", connection_type, connection_details)));
                    conn_label.set_halign(gtk::Align::Start);
                    conn_label.add_css_class("caption");
                    conn_box.append(&conn_label);
                    
                    peer_box.append(&conn_box);
                    
                    // Latency info
                    let latency_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                    
                    let latency_icon = gtk::Label::new(Some("⏱️"));
                    latency_box.append(&latency_icon);
                    
                    let latency_label = gtk::Label::new(Some(&latency));
                    latency_label.set_halign(gtk::Align::Start);
                    latency_label.add_css_class("caption");
                    latency_box.append(&latency_label);
                    
                    peer_box.append(&latency_box);
                    
                    side_pane.append(&peer_box);
                }
            }
        }
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
}
