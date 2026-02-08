#![allow(dead_code)]

use crate::buffer::BufferManager;
use crate::command::*;
use crate::error::BufferError;
use crate::error::*;
use crate::layout::layout_manager::*;

#[derive(Debug)]
pub enum Screen {
    Welcome,
    Editor,
    Command,
}

#[derive(Debug)]
pub enum Mod {
    Input,
    Commanding,
}

pub struct App {
    pub buf_manager: BufferManager,
    pub layout_manager: LayoutManager,
    pub command: KaoCo,
    pub current_screen: Screen,
    pub current_mod: Mod,
    pub cursor_pos: (usize, usize),
    pub scroll_offset: (usize, usize),
    pub scroll_threshold: (usize, usize),
}

impl App {
    pub fn new() -> Self {
        Self {
            buf_manager: BufferManager::new(),
            layout_manager: LayoutManager::new(),
            command: KaoCo::new(),
            current_screen: Screen::Welcome,
            current_mod: Mod::Input,
            scroll_offset: (0, 0),
            scroll_threshold: (0, 0),
            cursor_pos: (0, 0),
        }
    }

    pub fn init(&mut self, path: Option<String>) -> Result<(), LayoutError> {
        let id = if let Some(p) = path {
            match self.buf_manager.add_new_buffer_from_path(p) {
                Err(e) => {
                    match e {
                        BufferError::NotAFile => {
                            println!("The input is not a file.");
                        }
                        _ => println!("Unknown Error"),
                    }
                    return Ok(());
                }
                Ok(b) => b,
            }
        } else {
            self.buf_manager.add_new_buffer("Untitled")
        };

        self.layout_manager.init(id);
        self.current_screen = Screen::Editor;
        Ok(())
    }

    pub fn from(screen: Screen, buf_manager: BufferManager) -> Self {
        Self {
            buf_manager,
            layout_manager: LayoutManager::new(),
            command: KaoCo::new(),
            current_screen: screen,
            current_mod: Mod::Input,
            scroll_offset: (0, 0),
            scroll_threshold: (0, 0),
            cursor_pos: (0, 0),
        }
    }
}
