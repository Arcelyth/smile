use std::sync::Arc;
use std::time::Duration;
use ratatui::style::Color;

#[derive(Debug)]
pub struct Popup {
    pub id: usize,
    pub content: Arc<str>,
    pub duration: Duration,
    pub position: (usize, usize),
    pub size: (usize, usize),
    pub color: Color,
}
