use super::tree::*;
use crate::buffer::*;
use crate::error::*;
use crate::op::EditOp;
use crate::utils::{char_to_byte_idx, get_line_len, overlap};
use ratatui::layout::Rect;
use std::collections::HashMap;

pub struct LayoutManager {
    pub pane_rects: HashMap<usize, Rect>,
    pub id_counter: usize,
    pub panes: Option<LayoutNode>,
    pub current_layout: usize,
}

// for moving in panes
#[derive(Debug, Copy, Clone)]
pub enum MoveDir {
    Left,
    Right,
    Up,
    Down,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            pane_rects: HashMap::new(),
            id_counter: 1,
            panes: None,
            current_layout: 0,
        }
    }

    pub fn init(&mut self, buffer_id: usize) {
        self.panes = Some(LayoutNode::new_pane(1, buffer_id));
        self.id_counter += 1;
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

    pub fn remove(&mut self, target_id: usize) -> Result<Option<usize>, LayoutError> {
        let node = if let Some(n) = &self.panes {
            n
        } else {
            return Err(LayoutError::NoNode);
        };

        let new_nodes = remove_pane(node.clone(), target_id);

        self.panes = new_nodes;

        if self.current_layout == target_id {
            if let Some(ref new_root) = self.panes {
                if let Some(new_id) = get_first_pane_id(new_root) {
                    self.current_layout = new_id;
                } else {
                    self.current_layout = 0;
                }
            } else {
                self.current_layout = 0;
            }
        }
        self.pane_rects.remove(&target_id);
        Ok(Some(target_id))
    }

    pub fn get_current_rect(&self) -> Option<&Rect> {
        let current = self.current_layout;
        let pane_rects = &self.pane_rects;
        pane_rects.get(&current)
    }

    pub fn move_focus(&mut self, dir: MoveDir) -> Option<usize> {
        let current = self.current_layout;
        let pane_rects = &self.pane_rects;
        let cur = pane_rects.get(&current)?;

        let mut best: Option<(usize, u16)> = None;

        for (&id, rect) in pane_rects {
            if id == current {
                continue;
            }

            let candidate = match dir {
                MoveDir::Right => {
                    rect.x >= cur.x + cur.width
                        && overlap(rect.y, rect.y + rect.height, cur.y, cur.y + cur.height)
                }
                MoveDir::Left => {
                    rect.x + rect.width <= cur.x
                        && overlap(rect.y, rect.y + rect.height, cur.y, cur.y + cur.height)
                }
                MoveDir::Down => {
                    rect.y >= cur.y + cur.height
                        && overlap(rect.x, rect.x + rect.width, cur.x, cur.x + cur.width)
                }
                MoveDir::Up => {
                    rect.y + rect.height <= cur.y
                        && overlap(rect.x, rect.x + rect.width, cur.x, cur.x + cur.width)
                }
            };

            if !candidate {
                continue;
            }

            let dist = match dir {
                MoveDir::Right => rect.x - (cur.x + cur.width),
                MoveDir::Left => cur.x - (rect.x + rect.width),
                MoveDir::Down => rect.y - (cur.y + cur.height),
                MoveDir::Up => cur.y - (rect.y + rect.height),
            };

            match best {
                None => best = Some((id, dist)),
                Some((_, best_dist)) if dist < best_dist => best = Some((id, dist)),
                _ => {}
            }
        }

        match best.map(|(id, _)| id) {
            res @ Some(id) => {
                self.current_layout = id;
                res
            }
            None => None,
        }
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
        let pane = self
            .get_current_pane_mut()
            .ok_or(LayoutError::PaneNotFound)?;
        match pane {
            LayoutNode::Pane { buffer_id, .. } => {
                *buffer_id = id;
            }
            _ => return Err(LayoutError::NotPane),
        };
        Ok(())
    }

    pub fn contain_id(&self, id: usize) -> bool {
        self.pane_rects.contains_key(&id)
    }

    pub fn mv_cursor_right(&mut self, buf_m: &mut BufferManager) -> Result<(), LayoutError> {
        let freemod = false;
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
        } else if y + 1 < buf.content.len() && freemod {
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
        // press down can add new line
        let freemod = false;
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
        } else if freemod {
            cursor_pos.1 += 1;
            cursor_pos.0 = 0;
            buf.apply_op(
                EditOp::InsertLine {
                    y: cursor_pos.1,
                    text: "".into(),
                },
                true,
            )?;
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

    pub fn mv_cursor_head(&mut self) -> Result<(), LayoutError> {
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
            let del_str = &buf.content[y][x - 1..x];
            buf.apply_op(
                EditOp::Delete {
                    pos: (x - 1, y),
                    len: 1,
                    text: del_str.into(),
                },
                true,
            )?;
            cursor_pos.0 -= 1;
        } else if y > 0 {
            let prev_line = &buf.content[y - 1];
            let prev_len = get_line_len(prev_line);

            if prev_len > 0 {
                let start = char_to_byte_idx(prev_line, prev_len - 1);
                let end = char_to_byte_idx(prev_line, prev_len);

                let del_str = &prev_line[start..end];

                buf.apply_op(
                    EditOp::Delete {
                        pos: (prev_len - 1, y - 1),
                        len: 1,
                        text: del_str.into(),
                    },
                    true,
                )?;
            }
            cursor_pos.1 -= 1;
            cursor_pos.0 = prev_len;
        }

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
        buf.apply_op(
            EditOp::InsertLine {
                y: y + 1,
                text: next_line.into(),
            },
            true,
        )?;
        cursor_pos.1 += 1;
        cursor_pos.0 = 0;

        Ok(())
    }
   
}

pub fn update_scroll(
    buf_m: &BufferManager,
    viewport_height: usize,
    viewport_width: usize,
    cursor_pos: &mut (usize, usize),
    scroll_offset: &mut (usize, usize),
    scroll_thres: &mut (usize, usize),
    buffer_id: usize,
) -> Result<(), LayoutError> {
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

    if x >= (scroll_offset.0 + viewport_width).saturating_sub(scroll_thres.0) {
        scroll_offset.0 = x + scroll_thres.0 - viewport_width + 1;
    }

    if x < scroll_offset.0 {
        scroll_offset.0 = x;
    }

    Ok(())
}

pub fn check_cursor_pos(
    buf_m: &BufferManager,
    cursor_pos: &mut (usize, usize),
    buffer_id: usize,
) -> Result<(), LayoutError> {
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
