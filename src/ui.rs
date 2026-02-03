use ratatui::layout::{Rect, Constraint, Direction, Layout, Alignment, Flex};
use ratatui::style::{Modifier, Color, Style};
use ratatui::widgets::{Borders, Block, List, ListItem, Paragraph, ListDirection};
use ratatui::Frame;

use crate::app::{Screen, App};

pub fn ui(frame: &mut Frame, app: &App) {
     if let Screen::Welcome = app.current_screen {
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

