use super::tree::*;
use crate::buffer::*;
use crate::error::*;
use crate::utils::{char_to_byte_idx, get_line_len};
use std::sync::Arc;

pub struct LayoutManager {
    pub id_counter: usize,
    pub panes: Option<LayoutNode>,
    pub current_layout: usize,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            id_counter: 1,
            panes: None,
            current_layout: 0,
        }
    }

    pub fn init(&mut self, buffer_id: usize) {
        self.id_counter += 1;
        self.panes = Some(LayoutNode::new_pane(1, buffer_id));
        self.current_layout = 1;
    }

    pub fn split(
        &mut self,
        target_id: usize,
        new_buf_id: Option<usize>,
        direc: SplitDirection,
        buf_m: &mut BufferManager,
    ) -> Result<usize, LayoutError> {
        let node = if let Some(n) = &mut self.panes {
            n
        } else {
            return Err(LayoutError::NoNode);
        };
        split_current(node, target_id, self.id_counter, new_buf_id, direc, buf_m);
        self.current_layout = self.id_counter;
        self.id_counter += 1;
        Ok(self.id_counter)
    }

    pub fn get_current_pane(&self) -> Option<LayoutNode> {
        self.panes.as_ref()?.get_pane(self.current_layout).cloned()
    }

    pub fn get_current_pane_mut(&mut self) -> Option<&mut LayoutNode> {
        self.panes.as_mut()?.get_pane_mut(self.current_layout)
    }

    pub fn get_current_buffer<'a>(
        &self,
        buf_m: &'a BufferManager,
    ) -> Result<&'a Buffer, LayoutError> {
        let pane = self.get_current_pane().ok_or(LayoutError::PaneNotFound)?;

        let buffer_id = match pane {
            LayoutNode::Pane { buffer_id, .. } => buffer_id,
            _ => return Err(LayoutError::NotPane),
        };

        buf_m
            .get_buffer(buffer_id)
            .map_err(|e| LayoutError::BufferErr(e))
    }

    pub fn get_current_buffer_mut<'a>(
        &self,
        buf_m: &'a mut BufferManager,
    ) -> Result<&'a mut Buffer, LayoutError> {
        let pane = self.get_current_pane().ok_or(LayoutError::PaneNotFound)?;

        let buffer_id = match pane {
            LayoutNode::Pane { buffer_id, .. } => buffer_id,
            _ => return Err(LayoutError::NotPane),
        };

        buf_m
            .get_buffer_mut(buffer_id)
            .map_err(|e| LayoutError::BufferErr(e))
    }

    pub fn change_current_buffer_id(&mut self, id: usize) -> Result<(), LayoutError> {
        let pane = self.get_current_pane_mut().ok_or(LayoutError::PaneNotFound)?;
        match pane {
            LayoutNode::Pane { buffer_id , ..} => {
                *buffer_id = id;
            },
            _ => return Err(LayoutError::NotPane),
        };
        Ok(())
    }

    pub fn mv_cursor_right(&mut self, buf_m: &mut BufferManager) -> Result<(), LayoutError> {
        let pane = self
            .get_current_pane_mut()
            .ok_or(LayoutError::PaneNotFound)?;

        let (cursor_pos, buffer_id) = match pane {
            LayoutNode::Pane {
                cursor_pos,
                buffer_id,
                ..
            } => (cursor_pos, *buffer_id),
            _ => return Err(LayoutError::NotPane),
        };

        let buf = buf_m.get_buffer_mut(buffer_id)?;

        let y = cursor_pos.1;
        let x = cursor_pos.0;

        let line = &buf.content[y];
        let line_len = get_line_len(line);

        if x < line_len {
            cursor_pos.0 += 1;
        } else if y + 1 < buf.content.len() {
            cursor_pos.1 += 1;
            cursor_pos.0 = 0;
        }

        Ok(())
    }

    pub fn mv_cursor_left(&mut self, buf_m: &mut BufferManager) -> Result<(), LayoutError> {
        let pane = self
            .get_current_pane_mut()
            .ok_or(LayoutError::PaneNotFound)?;

        let (cursor_pos, buffer_id) = match pane {
            LayoutNode::Pane {
                cursor_pos,
                buffer_id,
                ..
            } => (cursor_pos, *buffer_id),
            _ => return Err(LayoutError::NotPane),
        };

        let buf = buf_m.get_buffer_mut(buffer_id)?;

        if cursor_pos.0 > 0 {
            cursor_pos.0 -= 1;
        } else if cursor_pos.1 > 0 {
            cursor_pos.1 -= 1;
            cursor_pos.0 = get_line_len(&buf.content[cursor_pos.1]);
        }

        Ok(())
    }

    pub fn mv_cursor_up(&mut self, buf_m: &mut BufferManager) -> Result<(), LayoutError> {
        let pane = self
            .get_current_pane_mut()
            .ok_or(LayoutError::PaneNotFound)?;

        let (cursor_pos, buffer_id) = match pane {
            LayoutNode::Pane {
                cursor_pos,
                buffer_id,
                ..
            } => (cursor_pos, *buffer_id),
            _ => return Err(LayoutError::NotPane),
        };

        let buf = buf_m.get_buffer_mut(buffer_id)?;

        if cursor_pos.1 > 0 {
            cursor_pos.1 -= 1;
            cursor_pos.0 = cursor_pos.0.min(get_line_len(&buf.content[cursor_pos.1]));
        }

        Ok(())
    }

    pub fn mv_cursor_down(&mut self, buf_m: &mut BufferManager) -> Result<(), LayoutError> {
        let pane = self
            .get_current_pane_mut()
            .ok_or(LayoutError::PaneNotFound)?;

        let (cursor_pos, buffer_id) = match pane {
            LayoutNode::Pane {
                cursor_pos,
                buffer_id,
                ..
            } => (cursor_pos, *buffer_id),
            _ => return Err(LayoutError::NotPane),
        };

        let buf = buf_m.get_buffer_mut(buffer_id)?;

        if cursor_pos.1 + 1 < buf.content.len() {
            cursor_pos.1 += 1;
            cursor_pos.0 = cursor_pos.0.min(get_line_len(&buf.content[cursor_pos.1]));
        } else {
            buf.content.push(String::new());
            cursor_pos.1 += 1;
            cursor_pos.0 = 0;
            buf.handle_change();
        }

        Ok(())
    }

    pub fn mv_cursor_tail(&mut self, buf_m: &mut BufferManager) -> Result<(), LayoutError> {
        let pane = self
            .get_current_pane_mut()
            .ok_or(LayoutError::PaneNotFound)?;

        let (cursor_pos, buffer_id) = match pane {
            LayoutNode::Pane {
                cursor_pos,
                buffer_id,
                ..
            } => (cursor_pos, *buffer_id),
            _ => return Err(LayoutError::NotPane),
        };

        let buf = buf_m.get_buffer_mut(buffer_id)?;

        cursor_pos.0 = get_line_len(&buf.content[cursor_pos.1]);
        Ok(())
    }

    pub fn mv_cursor_head(&mut self) -> Result<(), LayoutError>{
        let pane = self
            .get_current_pane_mut()
            .ok_or(LayoutError::PaneNotFound)?;

        let (cursor_pos, _buffer_id) = match pane {
            LayoutNode::Pane {
                cursor_pos,
                buffer_id,
                ..
            } => (cursor_pos, *buffer_id),
            _ => return Err(LayoutError::NotPane),
        };

        cursor_pos.0 = 0;
        Ok(())
    }


    pub fn handle_backspace(&mut self, buf_m: &mut BufferManager) -> Result<(), LayoutError> {
        let pane = self
            .get_current_pane_mut()
            .ok_or(LayoutError::PaneNotFound)?;

        let (cursor_pos, buffer_id) = match pane {
            LayoutNode::Pane {
                cursor_pos,
                buffer_id,
                ..
            } => (cursor_pos, *buffer_id),
            _ => return Err(LayoutError::NotPane),
        };

        let buf = buf_m.get_buffer_mut(buffer_id)?;

        let (x, y) = *cursor_pos;

        if x > 0 {
            let line = &mut buf.content[y];
            let byte_idx = char_to_byte_idx(line, x - 1);
            line.remove(byte_idx);
            cursor_pos.0 -= 1;
        } else if y > 0 {
            let current = buf.content.remove(y);
            let prev = &mut buf.content[y - 1];
            let prev_len = get_line_len(prev);

            prev.push_str(&current);
            cursor_pos.1 -= 1;
            cursor_pos.0 = prev_len;
        }

        buf.handle_change();
        Ok(())
    }

    pub fn handle_enter(&mut self, buf_m: &mut BufferManager) -> Result<(), LayoutError> {
        let pane = self
            .get_current_pane_mut()
            .ok_or(LayoutError::PaneNotFound)?;

        let (cursor_pos, buffer_id) = match pane {
            LayoutNode::Pane {
                cursor_pos,
                buffer_id,
                ..
            } => (cursor_pos, *buffer_id),
            _ => return Err(LayoutError::NotPane),
        };

        let buf = buf_m.get_buffer_mut(buffer_id)?;

        let (x, y) = *cursor_pos;
        let byte_idx = char_to_byte_idx(&buf.content[y], x);

        let next_line = buf.content[y].split_off(byte_idx);
        buf.content.insert(y + 1, next_line);

        cursor_pos.1 += 1;
        cursor_pos.0 = 0;

        buf.handle_change();
        Ok(())
    }

    pub fn update_scroll(
        &mut self,
        buf_m: &BufferManager,
        viewport_height: usize,
        viewport_width: usize,
    ) -> Result<(), LayoutError> {
        let pane = self
            .get_current_pane_mut()
            .ok_or(LayoutError::PaneNotFound)?;

        let (cursor_pos, scroll_offset, scroll_thres, buffer_id) = match pane {
            LayoutNode::Pane {
                cursor_pos,
                scroll_offset,
                scroll_thres,
                buffer_id,
                ..
            } => (cursor_pos, scroll_offset, scroll_thres, *buffer_id),
            _ => return Err(LayoutError::NotPane),
        };

        let buf = buf_m.get_buffer(buffer_id)?;

        let (x, y) = *cursor_pos;
        let total_lines = buf.content.len();

        if y >= total_lines {
            cursor_pos.1 = total_lines.saturating_sub(1);
        }

        if y >= (scroll_offset.1 + viewport_height).saturating_sub(scroll_thres.1) {
            scroll_offset.1 = y + scroll_thres.1 - viewport_height + 1;
        }

        if y < scroll_offset.1 {
            scroll_offset.1 = y;
        }

        let line_width = buf.content[y].len();

        if x >= (scroll_offset.0 + viewport_width).saturating_sub(scroll_thres.0) {
            scroll_offset.0 = x + scroll_thres.0 - viewport_width + 1;
        }

        if x < scroll_offset.0 {
            scroll_offset.0 = x;
        }

        Ok(())
    }

    pub fn check_cursor_pos(&mut self, buf_m: &BufferManager) -> Result<(), LayoutError> {
        let pane = self
            .get_current_pane_mut()
            .ok_or(LayoutError::PaneNotFound)?;

        let (cursor_pos, buffer_id) = match pane {
            LayoutNode::Pane {
                cursor_pos,
                buffer_id,
                ..
            } => (cursor_pos, *buffer_id),
            _ => return Err(LayoutError::NotPane),
        };

        let buf = buf_m.get_buffer(buffer_id)?;

        let (px, py) = cursor_pos;

        if *py >= buf.content.len() {
            *py = buf.content.len().saturating_sub(1);
        }

        let line_len = get_line_len(&buf.content[*py]);
        if *px > line_len {
            *px = line_len;
        }

        Ok(())
    }
}
