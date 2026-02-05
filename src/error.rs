#![allow(dead_code)]

#[derive(Debug)]
pub enum BufferError {
    InvalidPosition,
    NotAFile,
    InvalidPath,
    IOError(std::io::Error),
}

impl From<std::io::Error> for BufferError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            _ => BufferError::IOError(err),
        }
    }
}

