#![allow(dead_code)]

#[derive(Debug)]
pub enum BufferError {
    InvalidPosition,
    InvalidId,
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

#[derive(Debug)]
pub enum LayoutError {
    IdNotFound,
    PaneNotFound,
    NoNode,
    NotPane,
    BufferErr(BufferError),
    IOError(std::io::Error),
}

impl From<BufferError> for LayoutError {
    fn from(err: BufferError) -> Self {
        Self::BufferErr(err)
    }
}

impl From<std::io::Error> for LayoutError{
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            _ => LayoutError::IOError(err),
        }
    }
}
#[derive(Debug)]
pub enum RenderError {
    BufferErr(BufferError),
    LayoutErr(LayoutError)
}

impl From<BufferError> for RenderError {
    fn from(err: BufferError) -> Self {
        Self::BufferErr(err)
    }
}

impl From<LayoutError> for RenderError {
    fn from(err: LayoutError) -> Self {
        Self::LayoutErr(err)
    }
}

