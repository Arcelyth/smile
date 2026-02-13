use crate::command::Arc;

pub enum Instruction {
    InsertText(Arc<str>),
    DeleteText(usize),
    InsertLine,
    DeleteLine,
    DeleteBlock((usize, usize)),
    InsertBlock(Vec<Arc<str>>),
}

