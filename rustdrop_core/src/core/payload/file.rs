use std::{fs, os::unix::fs::MetadataExt, path::PathBuf};

use super::traits::IncomingMeta;
use crate::protobuf::sharing::nearby::{file_metadata::Type, FileMetadata};

#[derive(Debug, Clone)]
pub struct IncomingFile {
    pub name: String, // Absolute path for sending
    pub mime_type: String,
    pub size: i64,
    pub file_type: Type,
}
impl IncomingMeta for IncomingFile {
    type ProtoType = FileMetadata;
    fn into_proto_type_with_id(self, payload_id: i64, id: i64) -> Self::ProtoType {
        FileMetadata {
            payload_id: Some(payload_id),
            size: Some(self.size),
            name: Some(self.name),
            r#type: Some(self.file_type.into()),
            mime_type: Some(self.mime_type),
            id: Some(id),
        }
    }
    fn describe(&self, quantity: usize) -> String {
        if quantity > 1 {
            format!("{} files", quantity)
        } else {
            "a file".into()
        }
    }
}
impl From<PathBuf> for IncomingFile {
    fn from(path: PathBuf) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap().into();
        let metadata = fs::metadata(&path).unwrap();
        let size = metadata.size().try_into().unwrap();
        let mime_type = infer::get_from_path(path).unwrap().unwrap();
        let file_type = match mime_type.matcher_type() {
            infer::MatcherType::App => Type::App,
            infer::MatcherType::Audio => Type::Audio,
            infer::MatcherType::Image => Type::Image,
            infer::MatcherType::Video => Type::Video,
            _ => Type::Unknown,
        };
        Self {
            name,
            size,
            mime_type: mime_type.mime_type().into(),
            file_type,
        }
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
