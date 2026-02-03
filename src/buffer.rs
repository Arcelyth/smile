#![allow(dead_code)]

use std::sync::Arc;
use crate::error::*;


pub struct Buffer {
    pub content: Vec<String>,
    pub name: Arc<str>,
}

impl Buffer {
    pub fn new(name: &str) -> Self {
        Self {
            content: vec![String::new()],
            name: Arc::from(name), 
        }
    }
    pub fn from_content(content: &str, name: & str) -> Self {
        let c: Vec<String> = content.split('\n').map(String::from).collect();
        Self {
            content: c,
            name: Arc::from(name),
        }
    } 
    pub fn change_name(&mut self, name: &str) {
        self.name = Arc::from(name)
    }

    pub fn get_line_count(&self) -> usize {
        self.content.len()        
    }

    // x, y start from 1
    pub fn change_content_at(&mut self, x: usize, y: usize, len: usize, replace_str: &str) -> Result<(), BufferError> {
        if self.get_line_count() < y {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        self.content[y-1].replace_range(x-1..x+len, replace_str);
        Ok(())
    }

    pub fn delete_content_at(&mut self, x: usize, y: usize, len: usize) -> Result<(), BufferError> {
         if self.get_line_count() < y {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        self.content[y-1].drain(x-1..x+len);
        Ok(())
    }

    pub fn add_content_at(&mut self, x: usize, y: usize, add_str: &str) -> Result<(), BufferError> {
         if self.get_line_count() < y {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        Ok(self.content[y-1].insert_str(x-1, add_str))
    }

    pub fn delete_line(&mut self, y: usize) -> Result<String, BufferError> {
        if self.get_line_count() < y {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        Ok(self.content.remove(y - 1)) 
    }

    pub fn add_new_line(&mut self, y: usize) -> Result<(), BufferError> {
        if self.get_line_count() < y {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        Ok(self.content.insert(y - 1, String::new()))   
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

    pub fn add_buffer(&mut self, name: &str) {
        let new_buffer = Buffer::new(name);
        self.buffers.push(new_buffer);
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
