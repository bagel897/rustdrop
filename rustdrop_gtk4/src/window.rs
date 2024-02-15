use adw::{prelude::*, subclass::prelude::*, Application};
use glib::subclass::InitializingObject;
use gtk::CompositeTemplate;
mod imp {

    use gtk::Stack;

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
        outgoing: TemplateChild<OutgoingWindow>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "NearbySharing";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for Window {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
            self.discovery
                .get()
                .set_outgoing(self.outgoing.get().imp().outgoing_handle.clone())
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
