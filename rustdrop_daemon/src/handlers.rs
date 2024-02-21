use arboard::Clipboard;
use ashpd::desktop::notification::{Button, Notification, NotificationProxy, Priority};
use futures::StreamExt;
use opener::{open, open_browser};
use rustdrop::{IncomingText, PairingRequest, ReceiveEvent};

use crate::consts::ID;
async fn handle_pairing_request(request: PairingRequest) {
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
    let action = match action.name() {
        "accept" => true,
        "reject" => false,
        _ => todo!(),
    };
    request.respond(action)
}
async fn handle_url(text: IncomingText) {
    open_browser(text.text).unwrap()
}
async fn handle_text(text: IncomingText) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(text.text).unwrap();
}
async fn handle_phone(text: IncomingText) {
    open(format!("tel:{}", text.text)).unwrap()
}
pub async fn handle_event(event: ReceiveEvent) {
    match event {
        ReceiveEvent::Text(text) => match text.text_type {
            rustdrop::TextType::Text => handle_text(text).await,
            rustdrop::TextType::Url => handle_url(text).await,
            rustdrop::TextType::PhoneNumber => handle_phone(text).await,
            _ => todo!(),
        },
        ReceiveEvent::Wifi(_) => todo!(),
        ReceiveEvent::PairingRequest(request) => handle_pairing_request(request).await,
    }
}
