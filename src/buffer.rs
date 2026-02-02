use std::sync::Arc;

pub struct Buffer {
    content: Arc<str>,
    name: Arc<str>,
}

impl Buffer {
    pub fn new(name: &str) -> Self {
        Self {
            content: Arc::from(""),
            name: Arc::from(name), 
        }
    }
    pub fn from_content(content: &str, name: & str) -> Self {
        Self {
            content: Arc::from(content),
            name: Arc::from(name),
        }
    } 
    pub fn change_name(&mut self, name: &str) {
        self.name = Arc::from(name);
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
    pub fn add_buffer(&mut self, name: &str) {
        let new_buffer = Buffer::new(name);
        self.buffers.push(new_buffer);
        self.current_buffer = Some(self.buffers.len() - 1);
    }

}
