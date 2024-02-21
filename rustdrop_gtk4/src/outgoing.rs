use adw::{prelude::*, subclass::prelude::*, ActionRow};

mod imp {

    use adw::{HeaderBar, StatusPage};
    use gtk::{Button, ListBox};

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(file = "blueprints/outgoing.blp")]
    pub struct OutgoingWindow {
        #[template_child]
        pub send: TemplateChild<Button>,
        #[template_child]
        add_file: TemplateChild<Button>,
        #[template_child]
        titlebar: TemplateChild<HeaderBar>,
        #[template_child]
        pub outgoing: TemplateChild<ListBox>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for OutgoingWindow {
        const NAME: &'static str = "OutgoingWindow";
        type Type = super::OutgoingWindow;
        type ParentType = adw::Bin;
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for OutgoingWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let placeholder = StatusPage::builder()
                .title("Nothing to Send")
                .description("Add some files or text to send")
                .build();
            self.outgoing.get().set_placeholder(Some(&placeholder));
        }
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }
    #[gtk::template_callbacks]
    impl OutgoingWindow {}
    impl WidgetImpl for OutgoingWindow {}
    impl BinImpl for OutgoingWindow {}
}

glib::wrapper! {
    pub struct OutgoingWindow(ObjectSubclass<imp::OutgoingWindow>) @extends gtk::Widget;
}
impl OutgoingWindow {
    pub fn update_visibility(&self, visibile: bool) {
        self.imp().send.set_visible(visibile)
    }
    pub fn add_row(&self, row: &ActionRow) {
        self.imp().outgoing.append(row);
    }
}
