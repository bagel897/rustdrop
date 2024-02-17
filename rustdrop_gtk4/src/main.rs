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
    application.connect_activate(build_ui);
    application.run();
}
fn build_ui(app: &Application) {
    // Create new window and present it
    let window = Window::new(app);
    window.present();
}
