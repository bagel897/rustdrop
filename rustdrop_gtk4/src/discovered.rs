use rustdrop::DiscoveryHandle;
use std::sync::{Arc, Mutex};

use adw::subclass::prelude::*;
use glib::Object;
use rustdrop::Outgoing;

mod imp {

    use std::cell::OnceCell;

    use gtk::ProgressBar;
    use rustdrop::SenderEvent;

    use crate::event_loop::runtime;

    use super::*;
    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(file = "blueprints/discovered.blp")]
    pub struct DiscoveredRow {
        pub handle: OnceCell<DiscoveryHandle>,
        pub outgoing_handle: OnceCell<Arc<Mutex<Outgoing>>>,
        #[template_child]
        progress: TemplateChild<ProgressBar>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for DiscoveredRow {
        const NAME: &'static str = "DiscoveredRow";
        type Type = super::DiscoveredRow;
        type ParentType = adw::ActionRow;
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    #[gtk::template_callbacks]
    impl DiscoveredRow {
        #[template_callback]
        async fn handle_activate(&self) {
            let outgoing = self.outgoing_handle.get().unwrap().lock().unwrap().clone();
            let rx = self
                .handle
                .get()
                .unwrap()
                .send_file(outgoing, runtime().handle())
                .unwrap();
            self.progress.set_text(Some("Sending"));
            self.progress.set_fraction(0.25);
            while let Ok(event) = rx.recv_async().await {
                match event {
                    SenderEvent::Accepted() => {
                        self.progress.set_fraction(0.75);
                        self.progress.set_text(Some("Accepted"));
                    }
                    SenderEvent::AwaitingResponse() => {
                        self.progress.set_text(Some("Awaiting Response"));
                        self.progress.set_fraction(0.5);
                    }
                    SenderEvent::Finished() => {
                        self.progress.set_text(Some("Finished"));
                        self.progress.set_fraction(1.0);
                    }
                    SenderEvent::Rejected() => {
                        self.progress.set_text(Some("Rejected"));
                        self.progress.set_fraction(1.0);
                        break;
                    }
                }
            }
        }
    }
    impl WidgetImpl for DiscoveredRow {}
    impl ObjectImpl for DiscoveredRow {}
    impl ListBoxRowImpl for DiscoveredRow {}
    impl PreferencesRowImpl for DiscoveredRow {}
    impl ActionRowImpl for DiscoveredRow {}
    impl SpinRowImpl for DiscoveredRow {}
}
glib::wrapper! {
    pub struct DiscoveredRow(ObjectSubclass<imp::DiscoveredRow>) @extends adw::SpinRow, adw::ActionRow, adw::PreferencesRow,gtk::Widget;
}
impl DiscoveredRow {
    pub fn new(handle: DiscoveryHandle, outgoing: Arc<Mutex<Outgoing>>) -> Self {
        let res: Self = Object::builder()
            .property("title", &handle.device().device_name)
            .property("subtitle", &format!("{:?}", handle.device().device_type))
            .build();
        res.init(handle, outgoing);
        res
    }
    pub fn init(&self, handle: DiscoveryHandle, outgoing: Arc<Mutex<Outgoing>>) {
        self.imp().handle.set(handle).unwrap();
        self.imp().outgoing_handle.set(outgoing).unwrap();
    }
}
