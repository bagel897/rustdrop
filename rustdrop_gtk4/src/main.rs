mod consts;
mod daemon;
mod event_loop;
use std::collections::HashSet;

use adw::prelude::*;

use adw::{ActionRow, Application, ApplicationWindow, HeaderBar};
use consts::ID;
use daemon::run_child;
use event_loop::runtime;
use gtk::glib::clone;
use gtk::{Box, ListBox, Orientation, SelectionMode};
use rustdrop::DiscoveryEvent;

fn main() {
    tracing_subscriber::fmt::init();
    let application = Application::builder().application_id(ID).build();
    application.connect_activate(|app| {
        let (tx, rx) = flume::bounded(1);
        let (tx_send, rx_send) = flume::unbounded();
        runtime().spawn(async move { run_child(rx_send, tx).await });
        let list = ListBox::builder()
            .margin_top(32)
            .margin_end(32)
            .margin_bottom(32)
            .margin_start(32)
            .selection_mode(SelectionMode::Single)
            // makes the list look nicer
            .css_classes(vec![String::from("boxed-list")])
            .build();
        let discovery = rx.recv().unwrap();
        glib::spawn_future_local(clone!(@weak list => async move {
            let mut seen = HashSet::new();
            while let Ok(DiscoveryEvent::Discovered(dev)) = discovery.recv_async().await {
                if seen.contains(&dev) {
                    continue;
                }
                seen.insert(dev.clone());
                let row = ActionRow::builder()
                    .activatable(true)
                    .title(format!("{}: {:?}",dev.device_name.clone(), dev.discovery))
                    .build();
                row.connect_activated( clone!(@strong tx_send => move |_| {
                    tx_send.send(dev.clone()).unwrap();
                }));
                list.append(&row);
            }
        }));

        // Combine the content in a box
        let content = Box::new(Orientation::Vertical, 0);
        // Adwaitas' ApplicationWindow does not include a HeaderBar
        content.append(&HeaderBar::new());
        content.append(&list);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Nearby Sharing")
            .default_width(350)
            // add content to window
            .content(&content)
            .build();
        window.present();
    });

    application.run();
}
