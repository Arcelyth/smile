use crate::buffer::BufferManager;

#[derive(Debug, Copy, Clone)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub enum LayoutNode {
    Pane {
        id: usize,
        buffer_id: usize,
        cursor_pos: (usize, usize),
        scroll_offset: (usize, usize),
        scroll_thres: (usize, usize),
    },
    Split {
        direc: SplitDirection,
        ratio: f32, // 0.0 ~ 1.0
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

impl LayoutNode {
    pub fn new_pane(id: usize, buffer_id: usize) -> Self {
        LayoutNode::Pane {
            id,
            buffer_id,
            cursor_pos: (0, 0),
            scroll_offset: (0, 0),
            scroll_thres: (0, 0),
        }
    }

    pub fn get_buffer_id(&self, pane_id: usize) -> Option<usize> {
        match self {
            LayoutNode::Pane { id, buffer_id, .. } => {
                if *id == pane_id {
                    Some(*buffer_id)
                } else {
                    None
                }
            }
            LayoutNode::Split { first, second, .. } => first
                .get_buffer_id(pane_id)
                .or_else(|| second.get_buffer_id(pane_id)),
        }
    }

    pub fn get_pane(&self, pane_id: usize) -> Option<&LayoutNode> {
        match self {
            pane @ LayoutNode::Pane { id, .. } => {
                if *id == pane_id {
                    Some(pane)
                } else {
                    None
                }
            }
            LayoutNode::Split { first, second, .. } => {
                first.get_pane(pane_id).or_else(|| second.get_pane(pane_id))
            }
        }
    }

    pub fn get_pane_mut(&mut self, pane_id: usize) -> Option<&mut LayoutNode> {
        match self {
            LayoutNode::Pane { id, .. } => {
                if *id == pane_id {
                    Some(self)
                } else {
                    None
                }
            }
            LayoutNode::Split { first, second, .. } => first
                .get_pane_mut(pane_id)
                .or_else(|| second.get_pane_mut(pane_id)),
        }
    }
}

pub fn split_current(
    root: &mut LayoutNode,
    target: usize,
    new_id: usize,
    new_buf_id: Option<usize>,
    direc: SplitDirection,
    buf_m: &mut BufferManager,
) {
    if let LayoutNode::Pane {
        id,
        buffer_id,
        cursor_pos,
        scroll_offset,
        scroll_thres,
        ..
    } = root
    {
        if *id == target {
            let new_buf_id = if let Some(idx) = new_buf_id {
                idx
            } else {
                *buffer_id
            };
            *root = LayoutNode::Split {
                direc,
                ratio: 0.5,
                first: Box::new(LayoutNode::Pane {
                    id: *id,
                    buffer_id: *buffer_id,
                    cursor_pos: *cursor_pos,
                    scroll_offset: *scroll_offset,
                    scroll_thres: *scroll_thres,
                }),
                second: Box::new(LayoutNode::new_pane(new_id, new_buf_id)),
            };
        }
    } else if let LayoutNode::Split { first, second, .. } = root {
        split_current(first, target, new_id, new_buf_id, direc, buf_m);
        split_current(second, target, new_id, new_buf_id, direc, buf_m);
    }
}
