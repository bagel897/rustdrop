use crate::protobuf::sharing::nearby::FileMetadata;

use crate::protobuf::sharing::nearby::file_metadata;

use super::traits::IncomingMeta;

#[derive(Debug, Clone)]
pub struct IncomingFile {
    pub name: String, // Absolute path for sending
    pub mime_type: String,
    pub size: i64,
    pub file_type: file_metadata::Type,
}
impl IncomingMeta for IncomingFile {
    type ProtoType = FileMetadata;
    fn into_proto_type(self, payload_id: i64, id: i64) -> Self::ProtoType {
        FileMetadata {
            payload_id: Some(payload_id),
            size: Some(self.size),
            name: Some(self.name),
            r#type: Some(self.file_type.into()),
            mime_type: Some(self.mime_type.into()),
            id: Some(id),
        }
    }
    fn describe(&self, quantity: usize) -> String {
        "a file".into()
    }
}
impl From<FileMetadata> for IncomingFile {
    fn from(file: FileMetadata) -> Self {
        Self {
            name: file.name().to_string(),
            mime_type: file.mime_type().into(),
            size: file.size(),
            file_type: file.r#type(),
        }
    }
}
