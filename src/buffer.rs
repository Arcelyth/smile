#![allow(dead_code)]

use std::sync::Arc;
use crate::error::*;
use std::path::{Path, PathBuf};
use std::fs::{File, read_to_string};
use std::io::{self, Write};

pub struct Buffer {
    pub content: Vec<String>,
    pub name: Arc<str>,
    pub path: Option<PathBuf>,
    pub cursor_pos: (usize, usize),
    pub scroll_offset: (usize, usize),
    pub scroll_thres: (usize, usize),
}

impl Buffer {
    pub fn new(name: &str) -> Self {
        Self {
            content: vec![String::new()],
            name: Arc::from(name), 
            path: None,
            cursor_pos: (0, 0),
            scroll_offset: (0, 0),
            scroll_thres: (0, 0),
        }
    }
    pub fn from_content(content: &str, name: & str) -> Self {
        let c: Vec<String> = content.split('\n').map(String::from).collect();
        Self {
            content: c,
            name: Arc::from(name),
            path: None,
            cursor_pos: (0, 0),
            scroll_offset: (0, 0),
            scroll_thres: (0, 0),
        }
    } 
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let path_ref = path.as_ref();
        let content_str = read_to_string(path_ref)?;
        
        let mut content: Vec<String> = content_str
            .lines()
            .map(|s| s.to_string())
            .collect();
        
        if content.is_empty() {
            content.push(String::new());
        }

        let name = path_ref
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        Ok(Self {
            content,
            name: Arc::from(name),
            path: Some(path_ref.to_path_buf()),
            cursor_pos: (0, 0),
            scroll_offset: (0, 0),
            scroll_thres: (0, 0),
        })
    }
    pub fn change_name(&mut self, name: &str) {
        self.name = Arc::from(name)
    }

    pub fn get_line_count(&self) -> usize {
        self.content.len()        
    }

    pub fn change_content_at(&mut self, len: usize, replace_str: &str) -> Result<(), BufferError> {
        let (x, y) = self.cursor_pos;
        if self.get_line_count() <= y {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        self.content[y].replace_range(x..x+len, replace_str);
        Ok(())
    }

    pub fn delete_content_at(&mut self, len: usize) -> Result<(), BufferError> {
        let (x, y) = self.cursor_pos;
        if self.get_line_count() <= y {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        self.content[y].drain(x..x+len);
        Ok(())
    }

    pub fn add_content_at(&mut self, add_str: &str) -> Result<(), BufferError> {
        let (x, y) = self.cursor_pos;
         if self.get_line_count() <= y {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        Ok(self.content[y].insert_str(x, add_str))
    }

    pub fn delete_line(&mut self) -> Result<String, BufferError> {
        let y = self.cursor_pos.1;
        if self.get_line_count() <= y {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        Ok(self.content.remove(y)) 
    }

    pub fn add_new_line(&mut self) -> Result<(), BufferError> {
        let y = self.cursor_pos.1;
        if self.get_line_count() <= y {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        Ok(self.content.insert(y, String::new()))   
    }

    pub fn save(&mut self) -> io::Result<()> {
        if let Some(ref path) = self.path {
            self.save_to(path.clone())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "No file path bound to this buffer"))
        }
    }

    pub fn save_to<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let path_ref = path.as_ref();
        let mut file = File::create(path_ref)?; 
        
        for (i, line) in self.content.iter().enumerate() {
            file.write_all(line.as_bytes())?;
            if i < self.content.len() - 1 {
                file.write_all(b"\n")?;
            }
        }

        if self.path.is_none() {
            self.path = Some(path_ref.to_path_buf());
            if let Some(name) = path_ref.file_name().and_then(|n| n.to_str()) {
                self.name = Arc::from(name);
            }
        }
        Ok(())
    }

    pub fn mv_cursor_right(&mut self) {
        let cur_pos = &mut self.cursor_pos;
        let line_len = self.content[cur_pos.1].len();

        if cur_pos.0 < line_len {
            cur_pos.0 += 1;
        } else if cur_pos.1 < self.content.len() - 1 {
            cur_pos.1 += 1;
            cur_pos.0 = 0;
        }
    }

    pub fn mv_cursor_left(&mut self) {
        let cur_pos = &mut self.cursor_pos;
        if cur_pos.0 > 0 {
            cur_pos.0 -= 1;
        } else if cur_pos.1 > 0 {
            cur_pos.1 -= 1;
            cur_pos.0 = self.content[cur_pos.1].len();
        }
    }

    pub fn mv_cursor_up(&mut self) {
        if self.cursor_pos.1 > 0 {
            self.cursor_pos.1 -= 1;
            self.cursor_pos.0 = self.cursor_pos.0.min(self.content[self.cursor_pos.1].len());
        }
    }

    pub fn mv_cursor_down(&mut self) {
        self.cursor_pos.1 += 1;
        if self.cursor_pos.1 >= self.content.len() {
            self.content.push(String::new());
            self.cursor_pos.0 = 0;
        } else {
            self.cursor_pos.0 = self.cursor_pos.0.min(self.content[self.cursor_pos.1].len());
        }
    }

     pub fn handle_backspace(&mut self) {
        let (x, y) = (self.cursor_pos.0, self.cursor_pos.1);
        
        if x > 0 {
            self.content[y].remove(x - 1);
            self.cursor_pos.0 -= 1;
        } else if y > 0 {
            let current_line = self.content.remove(y);
            let prev_line = &mut self.content[y - 1];
            let prev_len = prev_line.len();
            
            prev_line.push_str(&current_line);
            
            self.cursor_pos.1 -= 1;
            self.cursor_pos.0 = prev_len;
        }
    }

    pub fn handle_enter(&mut self) {
        let (x, y) = (self.cursor_pos.0, self.cursor_pos.1);
        let current_line = &mut self.content[y];
        
        let next_line_content = current_line.split_off(x);
        
        self.content.insert(y + 1, next_line_content);
        
        self.cursor_pos.1 += 1;
        self.cursor_pos.0 = 0;
    }

    pub fn update_scroll(&mut self, viewport_height: usize, viewport_width: usize) {
        let thres = self.scroll_thres;
        let total_lines = self.content.len();
        let (x, y) = &mut self.cursor_pos;
        let width = self.content[*y].len();
        if *y >= total_lines {
            *y = total_lines.saturating_sub(1);
        }

        if *y >= (self.scroll_offset.1 + viewport_height).saturating_sub(thres.1) {
            self.scroll_offset.1 = *y + thres.1 - viewport_height + 1;
        }

        if *y < self.scroll_offset.1 {
            self.scroll_offset.1 = *y;
        }

        if *x >= width + 1 {
            *x = width.saturating_sub(1);
        }

        if *x >= (self.scroll_offset.0 + viewport_width).saturating_sub(thres.0) {
            self.scroll_offset.0 = *x + thres.0 - viewport_width + 1;
        }
        if *x < self.scroll_offset.0 {
            self.scroll_offset.0 = *x;
        }
    }


}

pub struct BufferManager {
    pub buffers: Vec<Buffer>,
    pub current_buffer: Option<usize>,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            buffers: vec![],
            current_buffer: None,
        }
    }

    pub fn add_new_buffer(&mut self, name: &str) {
        let new_buffer = Buffer::new(name);
        self.buffers.push(new_buffer);
        self.current_buffer = Some(self.buffers.len() - 1);
    }

    pub fn add_buffer(&mut self, buf: Buffer) {
        self.buffers.push(buf);
        self.current_buffer = Some(self.buffers.len() - 1);
    }

    pub fn get_current_buffer(&self) -> Option<&Buffer> {
        if let Some(cur) = self.current_buffer {
            if cur >= self.buffers.len() {
                return None;
            }
            return Some(&self.buffers[cur]);
        }
        None
    }
    
    pub fn get_current_buffer_mut(&mut self) -> Option<&mut Buffer> {
        if let Some(cur) = self.current_buffer {
            if cur >= self.buffers.len() {
                return None;
            }
            return Some(&mut self.buffers[cur]);
        }
        None
    }

    pub fn change_to_prev(&mut self) {
        if let Some(cur) = self.current_buffer {
            let len = self.buffers.len();
            if len == 0 || len == 1 { return; }
            if cur == 0 { self.current_buffer = Some(len - 1); }
            self.current_buffer = Some(cur - 1);
        }
    }

    pub fn change_to_next(&mut self) {
        if let Some(cur) = self.current_buffer {
            let len = self.buffers.len();
            if len == 0 || len == 1 { return; }
            if cur == len - 1 { self.current_buffer = Some(0); }
            self.current_buffer = Some(cur + 1);
        }
    }
}
