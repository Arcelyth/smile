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
}

impl Buffer {
    pub fn new(name: &str) -> Self {
        Self {
            content: vec![String::new()],
            name: Arc::from(name), 
            path: None,
        }
    }
    pub fn from_content(content: &str, name: & str) -> Self {
        let c: Vec<String> = content.split('\n').map(String::from).collect();
        Self {
            content: c,
            name: Arc::from(name),
            path: None,
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
        })
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
