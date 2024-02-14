mod consts;
mod daemon;
mod event_loop;
mod outgoing;
mod window;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use adw::{
    prelude::*, ActionRow, Application, ApplicationWindow, HeaderBar, NavigationPage,
    NavigationView, StatusPage,
};
use consts::ID;
use daemon::run_child;
use event_loop::runtime;
use gtk::{
    gio::Cancellable, glib::clone, Box, Button, FileDialog, ListBox, Orientation, SelectionMode,
    Stack, StackPage, Widget,
};
use rustdrop::{DiscoveryEvent, Outgoing};
use window::Window;
fn build_outgoing(outgoing: Arc<Mutex<Outgoing>>) -> impl IsA<Widget> {
    let stack = Stack::new();
    let list = ListBox::builder()
        .margin_top(32)
        .margin_end(32)
        .margin_bottom(32)
        .margin_start(32)
        .selection_mode(SelectionMode::Single)
        // makes the list look nicer
        .css_classes(vec![String::from("boxed-list")])
        .build();
    let button = Button::with_label("Add file");
    button.connect_clicked(clone!(@weak list => move |_| {
        let dialog = FileDialog::new();
        dialog.open(None::<&Window>, Cancellable::NONE,clone!(@strong outgoing => move |res| {
            let file = res.unwrap();
            let path = file.path().unwrap();
            let row = ActionRow::builder().name(path.clone().to_str().unwrap()).build();
            outgoing.lock().unwrap().add_file(path);
            list.append(&row);
        }))
    }));
    let status = StatusPage::builder().name("No files selected").build();
    stack.add_child(&list);
    let header = HeaderBar::new();
    header.pack_start(&button);

    let content = Box::new(Orientation::Vertical, 0);
    // Adwaitas' ApplicationWindow does not include a HeaderBar
    content.append(&header);
    content.append(&stack);
    content
}
fn discovery(outgoing: Outgoing) -> NavigationPage {
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
            let outgoing = outgoing.clone();
            row.connect_activated( clone!(@strong tx_send => move |_| {
                tx_send.send((dev.clone(), outgoing.clone())).unwrap();
            }));
            list.append(&row);
        }
    }));
    let content = Box::new(Orientation::Vertical, 0);
    // Adwaitas' ApplicationWindow does not include a HeaderBar
    content.append(&HeaderBar::new());
    content.append(&list);
    NavigationPage::new(&content, "Send")
}
fn main() {
    tracing_subscriber::fmt::init();
    let application = Application::builder().application_id(ID).build();
    // application.connect_activate(|app| {
    //     let outgoing = Arc::new(Mutex::new(Outgoing::default()));
    //
    //     let nav = Stack::new();
    //     nav.add_named(&build_outgoing(outgoing), Some("Add files"));
    //
    //     // let outgoing = OnceCell::default();
    //     // let list = match outgoing.get() {
    //     //     Some(outgoing) => discovery(outgoing),
    //     //     None => build_outgoing(),
    //     // };
    //     let window = ApplicationWindow::builder()
    //         .application(app)
    //         .title("Nearby Sharing")
    //         .default_width(350)
    //         // add content to window
    //         .content(&nav)
    //         .build();
    //     window.present();
    // });
    application.connect_activate(build_ui);
    application.run();
}
fn build_ui(app: &Application) {
    // Create new window and present it
    let window = Window::new(app);
    window.present();
}
