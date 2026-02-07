use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use crate::app::{App, Screen};
use crate::buffer::*;
use crate::command::*;
use crate::error::*;
use crate::layout::layout_manager::*;
use crate::layout::tree::*;
use crate::utils::*;

pub fn ui(frame: &mut Frame, app: &mut App) -> Result<(), RenderError> {
    match app.current_screen {
        Screen::Welcome => Ok({
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
                .constraints([Constraint::Length(8), Constraint::Length(5)])
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
                .style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                );

            frame.render_widget(list, content[1]);
        }),

        _ => {
            let root = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(frame.area());

            let lay_m = &mut app.layout_manager;

            let editor_rect = render_layout(
                &lay_m.panes.as_ref().ok_or(LayoutError::NoNode)?.clone(),
                root[0],
                frame,
                &app.buf_manager,
                &app.command,
                &mut app.layout_manager,
            )?;

            // command frame include kaomoji and command line
            let cmd = &mut app.command;
            let command_line_frame = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(8), Constraint::Min(3)])
                .split(root[1]);

            let viewport_width = command_line_frame[1].width.saturating_sub(2) as usize;
            cmd.update_scroll(viewport_width);
            // command frame's color
            let command_line_border_color_active = Color::Rgb(167, 250, 244);
            let command_line_border_color_no = Color::Rgb(83, 166, 165);
            let kaomoji_color_active = Color::Rgb(226, 230, 156);
            let kaomoji_color_no = Color::Rgb(174, 179, 82);
            let command_line_text_color_active = Color::Rgb(182, 238, 252);
            let command_line_text_color_no = Color::Rgb(138, 196, 212);

            // handle command frame's color
            let (command_line_border_color, kaomoji_color, command_line_text_color) =
                match app.current_screen {
                    Screen::Command => (
                        command_line_border_color_active,
                        kaomoji_color_active,
                        command_line_text_color_active,
                    ),
                    _ => (
                        command_line_border_color_no,
                        kaomoji_color_no,
                        command_line_text_color_no,
                    ),
                };

            // show kaomoji
            cmd.kmj = match app.current_screen {
                Screen::Command => match cmd.status {
                    CmdStatus::Success => KaoMoJi::Wink,
                    CmdStatus::Failed => KaoMoJi::Angry,
                    _ => KaoMoJi::Happy,
                },
                _ => KaoMoJi::Smile,
            };
            let kaomoji_block = Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
                .border_style(Style::default().fg(command_line_border_color))
                .style(Style::default().fg(kaomoji_color));
            let kaomoji = Paragraph::new(kaomoji_to_text(cmd.kmj.clone()))
                .alignment(Alignment::Center)
                .block(kaomoji_block);

            frame.render_widget(kaomoji, command_line_frame[0]);

            // show command line
            let command_line_block = Block::default()
                .borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM)
                .border_style(Style::default().fg(command_line_border_color))
                .style(Style::default().fg(command_line_text_color));

            let command_line = Paragraph::new(format!("{}: {}", cmd.say, cmd.content))
                .scroll((cmd.scroll_offset.1 as u16, 0))
                .block(command_line_block);

            frame.render_widget(command_line, command_line_frame[1]);

            // show the cursor
            match app.current_screen {
                Screen::Editor => {
                    if let Some(LayoutNode::Pane {
                        cursor_pos,
                        scroll_offset,
                        buffer_id,
                        ..
                    }) = app.layout_manager.get_current_pane()
                    {
                        if let Ok(buf) = app.buf_manager.get_buffer(buffer_id) {
                            let (cx, cy) = cursor_pos;

                            let vx = buf.get_visual_width_upto(cy, cx);

                            let cursor_x =
                                editor_rect.x + (vx.saturating_sub(scroll_offset.0)) as u16;
                            let cursor_y =
                                editor_rect.y + (cy.saturating_sub(scroll_offset.1)) as u16 + 1;

                            if cursor_x < editor_rect.right() && cursor_y < editor_rect.bottom() {
                                frame.set_cursor_position((cursor_x, cursor_y));
                            }
                        }
                    }
                    //                    let vx = buf.get_visual_width_upto(buf.cursor_pos.1, buf.cursor_pos.0);
                    //                    let cursor_x = editor_rect.x + (vx.saturating_sub(buf.scroll_offset.0)) as u16;
                    //                    let cursor_y = editor_rect.y
                    //                        + (buf.cursor_pos.1.saturating_sub(buf.scroll_offset.1)) as u16
                    //                        + 1;
                    //                    if cursor_x < editor_rect.right() && cursor_y < editor_rect.bottom() {
                    //                        frame.set_cursor_position((cursor_x, cursor_y));
                    //                    }
                    //
                }
                Screen::Command => {
                    let command_line_area = command_line_frame[1];
                    let vx = cmd.get_visual_width_upto(cmd.cursor_pos.0);
                    let cursor_x = command_line_area.x
                        + get_line_len(&cmd.say.to_string()) as u16
                        + (vx.saturating_sub(cmd.scroll_offset.0)) as u16
                        + 2;
                    let cursor_y = command_line_area.y
                        + (cmd.cursor_pos.1.saturating_sub(cmd.scroll_offset.1)) as u16
                        + 1;
                    if cursor_x < command_line_area.right() && cursor_y < command_line_area.bottom()
                    {
                        frame.set_cursor_position((cursor_x, cursor_y));
                    }
                }
                _ => {}
            };

            // show the cursor
            Ok(())
        }
    }
}

pub fn render_layout(
    node: &LayoutNode,
    area: Rect,
    f: &mut Frame,
    buf_m: &BufferManager,
    cmd: &KaoCo,
    layout_m: &mut LayoutManager,
) -> Result<Rect, RenderError> {
    let mut cur_rect = area;
    match node {
        LayoutNode::Pane { id, .. } => {
            let buf = buf_m.get_buffer(*id)?;
            let res_rect = render_buffer(&buf, area, f, layout_m, buf_m);
            if *id == layout_m.current_layout {
                cur_rect = res_rect?;
            }
        }
        LayoutNode::Split {
            direc,
            ratio,
            first,
            second,
        } => {
            let chunks = match direc {
                SplitDirection::Horizontal => Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage((ratio * 100.0) as u16),
                        Constraint::Percentage(((1.0 - ratio) * 100.0) as u16),
                    ])
                    .split(area),

                SplitDirection::Vertical => Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage((ratio * 100.0) as u16),
                        Constraint::Percentage(((1.0 - ratio) * 100.0) as u16),
                    ])
                    .split(area),
            };

            render_layout(first, chunks[0], f, buf_m, cmd, layout_m)?;
            render_layout(second, chunks[1], f, buf_m, cmd, layout_m)?;
        }
    };
    Ok(cur_rect)
}

pub fn render_buffer(
    buf: &Buffer,
    rect: Rect,
    frame: &mut Frame,
    layout_m: &mut LayoutManager,
    buf_m: &BufferManager,
) -> Result<Rect, LayoutError> {
    let cur_pane = layout_m
        .get_current_pane()
        .ok_or(LayoutError::PaneNotFound)?;
    let (cursor_pos, _ , scroll_offset) = match cur_pane {
        LayoutNode::Pane {
            cursor_pos,
            buffer_id,
            scroll_offset, 
            ..
        } => (cursor_pos, buffer_id, scroll_offset),
        _ => return Err(LayoutError::NotPane),
    };

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(rect);

    // editor frame include editor and status bar
    let editor_frame = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(root[0]);

    // show the editor
    let editor_main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Min(0)])
        .split(editor_frame[0]);

    // check the scrolling
    let viewport_height = editor_main[1].height.saturating_sub(2) as usize;
    let viewport_width = editor_main[1].width.saturating_sub(2) as usize;

    layout_m.check_cursor_pos(buf_m)?;
    layout_m.update_scroll(buf_m, viewport_height, viewport_width)?;
    // editor's color
    let border_color_active = Color::Rgb(181, 235, 181);
    let border_color_no = Color::Rgb(129, 181, 129);
    let font_color = Color::Rgb(240, 235, 213);

    // handle editor's color
    let border_color = if layout_m.current_layout == buf.id {
        border_color_active
    } else {
        border_color_no
    };

    let line_num_block = Block::default()
        .borders(Borders::LEFT | Borders::TOP)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().fg(Color::DarkGray));
    let total_lines = buf.content.len();
    let line_num: Vec<Line> = (1..=total_lines)
        .map(|n| {
            Line::from(Span::styled(
                format!("{:>3} ", n),
                Style::default().fg(Color::DarkGray),
            ))
        })
        .collect();
    let line_num_text = Paragraph::new(Text::from(line_num))
        .block(line_num_block)
        .scroll((scroll_offset.1 as u16, 0));

    frame.render_widget(line_num_text, editor_main[0]);

    let save_text = if buf.saved { "" } else { "[+] " };
    let editor_block = Block::default()
        .title(format!(" {} {}", buf.name.as_ref(), save_text))
        .borders(Borders::RIGHT | Borders::TOP)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().fg(font_color));

    let lines: Vec<Line> = buf.content.iter().map(|s| Line::from(s.as_str())).collect();

    let content = Paragraph::new(Text::from(lines))
        .block(editor_block)
        .scroll((scroll_offset.1 as u16, scroll_offset.0 as u16));

    frame.render_widget(content, editor_main[1]);

    // show the status bar
    let status_bar_main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(editor_frame[1]);

    let status_pos_font_color = Color::Rgb(252, 207, 248);
    let status_second_font_color = Color::Rgb(240, 172, 125);
    let status_third_font_color = Color::Rgb(240, 186, 89);
    let status_forth_font_color = Color::Rgb(210, 240, 105);
    let status_last_font_color = Color::Rgb(210, 240, 105);

    // show the status pos
    let pos = cursor_pos;
    let status_pos_block = Block::default()
        .borders(Borders::BOTTOM | Borders::LEFT)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().fg(status_pos_font_color));
    let status_pos = Paragraph::new(format!("{}:{}", pos.1 + 1, pos.0 + 1))
        .alignment(Alignment::Center)
        .block(status_pos_block);

    frame.render_widget(status_pos, status_bar_main[0]);

    // show the second secondition of status bar
    let status_second_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().fg(status_second_font_color));
    let status_second = Paragraph::new(format!("{}%", (pos.1 + 1) * 100 / buf.content.len()))
        .alignment(Alignment::Center)
        .block(status_second_block);

    frame.render_widget(status_second, status_bar_main[1]);

    // show the third position of status bar
    let buf_size = if let Some(info) = &buf.file_info {
        info.size
    } else {
        0
    };
    let status_third_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().fg(status_third_font_color));
    let status_third = Paragraph::new(format!("{} B", buf_size))
        .alignment(Alignment::Center)
        .block(status_third_block);

    frame.render_widget(status_third, status_bar_main[2]);

    // show the forth position of status bar
    let buf_fmt = if let Some(info) = &buf.file_info {
        get_format_text(info.format.clone())
    } else {
        "-"
    };

    let status_last_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().fg(status_forth_font_color));
    let status_last = Paragraph::new(buf_fmt)
        .alignment(Alignment::Center)
        .block(status_last_block);

    frame.render_widget(status_last, status_bar_main[3]);

    // show the last position of status bar
    let status_last_block = Block::default()
        .borders(Borders::BOTTOM | Borders::RIGHT)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().fg(status_last_font_color));
    let is_rd_only = if let Some(info) = &buf.file_info {
        info.read_only
    } else {
        true
    };
    let last_text = if is_rd_only { "READONLY" } else { "-" };
    let status_last = Paragraph::new(last_text)
        .alignment(Alignment::Center)
        .block(status_last_block);

    frame.render_widget(status_last, status_bar_main[4]);
    return Ok(editor_main[1]);
}

fn get_banner() -> String {
    r#"
███████╗███╗   ███╗██╗██╗     ███████╗
██╔════╝████╗ ████║██║██║     ██╔════╝
███████╗██╔████╔██║██║██║     █████╗  
╚════██║██║╚██╔╝██║██║██║     ██╔══╝  
███████║██║ ╚═╝ ██║██║███████╗███████╗
╚══════╝╚═╝     ╚═╝╚═╝╚══════╝╚══════╝
    "#
    .to_string()
}
