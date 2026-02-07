#![allow(dead_code)]
use crate::buffer::{Buffer, BufferManager};
use crate::error::BufferError;
use crate::utils::*;
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Copy, Clone)]
pub enum KaoMoJi {
    Smile,
    Happy,
    Angry,
    Sleep,
    Wink,
}

#[derive(Debug, Copy, Clone)]
pub enum ExCmd {
    AskAndSave,
}

#[derive(Debug, Copy, Clone)]
pub enum CmdStatus {
    Normal,
    Exec(ExCmd),
    Success,
    Failed,
}

// KaoCo is the name of the command bar in smile
pub struct KaoCo {
    // kaomoji of KaoCo
    pub kmj: KaoMoJi,
    pub say: Arc<str>,
    pub content: String,
    pub cursor_pos: (usize, usize),
    pub scroll_offset: (usize, usize),
    pub status: CmdStatus,
}

impl KaoCo {
    pub fn new() -> Self {
        Self {
            kmj: KaoMoJi::Smile,
            say: Arc::from(""),
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

    pub fn clean_all(&mut self) {
        self.content = String::new();
        self.say = "".into();
        self.cursor_pos = (0, 0);
        self.scroll_offset = (0, 0);
    }

    pub fn change_content_at(&mut self, len: usize, replace_str: &str) -> Result<(), BufferError> {
        let x = self.cursor_pos.0;
        if x + len > self.content.len() {
            println!("[Warning]: Change content at a invalid position.");
            return Err(BufferError::InvalidPosition);
        }
        let line = &mut self.content;
        let start_byte = char_to_byte_idx(line, x);
        let end_byte = char_to_byte_idx(line, x + len);

        self.content
            .replace_range(start_byte..end_byte, replace_str);
        Ok(())
    }

    pub fn delete_content_at(&mut self, len: usize) -> Result<(), BufferError> {
        let x = self.cursor_pos.0;
        if x + len > self.content.len() {
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
        if x > get_line_len(&self.content) {
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
        line.graphemes(true).take(char_idx).map(|g| g.width()).sum()
    }

    pub fn handle_command(&mut self, buf_m: &mut BufferManager) -> Result<bool, BufferError> {
        let buf = if let Some(b) = buf_m.get_current_buffer_mut() {
            b
        } else {
            return Ok(false);
        };

        match self.status {
            CmdStatus::Exec(cmd) => match cmd {
                ExCmd::AskAndSave => {
                    buf.change_name(&self.content.trim());
                    println!("{}", buf.name);
                    buf.save()?;
                    self.say = "".into();
                }
            },
            _ => match self.content.trim() {
                "" => {
                    return Ok(false);
                }
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
                    if buf.path.is_some() {
                        let _ = buf.save();
                    } else {
                        let _ = self.ask_and_save();
                    }
                    return Ok(false);
                }
                "change name" => {
                    buf.change_name("omg");
                }
                "new buffer" => {
                    buf_m.add_new_buffer("Untitled");
                }
                "prev buffer" => {
                    buf_m.change_to_prev();
                }
                "next buffer" => {
                    buf_m.change_to_next();
                }
                _ => {
                    self.say = "Unknown command".into();
                    self.status = CmdStatus::Failed;
                    return Ok(false);
                }
            },
        }
        self.status = CmdStatus::Success;
        Ok(true)
    }

    pub fn ask_and_save(&mut self) {
        self.status = CmdStatus::Exec(ExCmd::AskAndSave);
        self.say = "Input the file's name".into();
    }
}

pub fn kaomoji_to_text(kind: KaoMoJi) -> String {
    match kind {
        KaoMoJi::Wink => "☆(>ᴗ•)",
        KaoMoJi::Angry => "(`ᴖ´)",
        KaoMoJi::Sleep => "(ᴗ˳ᴗ)ᶻ𝗓𐰁",
        KaoMoJi::Happy => "(>ᴗ<)",
        _ => "(•ᴗ•)",
    }
    .to_string()
}
