use crate::consts::ID;
use ashpd::desktop::notification::{Button, Notification, NotificationProxy, Priority};
use rustdrop::{Device, PairingRequest, UiHandle};
use tokio_stream::StreamExt;
use tracing::error;
#[derive(Debug, Default)]
pub struct DaemonUI {}
impl UiHandle for DaemonUI {
    fn handle_error(&mut self, t: String) {
        error!(t);
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
