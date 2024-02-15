mod consts;
mod daemon;
mod discovered;
mod discovery;
mod event_loop;
mod outgoing;
mod window;

use adw::{prelude::*, Application};
use consts::ID;
use window::Window;
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
