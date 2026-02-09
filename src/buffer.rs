#![allow(dead_code)]
use crate::error::*;
use crate::utils::*;
use std::collections::HashMap;
use color_eyre::Result;
use std::fs::{self, File, read_to_string};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub struct Buffer {
    pub id: usize,
    pub content: Vec<String>,
    pub history: Vec<Vec<Arc<str>>>,
    pub history_ptr: usize,
    pub name: Arc<str>,
    pub path: Option<PathBuf>,
    pub file_info: Option<FileInfo>,
    pub saved: bool,
}

impl Buffer {
    pub fn new(name: &str, id: usize) -> Self {
        Self {
            id,
            content: vec![String::new()],
            history: vec![vec![String::new().into()]],
            history_ptr: 0,
            name: Arc::from(name),
            path: None,
            file_info: Some(FileInfo::new()),
            saved: true,
        }
    }
    pub fn from_content(content: &str, name: &str, id: usize) -> Self {
        let c: Vec<String> = content.split('\n').map(String::from).collect();
        let v: Vec<Arc<str>> = c.clone().into_iter().map(|s| Arc::<str>::from(s)).collect();
        Self {
            id,
            content: c,
            history: vec![v],
            history_ptr: 0,
            name: Arc::from(name),
            path: None,
            file_info: Some(FileInfo::new()),
            saved: true,
        }
    }
    pub fn from_file<P: AsRef<Path>>(path: P, id: usize) -> Result<Self, BufferError> {
        let path_ref = path.as_ref();
        if !path_ref.is_file() {
            return Err(BufferError::NotAFile);
        }
        let content_str = read_to_string(path_ref)?;

        let mut content: Vec<String> = content_str.lines().map(|s| s.to_string()).collect();
        let v: Vec<Arc<str>> = content
            .clone()
            .into_iter()
            .map(|s| Arc::<str>::from(s))
            .collect();

        if content.is_empty() {
            content.push(String::new());
        }

        let name = path_ref
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        let mut s = Self {
            id,
            content: content.clone(),
            history: vec![v],
            history_ptr: 0,
            name: Arc::from(name),
            path: Some(path_ref.to_path_buf()),
            file_info: None,
            saved: true,
        };
        s.refresh_file_info().unwrap();
        Ok(s)
    }
    pub fn change_name(&mut self, name: &str) {
        self.name = Arc::from(name);
        self.path = Some(format!("./{}", name).into());
    }

    pub fn get_line_count(&self) -> usize {
        self.content.len()
    }

    pub fn delete_content_at(&mut self, len_in_chars: usize, cursor_pos: (usize, usize)) -> Result<(), BufferError> {
        let (x, y) = cursor_pos;
        if y >= self.content.len() {
            return Err(BufferError::InvalidPosition);
        }

        let line = &mut self.content[y];
        let start_byte = char_to_byte_idx(line, x);
        let end_byte = char_to_byte_idx(line, x + len_in_chars);

        line.drain(start_byte..end_byte);
        self.handle_change();
        Ok(())
    }

    pub fn change_content_at(
        &mut self,
        len_in_chars: usize,
        replace_str: &str,
        cursor_pos: (usize, usize)
    ) -> Result<(), BufferError> {
        let (x, y) = cursor_pos;
        if y >= self.content.len() {
            return Err(BufferError::InvalidPosition);
        }

        let line = &mut self.content[y];
        let start_byte = char_to_byte_idx(line, x);
        let end_byte = char_to_byte_idx(line, x + len_in_chars);

        line.replace_range(start_byte..end_byte, replace_str);
        self.handle_change();
        Ok(())
    }

    pub fn add_content_at(&mut self, add_str: &str, cursor_pos: (usize, usize)) -> Result<(), BufferError> {
        let (x, y) = cursor_pos;
        if y >= self.content.len() {
            return Err(BufferError::InvalidPosition);
        }

        let byte_idx = char_to_byte_idx(&self.content[y], x);
        self.content[y].insert_str(byte_idx, add_str);
        self.handle_change();
        Ok(())
    }

    pub fn delete_line(&mut self, cursor_pos: (usize, usize)) -> Result<String, BufferError> {
        let y = cursor_pos.1;
        if self.get_line_count() <= y {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        self.handle_change();
        Ok(self.content.remove(y))
    }

    pub fn add_new_line(&mut self, cursor_pos: (usize, usize)) -> Result<(), BufferError> {
        let y = cursor_pos.1;
        if self.get_line_count() <= y {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        self.handle_change();
        Ok(self.content.insert(y, String::new()))
    }

    pub fn save(&mut self) -> io::Result<()> {
        if let Some(ref path) = self.path {
            self.save_to(path.clone())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No file path bound to this buffer",
            ))
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
        if let Err(e) = self.refresh_file_info() {
            println!("error: {:?}", e);
        };
        self.saved = true;
        Ok(())
    }

    // change saved to false and add new content to history
    pub fn handle_change(&mut self) {
        self.saved = false;
        self.history.truncate(self.history_ptr + 1);
        let v: Vec<Arc<str>> = self
            .content
            .clone()
            .into_iter()
            .map(|s| Arc::<str>::from(s))
            .collect();
        self.history.push(v);
        self.history_ptr += 1;
    }

    pub fn get_visual_width_upto(&self, line_idx: usize, char_idx: usize) -> usize {
        let line = &self.content[line_idx];
        line.graphemes(true).take(char_idx).map(|g| g.width()).sum()
    }

    pub fn get_line_visual_width(&self, line_idx: usize) -> usize {
        self.content[line_idx].width()
    }

    pub fn refresh_file_info(&mut self) -> Result<()> {
        let path_str = match &self.path {
            Some(p) => match p.to_str() {
                Some(s) => s,
                None => return Ok(()),
            },
            None => return Ok(()),
        };

        let metadata = fs::metadata(path_str)?;

        let info = FileInfo {
            size: metadata.len(),
            read_only: metadata.permissions().readonly(),
            format: detect_line_ending(&self.content.join("")),
        };

        self.file_info = Some(info);
        Ok(())
    }

    pub fn revoke(&mut self) {
        self.history_ptr = if self.history_ptr <= 0 {
            0
        } else {
            self.history_ptr - 1
        };
        self.content = arc_vec_to_string(self.history[self.history_ptr].clone());
    }

}

pub struct BufferManager {
    pub id_counter: usize,
    pub buffers: HashMap<usize, Buffer>,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            id_counter: 1,
            buffers: HashMap::new(),
        }
    }

    pub fn add_new_buffer(&mut self, name: &str) -> usize {
        let old_id = self.id_counter;
        let new_buffer = Buffer::new(name, old_id);
        self.buffers.insert(old_id, new_buffer);
        self.id_counter += 1;
        old_id
    }

    pub fn add_new_buffer_from_path<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, BufferError> {
        let old_id = self.id_counter;
        let new_buffer = Buffer::from_file(path, old_id)?;
        self.buffers.insert(old_id, new_buffer);
        self.id_counter += 1;
        Ok(old_id)
    } 

    pub fn get_buffer(&self, id: usize) -> Result<&Buffer, BufferError> {
        self.buffers
            .get(&id)
            .ok_or(BufferError::InvalidId)
    }

    pub fn get_buffer_mut(&mut self, id: usize) -> Result<&mut Buffer, BufferError> {
        self.buffers
            .get_mut(&id)
            .ok_or(BufferError::InvalidId)
    }
}
