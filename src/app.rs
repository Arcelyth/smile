#![allow(dead_code)]

use crate::buffer::BufferManager;
use crate::error::BufferError;

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
    pub cursor_pos: (usize, usize),
}

impl App {
    pub fn new() -> Self {
        Self {
            buf_manager: BufferManager::new(),
            current_screen: Screen::Welcome,
            current_mod: Mod::Input,
            cursor_pos: (0, 0),
        }
    }
    
    pub fn mv_cursor_right(&mut self) {
        if let Some(buf) = self.buf_manager.get_current_buffer() {
            let cur_pos = &mut self.cursor_pos;
            let line_len = buf.content[cur_pos.1].len();

            if cur_pos.0 < line_len {
                cur_pos.0 += 1;
            } else if cur_pos.1 < buf.content.len() - 1 {
                cur_pos.1 += 1;
                cur_pos.0 = 0;
            }
        }
    }

    pub fn mv_cursor_left(&mut self) {
        let cur_pos = &mut self.cursor_pos;
        if cur_pos.0 > 0 {
            cur_pos.0 -= 1;
        } else if cur_pos.1 > 0 {
            if let Some(buf) = self.buf_manager.get_current_buffer() {
                cur_pos.1 -= 1;
                cur_pos.0 = buf.content[cur_pos.1].len();
            }
        }
    }

    pub fn mv_cursor_up(&mut self) {
        if self.cursor_pos.1 > 0 {
            self.cursor_pos.1 -= 1;
            if let Some(buf) = self.buf_manager.get_current_buffer() {
                self.cursor_pos.0 = self.cursor_pos.0.min(buf.content[self.cursor_pos.1].len());
            }
        }
    }

    pub fn mv_cursor_down(&mut self) {
        if let Some(buf) = self.buf_manager.get_current_buffer_mut() {
            self.cursor_pos.1 += 1;
            if self.cursor_pos.1 >= buf.content.len() {
                buf.content.push(String::new());
                self.cursor_pos.0 = 0;
            } else {
                self.cursor_pos.0 = self.cursor_pos.0.min(buf.content[self.cursor_pos.1].len());
            }
        }
    }

    pub fn insert_char(&mut self, ch: char) -> Result<(), BufferError> {
        if let Some(buf) = self.buf_manager.get_current_buffer_mut() {
            let cur_pos = self.cursor_pos;
            let c = format!("{}", ch);
            buf.add_content_at(cur_pos.0 + 1, cur_pos.1 + 1, c.as_str())?;
        }
        Ok(())
    }

    pub fn handle_backspace(&mut self) {
        let (x, y) = (self.cursor_pos.0, self.cursor_pos.1);
        
        if x > 0 {
            if let Some(buf) = self.buf_manager.get_current_buffer_mut() {
                buf.content[y].remove(x - 1);
                self.cursor_pos.0 -= 1;
            }
        } else if y > 0 {
            if let Some(buf) = self.buf_manager.get_current_buffer_mut() {
                let current_line = buf.content.remove(y);
                let prev_line = &mut buf.content[y - 1];
                let prev_len = prev_line.len();
                
                prev_line.push_str(&current_line);
                
                self.cursor_pos.1 -= 1;
                self.cursor_pos.0 = prev_len;
            }
        }
    }

    pub fn handle_enter(&mut self) {
        if let Some(buf) = self.buf_manager.get_current_buffer_mut() {
            let (x, y) = (self.cursor_pos.0, self.cursor_pos.1);
            let current_line = &mut buf.content[y];
            
            let next_line_content = current_line.split_off(x);
            
            buf.content.insert(y + 1, next_line_content);
            
            self.cursor_pos.1 += 1;
            self.cursor_pos.0 = 0;
        }
    }
}
