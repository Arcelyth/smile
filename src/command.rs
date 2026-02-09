#![allow(dead_code)]
use crate::buffer::BufferManager;
use crate::app::{Screen, App};
use crate::error::*;
use crate::layout::layout_manager::*;
use crate::layout::tree::*;
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

    pub fn handle_command(
        &mut self,
        buf_m: &mut BufferManager,
        lm: &mut LayoutManager,
        cur_screen: &mut Screen, 
    ) -> Result<bool, LayoutError> {
        let buf = lm.get_current_buffer_mut(buf_m)?;
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
                    mv_cursor_head(lm);
                }
                "tail" => {
                    mv_cursor_tail(buf_m, lm);
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
                "sv" => {
                    split(buf_m, lm, SplitDirection::Vertical, None)?;
                }
                "sh" => {
                    split(buf_m, lm, SplitDirection::Horizontal, None)?;
                }
                "close" => {
                    close_current_pane(lm, cur_screen)?;
                    move_focus_in_pane(lm, MoveDir::Right);
                }
                "right pane" => {
                    move_focus_in_pane(lm, MoveDir::Right);
                }
                "left pane" => {
                    move_focus_in_pane(lm, MoveDir::Left);
                }
                "up pane" => {
                    move_focus_in_pane(lm, MoveDir::Up);
                }
                "down pane" => {
                    move_focus_in_pane(lm, MoveDir::Down);
                }
                "change pane" => {
                    change_pane(lm, 1)?;
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

pub fn create_new_buffer(bm: &mut BufferManager, lm: &mut LayoutManager, name: &str) {
    let id = bm.add_new_buffer(name);
    lm.change_current_buffer_id(id);
}

pub fn mv_cursor_right(bm: &mut BufferManager, lm: &mut LayoutManager) {
    lm.mv_cursor_right(bm);
}

pub fn mv_cursor_left(bm: &mut BufferManager, lm: &mut LayoutManager) {
    lm.mv_cursor_left(bm);
}

pub fn mv_cursor_up(bm: &mut BufferManager, lm: &mut LayoutManager) {
    lm.mv_cursor_up(bm);
}

pub fn mv_cursor_down(bm: &mut BufferManager, lm: &mut LayoutManager) {
    lm.mv_cursor_down(bm);
}

pub fn mv_cursor_head(lm: &mut LayoutManager) {
    lm.mv_cursor_head();
}

pub fn mv_cursor_tail(bm: &mut BufferManager, lm: &mut LayoutManager) {
    lm.mv_cursor_tail(bm);
}

pub fn handle_backspace(bm: &mut BufferManager, lm: &mut LayoutManager) {
    lm.handle_backspace(bm);
}

pub fn handle_enter(bm: &mut BufferManager, lm: &mut LayoutManager) {
    lm.handle_enter(bm);
}

pub fn add_content_at(
    bm: &mut BufferManager,
    lm: &mut LayoutManager,
    add_str: &str,
) -> Result<(), LayoutError> {
    let pane = lm.get_current_pane_mut().ok_or(LayoutError::PaneNotFound)?;

    let (cursor_pos, buffer_id) = match pane {
        LayoutNode::Pane {
            cursor_pos,
            buffer_id,
            ..
        } => (cursor_pos, *buffer_id),
        _ => return Err(LayoutError::NotPane),
    };

    let buf = bm.get_buffer_mut(buffer_id)?;
    buf.add_content_at(add_str, *cursor_pos);
    Ok(())
}

pub fn save(bm: &mut BufferManager, lm: &mut LayoutManager) -> Result<(), LayoutError> {
    let pane = lm.get_current_pane_mut().ok_or(LayoutError::PaneNotFound)?;

    let (_cursor_pos, buffer_id) = match pane {
        LayoutNode::Pane {
            cursor_pos,
            buffer_id,
            ..
        } => (cursor_pos, *buffer_id),
        _ => return Err(LayoutError::NotPane),
    };

    let buf = bm.get_buffer_mut(buffer_id)?;
    Ok(buf.save()?)
}

pub fn is_buffer_binding(
    bm: &mut BufferManager,
    lm: &mut LayoutManager,
) -> Result<bool, LayoutError> {
    let pane = lm.get_current_pane_mut().ok_or(LayoutError::PaneNotFound)?;

    let (_cursor_pos, buffer_id) = match pane {
        LayoutNode::Pane {
            cursor_pos,
            buffer_id,
            ..
        } => (cursor_pos, *buffer_id),
        _ => return Err(LayoutError::NotPane),
    };

    let buf = bm.get_buffer_mut(buffer_id)?;

    Ok(buf.path.is_some())
}

pub fn revoke(bm: &mut BufferManager, lm: &mut LayoutManager) -> Result<(), LayoutError> {
    let pane = lm.get_current_pane_mut().ok_or(LayoutError::PaneNotFound)?;

    let (_cursor_pos, buffer_id) = match pane {
        LayoutNode::Pane {
            cursor_pos,
            buffer_id,
            ..
        } => (cursor_pos, *buffer_id),
        _ => return Err(LayoutError::NotPane),
    };

    let buf = bm.get_buffer_mut(buffer_id)?;
    buf.revoke();

    Ok(())
}

pub fn split(
    bm: &mut BufferManager,
    lm: &mut LayoutManager,
    direc: SplitDirection,
    buf_id: Option<usize>,
) -> Result<(), LayoutError> {
    let pane = lm.get_current_pane().ok_or(LayoutError::PaneNotFound)?;

    let id = match pane {
        LayoutNode::Pane { id, .. } => id,
        _ => return Err(LayoutError::NotPane),
    };

    let new_id = match buf_id {
        Some(new_id) => {
            // check if the buffer id is valid
            Some(bm.get_buffer(new_id)?.id)
        }
        None => None,
    };

    lm.split(id, new_id, direc, bm)?;
    Ok(())
}

pub fn close_current_pane(lm: &mut LayoutManager, cur_screen: &mut Screen) -> Result<(), LayoutError> {
    let pane = lm.get_current_pane().ok_or(LayoutError::PaneNotFound)?;

    let id = match pane {
        LayoutNode::Pane { id, .. } => id,
        _ => return Err(LayoutError::NotPane),
    };

    lm.remove(id)?;

    if lm.panes.is_none() {
        *cur_screen = Screen::Welcome;
    }
    Ok(())
}

pub fn move_focus_in_pane(lm: &mut LayoutManager, direc: MoveDir) {
    lm.move_focus(direc);
}

pub fn change_pane(lm: &mut LayoutManager, id: usize) -> Result<(), LayoutError> {
    if !lm.contain_id(id) {
        return Err(LayoutError::IdNotFound);
    }
    lm.current_layout = id;
    Ok(())
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
