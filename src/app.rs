use crate::buffer::{Buffer, BufferManager};

#[derive(Debug)]
pub enum Screen {
    Welcome,
    Editor,
    Popup, 
}

#[derive(Debug)]
pub enum Mod {
    Input,
}

pub struct App {
    pub buf_manager: BufferManager,
    pub current_screen: Screen,
    pub current_mod: Mod,
}

impl App {
    pub fn new() -> Self {
        Self {
            buf_manager: BufferManager::new(),
            current_screen: Screen::Editor,
            current_mod: Mod::Input,
        }
    }
}
