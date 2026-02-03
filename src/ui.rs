use ratatui::layout::{Constraint, Direction, Layout, Alignment};
use ratatui::text::{Text, Line, Span};
use ratatui::style::{Modifier, Color, Style};
use ratatui::widgets::{Borders, Block, List, ListItem, Paragraph, ListDirection, Clear};
use ratatui::Frame;

use crate::app::{Screen, App};

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

        // show the editor
        let editor_main = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(10), 
                Constraint::Min(0), 
            ])
            .split(root[0]);

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
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
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
            .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
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
        let status_bar_border_color = Color::Rgb(252, 207, 248);
        let status_bar_block = Block::default()
            .title("Status Bar")
            .borders(Borders::ALL)
            .style(Style::default().fg(status_bar_border_color));
        let status_bar  = Paragraph::new("OK")
            .block(status_bar_block);

        frame.render_widget(status_bar, root[1]);

        // show the cursor
        let editor_area = editor_main[1];

        let cursor_x = editor_area.x + (app.cursor_pos.0.saturating_sub(app.scroll_offset.0)) as u16;
        let cursor_y = editor_area.y + (app.cursor_pos.1.saturating_sub(app.scroll_offset.1)) as u16 + 1;
        if cursor_x < editor_area.right() && cursor_y < editor_area.bottom() {
            frame.set_cursor(cursor_x, cursor_y);
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

