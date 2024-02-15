use std::sync::{Arc, Mutex};

use adw::{prelude::*, subclass::prelude::*};
use rustdrop::Outgoing;

// fn discovery(outgoing: Outgoing) -> NavigationPage {
//     let (tx, rx) = flume::bounded(1);
//     let (tx_send, rx_send) = flume::unbounded();
//     runtime().spawn(async move { run_child(rx_send, tx).await });
//     let list = ListBox::builder()
//         .margin_top(32)
//         .margin_end(32)
//         .margin_bottom(32)
//         .margin_start(32)
//         .selection_mode(SelectionMode::Single)
//         // makes the list look nicer
//         .css_classes(vec![String::from("boxed-list")])
//         .build();
//     let discovery = rx.recv().unwrap();
//     glib::spawn_future_local(clone!(@weak list => async move {
//         let mut seen = HashSet::new();
//         while let Ok(DiscoveryEvent::Discovered(dev)) = discovery.recv_async().await {
//             if seen.contains(&dev) {
//                 continue;
//             }
//             seen.insert(dev.clone());
//             let row = ActionRow::builder()
//                 .activatable(true)
//                 .title(format!("{}: {:?}",dev.device_name.clone(), dev.discovery))
//                 .build();
//             let outgoing = outgoing.clone();
//             row.connect_activated( clone!(@strong tx_send => move |_| {
//                 tx_send.send((dev.clone(), outgoing.clone())).unwrap();
//             }));
//             list.append(&row);
//         }
//     }));
//     let content = Box::new(Orientation::Vertical, 0);
//     // Adwaitas' ApplicationWindow does not include a HeaderBar
//     content.append(&HeaderBar::new());
//     content.append(&list);
//     NavigationPage::new(&content, "Send")
// }
mod imp {

    use std::{
        cell::OnceCell,
        sync::{Arc, Mutex},
    };

    use adw::{HeaderBar, StatusPage};
    use futures_util::{pin_mut, StreamExt};
    use glib::clone;
    use gtk::ListBox;
    use rustdrop::Outgoing;

    use super::*;
    use crate::{daemon::DaemonHandle, discovered::DiscoveredRow};

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(file = "blueprints/discovery.blp")]
    pub struct DiscoveryWindow {
        // #[template_child]
        // pub send: TemplateChild<Button>,
        // #[template_child]
        // add_file: TemplateChild<Button>,
        #[template_child]
        titlebar: TemplateChild<HeaderBar>,
        #[template_child]
        pub discovery: TemplateChild<ListBox>,
        pub outgoing_handle: OnceCell<Arc<Mutex<Outgoing>>>,
        pub discovery_handle: DaemonHandle,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for DiscoveryWindow {
        const NAME: &'static str = "DiscoveryWindow";
        type Type = super::DiscoveryWindow;
        type ParentType = adw::Bin;
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            // klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DiscoveryWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let placeholder = StatusPage::builder()
                .title("No devices found")
                .description("Searching")
                .build();
            self.discovery.get().set_placeholder(Some(&placeholder));
            glib::spawn_future_local(clone!(@weak self as this => async move {
            let discovery = this.discovery_handle.recv();
                pin_mut!(discovery);
                            while let Some(handle) = discovery.next().await {
                                let row = DiscoveredRow::new(handle, this.outgoing_handle.get().unwrap().clone());

                                this.discovery.append(&row);
                            }
                        }));
        }
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }
    impl WidgetImpl for DiscoveryWindow {}
    impl BinImpl for DiscoveryWindow {}
}

glib::wrapper! {
    pub struct DiscoveryWindow(ObjectSubclass<imp::DiscoveryWindow>) @extends gtk::Widget;
}
impl DiscoveryWindow {
    pub fn set_outgoing(&self, outgoing: Arc<Mutex<Outgoing>>) {
        self.imp().outgoing_handle.set(outgoing).unwrap();
    }
}
