use ratatui::layout::{Constraint, Direction, Layout, Alignment};
use ratatui::text::{Text, Line};
use ratatui::style::{Modifier, Color, Style};
use ratatui::widgets::{Borders, Block, List, ListItem, Paragraph, ListDirection, Clear};
use ratatui::Frame;

use crate::app::{Screen, App};

pub fn ui(frame: &mut Frame, app: &App) {
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
        let editor_block = Block::default()
            .title("Untitled")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let buf = if let Some(buf) = app.buf_manager.get_current_buffer() {
            buf
        } else {
            println!("Failed to get buffer.");
            return; 
        };
        let lines: Vec<Line> = buf.content
            .iter()
            .map(|s| Line::from(s.as_str()))
            .collect();

        let content = Paragraph::new(Text::from(lines))
            .block(editor_block);
        frame.render_widget(content, root[0]);

        // show the status bar
        let status_bar_block = Block::default()
            .title("Status Bar")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightMagenta));
        let status_bar  = Paragraph::new("OK")
            .block(status_bar_block);

        frame.render_widget(status_bar, root[1]);

        // show the cursor
        let editor_area = root[0];

        let cursor_x = editor_area.x + app.cursor_pos.0 as u16 + 1;
        let cursor_y = editor_area.y + app.cursor_pos.1 as u16 + 1;

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

