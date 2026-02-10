use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum EditOp {
    Insert {
        pos: (usize, usize),
        text: Arc<str>,
        len: usize,
    },
    Delete {
        pos: (usize, usize),
        text: Arc<str>,
        len: usize,
    },
    InsertLine {
        y: usize, 
        text: Arc<str>
    },
    DeleteLine {
        y: usize, 
        text: Arc<str>
    }

}

impl EditOp {
    pub fn inverse(&self) -> Self {
        match self {
            Self::Insert { pos, text, len } => Self::Delete {
                pos: *pos,
                text: text.clone(),
                len: *len
            },
            Self::Delete { pos, text, len } => Self::Insert {
                pos: *pos,
                text: text.clone(),
                len: *len
            },
            Self::InsertLine{ y, text } => Self::DeleteLine{
                y: *y,
                text: text.clone(),
            },

            Self::DeleteLine { y, text } => Self::InsertLine {
                y: *y,
                text: text.clone(),
            },

        }
    }
}
