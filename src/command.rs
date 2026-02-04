#![allow(dead_code)]

use crate::error::BufferError;

#[derive(Debug, Copy, Clone)]
pub enum KaoMoJi {
    Smile,
    Happy,
    Angry,
    Sleep,
    Wink,
}

// KaoCo is the name of the command bar in smile
pub struct KaoCo {
    // kaomoji of KaoCo
    pub kmj: KaoMoJi,
    pub content: String,
    pub cursor_pos: (usize, usize),
    pub scroll_offset: (usize, usize),
}

impl KaoCo {
    pub fn new() -> Self {
        Self {
            kmj: KaoMoJi::Smile,
            content: String::new(),
            cursor_pos: (0, 0),
            scroll_offset: (0, 0),
        }
    }

    pub fn clean(&mut self) {
        self.content = String::new();
        self.cursor_pos = (0, 0);
        self.scroll_offset = (0, 0);
    }

    pub fn change_content_at(&mut self, len: usize, replace_str: &str) -> Result<(), BufferError> {
        let x = self.cursor_pos.0;
        if x + len > self.content.len()  {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        self.content.replace_range(x..x+len, replace_str);
        Ok(())
    }

    pub fn delete_content_at(&mut self, len: usize) -> Result<(), BufferError> {
        let x = self.cursor_pos.0;
        if x + len > self.content.len()  {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        self.content.drain(x..x+len);
        Ok(())
    }

    pub fn add_content_at(&mut self, add_str: &str) -> Result<(), BufferError> {
        let x = self.cursor_pos.0;
        if x > self.content.len()  {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        Ok(self.content.insert_str(x, add_str))
    }

    pub fn mv_cursor_right(&mut self) {
        let line_str = &self.content;
        let x_byte = self.cursor_pos.0;
        
        if x_byte < line_str.len() {
            let next_pos = line_str[x_byte..]
                .char_indices()
                .nth(1)
                .map(|(idx, _)| x_byte + idx)
                .unwrap_or_else(|| line_str.len());
            self.cursor_pos.0 = next_pos;
        }
    }

    pub fn mv_cursor_left(&mut self) {
        let x_byte = self.cursor_pos.0;
        if x_byte > 0 {
            let prev_pos = self.content[..x_byte]
                .char_indices()
                .last()
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            self.cursor_pos.0 = prev_pos;
        }
    }

    pub fn handle_backspace(&mut self) {
        let x_byte = self.cursor_pos.0;
        if x_byte > 0 {
            let prev_pos = self.content[..x_byte]
                .char_indices()
                .last()
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            
            self.content.remove(prev_pos);
            self.cursor_pos.0 = prev_pos;
        }
    }

    pub fn update_scroll(&mut self, viewport_width: usize) {
        let thres = 0; 
        let x = self.cursor_pos.0;
        
        let visual_x = self.content[..x].len();

        if visual_x >= (self.scroll_offset.0 + viewport_width).saturating_sub(thres) {
            self.scroll_offset.0 = visual_x + thres - viewport_width + 1;
        }

        if visual_x < self.scroll_offset.0 {
            self.scroll_offset.0 = visual_x.saturating_sub(thres);
        }
    }

    pub fn handle_command(&mut self) {
         
    }
}

pub fn kaomoji_to_text(kind: KaoMoJi) -> String {
    match kind {
        KaoMoJi::Wink => "☆(>ᴗ•)",
        KaoMoJi::Angry => "(`ᴖ´)",
        KaoMoJi::Sleep => "(ᴗ˳ᴗ)ᶻ𝗓𐰁",
        KaoMoJi::Happy => "(>ᴗ<)",
        _ => "(•ᴗ•)"
    }.to_string()
} 


