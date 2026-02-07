#![allow(dead_code)]

use crate::buffer::BufferManager;
use crate::error::BufferError;
use crate::command::*;
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

