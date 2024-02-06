use crate::consts::ID;
use ashpd::desktop::{
    clipboard::Clipboard,
    notification::{Button, Notification, NotificationProxy, Priority},
    Session,
};
use opener::open;
use opener::open_browser;
use rustdrop::{Device, IncomingText, PairingRequest, UiHandle};
use tokio_stream::StreamExt;
#[derive(Debug, Default)]
pub struct DaemonUI {}
impl UiHandle for DaemonUI {
    async fn handle_text(&mut self, text: IncomingText) {
        todo!()
        // let clipboard = Clipboard::new().await.unwrap();
        // clipboard.set_selection(, mime_types)
    }
    async fn handle_url(&mut self, text: IncomingText) {
        open_browser(text.text).unwrap()
    }
    async fn handle_phone(&mut self, text: IncomingText) {
        open(format!("tel:{}", text.text)).unwrap()
    }

    async fn handle_pairing_request(&mut self, request: &PairingRequest) -> bool {
        let proxy = NotificationProxy::new().await.unwrap();
        let notif = Notification::new(&request.name())
            .default_action("accept")
            .body(Some(&*request.body()))
            .priority(Priority::High)
            .button(Button::new("Accept", "accept"))
            .button(Button::new("Reject", "reject"));
        proxy.add_notification(ID, notif).await.unwrap();
        let action = proxy
            .receive_action_invoked()
            .await
            .unwrap()
            .next()
            .await
            .expect("Stream exhausted");
        proxy.remove_notification(ID).await.unwrap();
        match action.name() {
            "accept" => true,
            "reject" => false,
            _ => todo!(),
        }
    }
    fn pick_dest<'a>(&mut self, devices: &'a [Device]) -> Option<&'a Device> {
        todo!();
    }
}
