use std::{fs, io::Result};
fn main() -> Result<()> {
    let files: Vec<String> = fs::read_dir("./src/ProtobufSource/")
        .unwrap()
        .map(|f| f.unwrap().path().to_str().unwrap().to_string())
        .collect();
    prost_build::compile_protos(&files, &["./src/ProtobufSource/"])?;
    Ok(())
}
