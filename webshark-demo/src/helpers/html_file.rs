use std::path::PathBuf;
use std::io::{Error};

pub fn send_http_file(file_path: String) -> Result<Vec<u8>, Error> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources");
    path.push(file_path);

    let file_content = std::fs::read(path)?;

    Ok(file_content)
}