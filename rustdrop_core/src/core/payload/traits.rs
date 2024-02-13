use prost::Message;

pub trait IncomingMeta: From<Self::ProtoType> {
    type ProtoType: Message;
    fn into_proto_type(self, payload_id: i64, id: i64) -> Self::ProtoType;
    fn describe(&self, quantity: usize) -> String;
}
