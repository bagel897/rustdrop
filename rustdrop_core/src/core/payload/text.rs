use super::traits::IncomingMeta;
use crate::protobuf::sharing::nearby::{text_metadata, TextMetadata};

#[derive(Debug, Clone)]
pub struct IncomingText {
    pub name: String,
    pub text_type: text_metadata::Type,
    pub size: i64,
    pub text: String,
}
impl From<TextMetadata> for IncomingText {
    fn from(text: TextMetadata) -> Self {
        Self {
            name: text.text_title().into(),
            text_type: text.r#type(),
            size: text.size(),
            text: String::new(),
        }
    }
}
impl IncomingMeta for IncomingText {
    type ProtoType = TextMetadata;
    fn into_proto_type_with_id(self, payload_id: i64, id: i64) -> Self::ProtoType {
        TextMetadata {
            text_title: Some(self.name),
            r#type: Some(self.text_type.into()),
            payload_id: Some(payload_id),
            size: Some(self.size),
            id: Some(id),
        }
    }
    fn describe(&self, quantity: usize) -> String {
        if quantity > 1 {
            format!(
                "{} {}",
                quantity,
                match self.text_type {
                    crate::TextType::Unknown => todo!(),
                    crate::TextType::Text => "texts",
                    crate::TextType::Url => "links",
                    crate::TextType::Address => "addresses",
                    crate::TextType::PhoneNumber => "phone numberes",
                }
            )
        } else {
            match self.text_type {
                crate::TextType::Unknown => todo!(),
                crate::TextType::Text => "some text",
                crate::TextType::Url => "a link",
                crate::TextType::Address => "an address",
                crate::TextType::PhoneNumber => "a phone number",
            }
            .into()
        }
    }
}
