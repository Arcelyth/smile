use unicode_width::UnicodeWidthStr;
use unicode_segmentation::UnicodeSegmentation;

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


