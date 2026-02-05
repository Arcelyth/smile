#![allow(dead_code)]
use unicode_width::UnicodeWidthStr;
use unicode_segmentation::UnicodeSegmentation;
use std::fs;
use color_eyre::Result;

#[derive(Debug, Clone, Copy)]
pub enum FileFormat {
    UNIX,
    DOS,
}

pub struct FileInfo {
    pub size: u64,
    pub read_only: bool,
    pub format: FileFormat,
}

impl FileInfo {
    pub fn new() -> Self {
        Self {
            size: 0, 
            read_only: false,
            format: FileFormat::UNIX,
        }
    }
}

pub fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
    s.grapheme_indices(true)
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

pub fn get_line_len(line: &String) -> usize {
    let graphemes = line.graphemes(true).collect::<Vec<&str>>();
    graphemes.len()
}

pub fn detect_line_ending(content: &str) -> FileFormat {
    if content.contains("\r\n") {
        FileFormat::DOS    // CRLF
    } else if content.contains('\n') {
        FileFormat::UNIX   // LF
    } else {
        FileFormat::UNIX   // CR
    }
}

pub fn get_format_text(f: FileFormat) -> &'static str {
    match f {
        FileFormat::DOS => "DOS",
        FileFormat::UNIX => "UNIX"
    }
}


