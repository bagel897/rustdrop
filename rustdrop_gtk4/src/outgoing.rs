use adw::{prelude::*, subclass::prelude::*};
use gtk::gio::Cancellable;

mod imp {

    use std::sync::{Arc, Mutex};

    use adw::{ActionRow, HeaderBar, StatusPage, Window};
    use glib::clone;
    use gtk::{Button, FileDialog, ListBox};
    use rustdrop::Outgoing;

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
        pub outgoing_handle: Arc<Mutex<Outgoing>>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for OutgoingWindow {
        const NAME: &'static str = "OutgoingWindow";
        type Type = super::OutgoingWindow;
        type ParentType = adw::Bin;
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
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
            self.update_visibility();
        }
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }
    impl OutgoingWindow {
        fn update_visibility(&self) {
            let not_empty = self.outgoing_handle.lock().unwrap().len() > 0;
            self.send.set_visible(not_empty);
        }
    }
    #[gtk::template_callbacks]
    impl OutgoingWindow {
        #[template_callback]
        fn handle_send(&self) {
            self.obj().set_visible(false);
        }
        #[template_callback]
        fn handle_add_file(&self) {
            let dialog = FileDialog::new();
            let outgoing = self.outgoing_handle.clone();
            dialog.open(
                None::<&Window>,
                Cancellable::NONE,
                clone!(@weak self as this => move |res| {
                    let file = res.unwrap();
                    let path = file.path().unwrap();
                    let name = path.to_str().unwrap();
                    let row = ActionRow::builder().title(name).build();
                    outgoing.lock().unwrap().add_file(path);
                    this.outgoing.append(&row);
                    this.update_visibility();
                }),
            )
        }
    }
    impl WidgetImpl for OutgoingWindow {}
    impl BinImpl for OutgoingWindow {}
}

glib::wrapper! {
    pub struct OutgoingWindow(ObjectSubclass<imp::OutgoingWindow>) @extends gtk::Widget;
}
