use adw::{prelude::*, subclass::prelude::*, ActionRow, Application};
use glib::subclass::InitializingObject;
use gtk::{gio::Cancellable, CompositeTemplate, FileDialog};
mod imp {

    use std::sync::{Arc, Mutex};

    use gtk::Stack;
    use rustdrop::Outgoing;

    use super::*;
    use crate::{discovery::DiscoveryWindow, outgoing::OutgoingWindow};

    #[derive(CompositeTemplate, Default)]
    #[template(file = "blueprints/main.blp")]
    pub struct Window {
        #[template_child]
        view: TemplateChild<Stack>,
        #[template_child]
        discovery: TemplateChild<DiscoveryWindow>,
        #[template_child]
        pub outgoing: TemplateChild<OutgoingWindow>,
        pub outgoing_handle: Arc<Mutex<Outgoing>>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "NearbySharing";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            OutgoingWindow::ensure_type();
            klass.bind_template();
            klass.install_action("win.send", None, |window, _, _| {
                window.imp().outgoing.set_visible(false);
            });
            klass.install_action_async("win.add_file", None, |window, _, _| async move {
                window.imp().handle_add_file().await;
            })
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl Window {
        pub fn update_visibility(&self) {
            let not_empty = self.outgoing_handle.lock().unwrap().len() > 0;
            self.outgoing.update_visibility(not_empty);
        }
        async fn handle_add_file(&self) {
            let dialog = FileDialog::new();
            if let Ok(file) = dialog.open_future(Some(&self.obj().clone())).await {
                let path = file.path().unwrap();
                let name = path.to_str().unwrap();
                let row = ActionRow::builder().title(name).build();
                self.outgoing_handle.lock().unwrap().add_file(path);
                self.outgoing.add_row(&row);
                self.update_visibility();
            }
        }
    }
    impl ObjectImpl for Window {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
            self.discovery
                .get()
                .set_outgoing(self.outgoing_handle.clone());
            self.update_visibility();
        }
    }
    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl AdwWindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

use glib::Object;
use gtk::{gio, glib};

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, adw::Window, gtk::Widget, gtk::Window,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }
}
