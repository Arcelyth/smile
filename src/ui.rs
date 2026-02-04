use ratatui::layout::{Constraint, Direction, Layout, Alignment};
use ratatui::text::{Text, Line, Span};
use ratatui::style::{Modifier, Color, Style};
use ratatui::widgets::{Borders, Block, List, ListItem, Paragraph, ListDirection, Clear};
use ratatui::Frame;

use crate::app::{Screen, App};
use crate::kaomoji::*;

pub fn ui(frame: &mut Frame, app: &mut App) {
     if let Screen::Welcome = app.current_screen {
        frame.render_widget(Clear, frame.area());
        let root = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Min(0)])
            .split(frame.area());

        let centered = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Length(50), 
                Constraint::Percentage(50),
            ])
            .split(root[0]);

        let content = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), 
                Constraint::Length(5), 
            ])
            .split(centered[1]);

        let banner = Paragraph::new(get_banner())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(banner, content[0]);

        let items = [
            ListItem::new("        Press a         Start a new buffer"),
            ListItem::new("        Press q                       Quit"),
        ];

        let list = List::new(items)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));

        frame.render_widget(list, content[1]);
    }

    if let Screen::Editor = app.current_screen {
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), 
                Constraint::Length(3), 
            ])
            .split(frame.area());

        // editor frame include editor and status bar
        let editor_frame = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3), 
                Constraint::Length(2), 
            ])
            .split(root[0]);

        // show the editor
        let editor_main = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(10), 
                Constraint::Min(0), 
            ])
            .split(editor_frame[0]);

        // check the scrolling
        let viewport_height = editor_main[1]
            .height
            .saturating_sub(2) as usize;
        let viewport_width = editor_main[1]
            .width
            .saturating_sub(2) as usize;

        app.update_scroll(viewport_height, viewport_width);

        let buf = if let Some(buf) = app.buf_manager.get_current_buffer() {
            buf
        } else {
            println!("Failed to get buffer.");
            return; 
        };

        let border_color = Color::Rgb(181, 235, 181);
        let font_color = Color::Rgb(240, 235, 213);
        let line_num_block =  Block::default()
            .borders(Borders::LEFT | Borders::TOP)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().fg(Color::DarkGray));
        let total_lines = buf.content.len();
        let line_num: Vec<Line> = (1..=total_lines)
        .map(|n| {
            Line::from(Span::styled(
                format!("{:>3} ", n), 
                Style::default().fg(Color::DarkGray)
            ))
        })
        .collect();
        let line_num_text= Paragraph::new(Text::from(line_num))
            .block(line_num_block)
            .scroll((app.scroll_offset.1 as u16, 0));

        frame.render_widget(line_num_text, editor_main[0]);

        let editor_block = Block::default()
            .title(buf.name.as_ref())
            .borders(Borders::RIGHT | Borders::TOP)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().fg(font_color));

        let lines: Vec<Line> = buf.content
            .iter()
            .map(|s| Line::from(s.as_str()))
            .collect();

        
        let content = Paragraph::new(Text::from(lines))
            .block(editor_block)
            .scroll((app.scroll_offset.1 as u16, app.scroll_offset.0 as u16));

        frame.render_widget(content, editor_main[1]);

        // show the status bar 
        let status_bar_main = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), 
                Constraint::Percentage(25), 
                Constraint::Percentage(25), 
                Constraint::Percentage(25), 
            ])
            .split(editor_frame[1]);

        let status_pos_font_color = Color::Rgb(252, 207, 248);
        let status_change_font_color = Color::Rgb(252, 207, 248);
        let status_second_font_color = Color::Rgb(252, 207, 248);
        let status_third_font_color = Color::Rgb(252, 207, 248);
//        let status_bar_bg_color = Color::Rgb(56, 53, 52);

        // show the status pos
        let pos = app.cursor_pos;
        let status_pos_block = Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT)
            .border_style(Style::default().fg(border_color))
            .style(Style::default()
                .fg(status_pos_font_color));
        let status_pos  = Paragraph::new(format!("{}:{}", pos.1 + 1, pos.0 + 1))
            .alignment(Alignment::Center)
            .block(status_pos_block);

        frame.render_widget(status_pos, status_bar_main[0]);

        // show the second secondition of status bar
        let status_second_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(border_color))
            .style(Style::default()
                .fg(status_second_font_color));
        let status_second  = Paragraph::new("-")
            .alignment(Alignment::Center)
            .block(status_second_block);

        frame.render_widget(status_second, status_bar_main[1]);

        // show the third thirdition of status bar
        let status_third_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(border_color))
            .style(Style::default()
                .fg(status_third_font_color));
        let status_third  = Paragraph::new("-")
            .alignment(Alignment::Center)
            .block(status_third_block);

        frame.render_widget(status_third, status_bar_main[2]);

        // show the change
        let status_ch_block = Block::default()
            .borders(Borders::BOTTOM | Borders::RIGHT)
            .border_style(Style::default().fg(border_color))
            .style(Style::default()
                .fg(status_change_font_color));
        let status_ch  = Paragraph::new("-")
            .alignment(Alignment::Center)
            .block(status_ch_block);

        frame.render_widget(status_ch, status_bar_main[3]);



        // command frame include kaomoji and command line
        let command_line_frame = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(8), 
                Constraint::Min(3), 
            ])
            .split(root[1]);

        let command_line_border_color = Color::Rgb(252, 207, 248);
        let kaomoji_color = Color::Rgb(226, 230, 156);
        let command_line_text_color = Color::Rgb(252, 207, 248);

        // show command line
        let kaomoji_block = Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .border_style(Style::default().fg(command_line_border_color))
            .style(Style::default().fg(kaomoji_color));
        let kmj = get_kaomoji(KaoMoJi::Smile); 
        let kaomoji  = Paragraph::new(kmj)
            .alignment(Alignment::Center)
            .block(kaomoji_block);

        frame.render_widget(kaomoji, command_line_frame[0]);

        // show command line
        let command_line_block = Block::default()
            .borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM)
            .border_style(Style::default().fg(command_line_border_color))
            .style(Style::default().fg(command_line_text_color));

        let command_line  = Paragraph::new(":")
            .block(command_line_block);

        frame.render_widget(command_line, command_line_frame[1]);

        // show the cursor
        let editor_area = editor_main[1];

        let cursor_x = editor_area.x + (app.cursor_pos.0.saturating_sub(app.scroll_offset.0)) as u16;
        let cursor_y = editor_area.y + (app.cursor_pos.1.saturating_sub(app.scroll_offset.1)) as u16 + 1;
        if cursor_x < editor_area.right() && cursor_y < editor_area.bottom() {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

fn get_banner() -> String {
    r#"
███████╗███╗   ███╗██╗██╗     ███████╗
██╔════╝████╗ ████║██║██║     ██╔════╝
███████╗██╔████╔██║██║██║     █████╗  
╚════██║██║╚██╔╝██║██║██║     ██╔══╝  
███████║██║ ╚═╝ ██║██║███████╗███████╗
╚══════╝╚═╝     ╚═╝╚═╝╚══════╝╚══════╝
    "#.to_string()
}


