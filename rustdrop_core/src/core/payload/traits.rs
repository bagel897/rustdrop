use prost::Message;

use super::id::get_unique;

pub trait IncomingMeta: From<Self::ProtoType> {
    type ProtoType: Message;
    fn into_proto_type_with_id(self, payload_id: i64, id: i64) -> Self::ProtoType;
    fn into_proto_type(self, payload_id: i64) -> Self::ProtoType {
        self.into_proto_type_with_id(payload_id, get_unique())
    }
    fn describe(&self, quantity: usize) -> String;
}
