mod consts;
mod daemon;
mod event_loop;
use adw::prelude::*;

use adw::{ActionRow, Application, ApplicationWindow, HeaderBar};
use consts::ID;
use daemon::Handler;
use gtk::glib::clone;
use gtk::{Box, ListBox, Orientation, SelectionMode};

fn main() {
    tracing_subscriber::fmt::init();
    let application = Application::builder().application_id(ID).build();
    application.connect_activate(|app| {
        let handler = Handler::new();
        let list = ListBox::builder()
            .margin_top(32)
            .margin_end(32)
            .margin_bottom(32)
            .margin_start(32)
            .selection_mode(SelectionMode::Single)
            // makes the list look nicer
            .css_classes(vec![String::from("boxed-list")])
            .build();
        glib::spawn_future_local(clone!(@weak list => async move {
            while let Ok(dev) = handler.get_device().await {
                let row = ActionRow::builder()
                    .activatable(true)
                    .title(dev.device_name)
                    .build();
                row.connect_activated(|_| {
                    eprintln!("Clicked!");
                });
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
            .title("First App")
            .default_width(350)
            // add content to window
            .content(&content)
            .build();
        window.present();
    });

    application.run();
}
