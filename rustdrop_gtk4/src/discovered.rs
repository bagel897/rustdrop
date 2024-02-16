use std::sync::{Arc, Mutex};

use adw::subclass::prelude::*;
use glib::Object;
use rustdrop::Outgoing;

use crate::daemon::DiscoveryHandle;
mod imp {

    use std::{
        cell::OnceCell,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use flume::RecvTimeoutError;
    use glib::clone;
    use gtk::ProgressBar;
    use rustdrop::{Outgoing, SenderEvent};

    use super::*;
    use crate::daemon::DiscoveryHandle;
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
        fn handle_activate(&self) {
            eprintln!("Activated");
            let outgoing = self.outgoing_handle.get().unwrap().lock().unwrap().clone();
            let rx = self.handle.get().unwrap().send(outgoing);
            glib::spawn_future_local(clone!(@weak self as this => async move {
                            this.progress.pulse();
                            this.progress.set_text(Some("Sending"));
                            while let Ok(event) = rx.recv_async().await {
                                match event {
                                    SenderEvent::Accepted() => {
                                        this.progress.set_text(Some("Accepted"));
                                    }
                                    SenderEvent::AwaitingResponse() => {
                            this.progress.set_text(Some("Awaiting Response"));
                            this.progress.set_fraction(0.5);
                                break;
                            }
                                    SenderEvent::Finished() => {
                            this.progress.set_text(Some("Finished"));
                            this.progress.set_fraction(1.0);
                                break;
                            }
                                    SenderEvent::Rejected() => {
                                        this.progress.set_text(Some("Rejected"));
                                        this.progress.set_fraction(1.0);
                                        return;
                                    }
                                }
            }

                        }));
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
            .property("title", &handle.device.device_name)
            .property("subtitle", &format!("{:?}", handle.device.discovery))
            .build();
        res.init(handle, outgoing);
        res
    }
    pub fn init(&self, handle: DiscoveryHandle, outgoing: Arc<Mutex<Outgoing>>) {
        self.imp().handle.set(handle).unwrap();
        self.imp().outgoing_handle.set(outgoing).unwrap();
    }
}
