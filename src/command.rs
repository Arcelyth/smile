#![allow(dead_code)]
use unicode_width::UnicodeWidthStr;
use unicode_segmentation::UnicodeSegmentation;
use crate::error::BufferError;
use crate::utils::*;
use crate::buffer::Buffer;

#[derive(Debug, Copy, Clone)]
pub enum KaoMoJi {
    Smile,
    Happy,
    Angry,
    Sleep,
    Wink,
}

#[derive(Debug, Copy, Clone)]
pub enum CmdStatus{
    Normal, 
    Success,
    Failed,
}


// KaoCo is the name of the command bar in smile
pub struct KaoCo {
    // kaomoji of KaoCo
    pub kmj: KaoMoJi,
    pub content: String,
    pub cursor_pos: (usize, usize),
    pub scroll_offset: (usize, usize),
    pub status: CmdStatus,
}

impl KaoCo {
    pub fn new() -> Self {
        Self {
            kmj: KaoMoJi::Smile,
            content: String::new(),
            cursor_pos: (0, 0),
            scroll_offset: (0, 0),
            status: CmdStatus::Normal,
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
        let line = &mut self.content;
        let start_byte = char_to_byte_idx(line, x);
        let end_byte = char_to_byte_idx(line, x + len);

        self.content.replace_range(start_byte..end_byte, replace_str);
        Ok(())
    }

    pub fn delete_content_at(&mut self, len: usize) -> Result<(), BufferError> {
        let x = self.cursor_pos.0;
        if x + len > self.content.len()  {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        let line = &mut self.content;
        let start_byte = char_to_byte_idx(line, x);
        let end_byte = char_to_byte_idx(line, x + len);
        
        self.content.drain(start_byte..end_byte);

        Ok(())
    }

    pub fn add_content_at(&mut self, add_str: &str) -> Result<(), BufferError> {
        let x = self.cursor_pos.0;
        if x > get_line_len(&self.content)  {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }

        let byte_idx = char_to_byte_idx(&self.content, x);
        Ok(self.content.insert_str(byte_idx, add_str))
    }

    pub fn mv_cursor_right(&mut self) {
        let line = &self.content;
        let char_count = get_line_len(line);
        let cur_pos = &mut self.cursor_pos;

        if cur_pos.0 < char_count {
            cur_pos.0 += 1;
        } 
    }

    pub fn mv_cursor_left(&mut self) {
        if self.cursor_pos.0 > 0 {
            self.cursor_pos.0 -= 1;
        } 
    }

    pub fn handle_backspace(&mut self) {
        let x = self.cursor_pos.0;
        if x > 0 {
            let line = &mut self.content;
            let byte_idx = char_to_byte_idx(line, x - 1);
            line.remove(byte_idx);
            self.cursor_pos.0 -= 1;
        } 
    }
   
    pub fn update_scroll(&mut self, viewport_width: usize) {
        let thres = 0; 
        
        let visual_x = get_line_len(&self.content);

        if visual_x >= (self.scroll_offset.0 + viewport_width).saturating_sub(thres) {
            self.scroll_offset.0 = visual_x + thres - viewport_width + 1;
        }

        if visual_x < self.scroll_offset.0 {
            self.scroll_offset.0 = visual_x.saturating_sub(thres);
        }
    }

    pub fn get_visual_width_upto(&self, char_idx: usize) -> usize {
        let line = &self.content;
        line.graphemes(true)
            .take(char_idx)
            .map(|g| g.width())
            .sum()
    }


    pub fn handle_command(&mut self, buf: &mut Buffer) -> Result<(), BufferError>{
        match self.content.trim() {
            "revoke" => {
                buf.revoke();
            }
            "head" => {
                buf.mv_cursor_head();
            }
            "tail" => {
                buf.mv_cursor_tail();
            }
            "save" => {
                buf.save()?;
            }
            _ => {}
        };  
        self.status = CmdStatus::Success;
        Ok(())
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


