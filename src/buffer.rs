#![allow(dead_code)]
use unicode_width::UnicodeWidthStr;
use unicode_segmentation::UnicodeSegmentation;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs::{self, File, read_to_string};
use std::io::{self, Write};
use color_eyre::Result;
use crate::utils::*;
use crate::error::*;

pub struct Buffer {
    pub content: Vec<String>,
    pub history: Vec<Vec<Arc<str>>>,
    pub history_ptr: usize, 
    pub name: Arc<str>,
    pub path: Option<PathBuf>,
    pub cursor_pos: (usize, usize),
    pub scroll_offset: (usize, usize),
    pub scroll_thres: (usize, usize),
    pub file_info: Option<FileInfo>,
    pub saved: bool,
}

impl Buffer {
    pub fn new(name: &str) -> Self {
        Self {
            content: vec![String::new()],
            history: vec![vec![String::new().into()]],
            history_ptr: 0,
            name: Arc::from(name), 
            path: None,
            cursor_pos: (0, 0),
            scroll_offset: (0, 0),
            scroll_thres: (0, 0),
            file_info: Some(FileInfo::new()),
            saved: true,
        }
    }
    pub fn from_content(content: &str, name: & str) -> Self {
        let c: Vec<String> = content.split('\n').map(String::from).collect();
        let v: Vec<Arc<str>> = c.clone()
            .into_iter()
            .map(|s| Arc::<str>::from(s))
            .collect();
        Self {
            content: c,
            history: vec![v],
            history_ptr: 0,
            name: Arc::from(name),
            path: None,
            cursor_pos: (0, 0),
            scroll_offset: (0, 0),
            scroll_thres: (0, 0),
            file_info: Some(FileInfo::new()),
            saved: true,
        }
    } 
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, BufferError> {
        let path_ref = path.as_ref();
        if !path_ref.is_file() {
            return Err(BufferError::NotAFile);
        }
        let content_str = read_to_string(path_ref)?;

        let mut content: Vec<String> = content_str
            .lines()
            .map(|s| s.to_string())
            .collect();
         let v: Vec<Arc<str>> = content.clone()
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
            content: content.clone(),
            history: vec![v],
            history_ptr: 0,
            name: Arc::from(name),
            path: Some(path_ref.to_path_buf()),
            cursor_pos: (0, 0),
            scroll_offset: (0, 0),
            scroll_thres: (0, 0),
            file_info: None,
            saved: true,
        };
        s.refresh_file_info().unwrap();
        Ok(s)
    }
    pub fn change_name(&mut self, name: &str) {
        self.name = Arc::from(name)
    }

    pub fn get_line_count(&self) -> usize {
        self.content.len()        
    }

    pub fn delete_content_at(&mut self, len_in_chars: usize) -> Result<(), BufferError> {
        let (x, y) = self.cursor_pos;
        if y >= self.content.len() { return Err(BufferError::InvalidPosition); }
        
        let line = &mut self.content[y];
        let start_byte = char_to_byte_idx(line, x);
        let end_byte = char_to_byte_idx(line, x + len_in_chars);
        
        line.drain(start_byte..end_byte);
        self.handle_change();
        Ok(())
    }

    pub fn change_content_at(&mut self, len_in_chars: usize, replace_str: &str) -> Result<(), BufferError> {
        let (x, y) = self.cursor_pos;
        if y >= self.content.len() { return Err(BufferError::InvalidPosition); }
        
        let line = &mut self.content[y];
        let start_byte = char_to_byte_idx(line, x);
        let end_byte = char_to_byte_idx(line, x + len_in_chars);
        
        line.replace_range(start_byte..end_byte, replace_str);
        self.handle_change();
        Ok(())
    }

    pub fn add_content_at(&mut self, add_str: &str) -> Result<(), BufferError> {
        let (x, y) = self.cursor_pos;
        if y >= self.content.len() { return Err(BufferError::InvalidPosition); }
        
        let byte_idx = char_to_byte_idx(&self.content[y], x);
        self.content[y].insert_str(byte_idx, add_str);
        self.handle_change();
        Ok(())
    }

    pub fn delete_line(&mut self) -> Result<String, BufferError> {
        let y = self.cursor_pos.1;
        if self.get_line_count() <= y {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        self.handle_change();
        Ok(self.content.remove(y)) 
    }

    pub fn add_new_line(&mut self) -> Result<(), BufferError> {
        let y = self.cursor_pos.1;
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
        if let Err(e) = self.refresh_file_info() {
            println!("error: {:?}", e);
        };
        self.saved = true;
        Ok(())
    }

    pub fn mv_cursor_right(&mut self) {
        let line = &self.content[self.cursor_pos.1];
        let char_count = get_line_len(line);
        let cur_pos = &mut self.cursor_pos;

        if cur_pos.0 < char_count {
            cur_pos.0 += 1;
        } else if cur_pos.1 < self.content.len() - 1 {
            cur_pos.1 += 1;
            cur_pos.0 = 0;
        }
    }

    pub fn mv_cursor_left(&mut self) {
        if self.cursor_pos.0 > 0 {
            self.cursor_pos.0 -= 1;
        } else if self.cursor_pos.1 > 0 {
            self.cursor_pos.1 -= 1;
            self.cursor_pos.0 = get_line_len(&self.content[self.cursor_pos.1])
        }
    }

    pub fn mv_cursor_up(&mut self) {
        if self.cursor_pos.1 > 0 {
            self.cursor_pos.1 -= 1;
            self.cursor_pos.0 = self.cursor_pos.0.min(get_line_len(&self.content[self.cursor_pos.1]));
        }
    }

    pub fn mv_cursor_down(&mut self) {
        if self.cursor_pos.1 < self.content.len() - 1 {
            self.cursor_pos.1 += 1;
            self.cursor_pos.0 = self.cursor_pos.0.min(get_line_len(&self.content[self.cursor_pos.1]));
        } else {
            self.content.push(String::new());
            self.cursor_pos.1 += 1;
            self.cursor_pos.0 = 0;
            self.handle_change();
        }
    }

    pub fn handle_backspace(&mut self) {
        let (x, y) = self.cursor_pos;
        if x > 0 {
            let line = &mut self.content[y];
            let byte_idx = char_to_byte_idx(line, x - 1);
            line.remove(byte_idx);
            self.cursor_pos.0 -= 1;
        } else if y > 0 {
            let current_line = self.content.remove(y);
            let prev_line = &mut self.content[y - 1];
            let prev_char_len = get_line_len(prev_line);
            
            prev_line.push_str(&current_line);
            self.cursor_pos.1 -= 1;
            self.cursor_pos.0 = prev_char_len;
        }
        self.handle_change();
    }

    pub fn handle_enter(&mut self) {
        let (x, y) = self.cursor_pos;
        let byte_idx = char_to_byte_idx(&self.content[y], x);
        
        let next_line_content = self.content[y].split_off(byte_idx);
        self.content.insert(y + 1, next_line_content);
        
        self.cursor_pos.1 += 1;
        self.cursor_pos.0 = 0;
        self.handle_change();
    }
    
    // change saved to false and add new content to history
    pub fn handle_change(&mut self) {
        self.saved = false;
        self.history.truncate(self.history_ptr + 1);
        let v: Vec<Arc<str>> = self.content.clone()
            .into_iter()
            .map(|s| Arc::<str>::from(s))
            .collect();
        self.history.push(v);
        self.history_ptr += 1;
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

    pub fn get_visual_width_upto(&self, line_idx: usize, char_idx: usize) -> usize {
        let line = &self.content[line_idx];
        line.graphemes(true)
            .take(char_idx)
            .map(|g| g.width())
            .sum()
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
            format: detect_line_ending(&self.content.join(""))
        };
       
        self.file_info = Some(info);
        Ok(())
    }

    pub fn check_cursor_pos(&mut self) {
        let (px, py) = &mut self.cursor_pos;
        if *py >= self.content.len() {
            *py = self.content.len() - 1; 
        }
        if *px > get_line_len(&self.content[*py]) {
            *px = get_line_len(&self.content[*py]);
        }
    }

    pub fn revoke(&mut self) {
        self.history_ptr = if self.history_ptr <= 0 {0} else {self.history_ptr - 1}; 
        self.content = arc_vec_to_string(self.history[self.history_ptr].clone());
    }

    pub fn mv_cursor_tail(&mut self) {
        self.cursor_pos.0 = get_line_len(&self.content[self.cursor_pos.1]);
    }

    pub fn mv_cursor_head(&mut self) {
        self.cursor_pos.0 = 0;
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


