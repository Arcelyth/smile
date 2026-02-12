use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("Invalid position.")]
    InvalidPosition,
    #[error("Invalid buffer id.")]
    InvalidId,
    #[error("Not a file.")]
    NotAFile,
    #[error("Invalid path.")]
    InvalidPath,
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}


#[derive(Error, Debug)]
pub enum LayoutError {
    #[error("Layout ID not found.")]
    IdNotFound,
    #[error("Pane not found.")]
    PaneNotFound,
    #[error("No node in the layout.")]
    NoNode,
    #[error("Not pane.")]
    NotPane,
    #[error("Buffer error: {0}")]
    BufferErr(#[from] BufferError),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}


#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Buffer error: {0}")]
    BufferErr(#[from] BufferError),
    #[error("Layout error: {0}")]
    LayoutErr(#[from] LayoutError),
    #[error("Render layout error")]
    RenderLayoutError,
    #[error("Rect not found")]
    RectNotFound,
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}


