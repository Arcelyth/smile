use ratatui::crossterm::cursor::{SetCursorStyle};

#[derive(Debug, Clone)]
pub struct Cursor {
    pub style: SetCursorStyle,
    pub pos: (usize, usize),
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            style: SetCursorStyle::DefaultUserShape,
            pos: (0, 0)
        }
    }
}
