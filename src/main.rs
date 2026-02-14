use clap::Parser;
use color_eyre::eyre::Result;
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::cursor::SetCursorStyle;
use ratatui::crossterm::event::DisableMouseCapture;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::crossterm::event::{EnableMouseCapture, Event, KeyModifiers};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use ratatui::crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use std::io;

mod app;
use app::{App, Mod, Screen};

mod buffer;

mod ui;
use ui::ui;

mod error;
use error::*;

mod cli;
use cli::Args;

mod utils;

mod command;
use command::instructions::*;
use command::*;

mod layout;
use layout::layout_manager::MoveDir;

mod cursor;
mod popup;

fn main() -> Result<()> {
    // setup terminal
    color_eyre::install()?;
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let args = Args::parse();
    let mut app = App::new();
    // initialize
    if let Some(_) = args.path {
        // todo: error handle
        app.init(args.path).unwrap();
    } else {
        app.current_screen = Screen::Welcome;
    };

    run_app(&mut terminal, &mut app)?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        SetCursorStyle::DefaultUserShape
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    <B as Backend>::Error: Sync + Send + 'static,
{
    loop {
        // todo: error handle
        terminal.draw(|f| match ui(f, app) {
            Ok(_) => {}
            Err(e) => match e {
                RenderError::LayoutErr(LayoutError::NoNode) => {
                    app.should_exit = true;
                }
                error => {
                    println!("{:?}", error)
                }
            },
        })?;

        if app.should_exit {
            break Ok(());
        }
        app.popups.update();
        let cur_cmd = &mut app.command;
        let cur_screen = &mut app.current_screen;
        let layout_m = &mut app.layout_manager;
        let buffer_m = &mut app.buf_manager;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match *cur_screen {
                Screen::Welcome => match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('a') => {
                        // todo: handle error
                        app.init(None).unwrap();
                    }
                    _ => {}
                },
                Screen::Editor => {
                    match app.current_mod {
                        Mod::Visual(vx, vy) => match (key.modifiers, key.code) {
                            (KeyModifiers::NONE, KeyCode::Esc) => {
                                app.current_mod = Mod::Input;
                            }
                            // move cursor to the head of line
                            (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
                                mv_cursor_head(layout_m)?;
                            }
                            // move cursor to the end of line
                            (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
                                mv_cursor_tail(buffer_m, layout_m)?;
                            }
                            (_, KeyCode::Char('d')) => {
                                cur_cmd.handle_instructions(
                                    buffer_m,
                                    layout_m,
                                    Instruction::DeleteBlock((vx, vy)),
                                )?;
                                app.current_mod = Mod::Input;
                            }
                            (_, KeyCode::Left) => mv_cursor_left(buffer_m, layout_m)?,
                            (_, KeyCode::Right) => mv_cursor_right(buffer_m, layout_m, 1)?,
                            (_, KeyCode::Up) => mv_cursor_up(buffer_m, layout_m)?,
                            (_, KeyCode::Down) => mv_cursor_down(buffer_m, layout_m)?,
                            _ => {}
                        },
                        _ => {
                            match (key.modifiers, key.code) {
                                // exit
                                (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                                    close_current_pane(
                                        cur_cmd,
                                        buffer_m,
                                        layout_m,
                                        &mut app.should_exit,
                                        &mut app.current_screen,
                                    )?;
                                }
                                // save file
                                (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                                    // todo: error handle
                                    if is_buffer_binding(buffer_m, layout_m).unwrap() {
                                        let _ = save(buffer_m, layout_m);
                                    } else {
                                        app.current_screen = Screen::Command;
                                        let _ = cur_cmd.ask_and_save();
                                    }
                                }
                                // revoke
                                (KeyModifiers::CONTROL, KeyCode::Char('z')) => {
                                    revoke(buffer_m, layout_m)?;
                                }
                                // move cursor to the head of line
                                (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
                                    mv_cursor_head(layout_m)?;
                                }
                                // move cursor to the end of line
                                (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
                                    mv_cursor_tail(buffer_m, layout_m)?;
                                }
                                // move cursor to the head of the next word
                                (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
                                    mv_cursor_next_word_head(buffer_m, layout_m)?;
                                }
                                // move cursor to the head of the prev word
                                (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                                    mv_cursor_prev_word_head(buffer_m, layout_m)?;
                                }
                                // active the command line
                                (KeyModifiers::CONTROL, KeyCode::Char('x')) => {
                                    app.current_screen = Screen::Command;
                                    cur_cmd.clean_all();
                                }
                                // delete current line
                                (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                                    cur_cmd.handle_instructions(
                                        buffer_m,
                                        layout_m,
                                        Instruction::DeleteLine,
                                    )?;
                                }
                                // move to the left pane
                                (KeyModifiers::CONTROL, KeyCode::Left) => {
                                    move_focus_in_pane(layout_m, MoveDir::Left);
                                }
                                // move to the right pane
                                (KeyModifiers::CONTROL, KeyCode::Right) => {
                                    move_focus_in_pane(layout_m, MoveDir::Right);
                                }
                                // move to the up pane
                                (KeyModifiers::CONTROL, KeyCode::Up) => {
                                    move_focus_in_pane(layout_m, MoveDir::Up);
                                }
                                // move to the down pane
                                (KeyModifiers::CONTROL, KeyCode::Down) => {
                                    move_focus_in_pane(layout_m, MoveDir::Down);
                                }
                                // change to visual mode
                                (KeyModifiers::CONTROL, KeyCode::Char('v')) => {
                                    enter_visual(layout_m, &mut app.current_mod)?;
                                }
                                // enter Tab
                                (_, KeyCode::Tab) => {
                                    if let Ok(_) = cur_cmd.handle_instructions(
                                        buffer_m,
                                        layout_m,
                                        Instruction::InsertText("    ".to_string().into()),
                                    ) {
                                        mv_cursor_right(buffer_m, layout_m, 4)?;
                                    }
                                }
                                // enter Home
                                (_, KeyCode::Home) => {
                                    mv_cursor_head(layout_m)?;
                                }
                                // enter End
                                (_, KeyCode::End) => {
                                    mv_cursor_tail(buffer_m, layout_m)?;
                                }

                                (_, KeyCode::Left) => mv_cursor_left(buffer_m, layout_m)?,
                                (_, KeyCode::Right) => mv_cursor_right(buffer_m, layout_m, 1)?,
                                (_, KeyCode::Up) => mv_cursor_up(buffer_m, layout_m)?,
                                (_, KeyCode::Down) => mv_cursor_down(buffer_m, layout_m)?,
                                (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(ch)) => {
                                    if let Ok(_) = cur_cmd.handle_instructions(
                                        buffer_m,
                                        layout_m,
                                        Instruction::InsertText(ch.to_string().into()),
                                    ) {
                                        mv_cursor_right(buffer_m, layout_m, 1)?;
                                    }
                                }
                                (KeyModifiers::NONE, KeyCode::Enter) => cur_cmd
                                    .handle_instructions(
                                        buffer_m,
                                        layout_m,
                                        Instruction::InsertLine,
                                    )?,
                                (KeyModifiers::NONE, KeyCode::Backspace) => {
                                    cur_cmd.handle_instructions(
                                        buffer_m,
                                        layout_m,
                                        Instruction::DeleteText(1),
                                    )?;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Screen::Command => {
                    match (key.modifiers, key.code) {
                        // exit
                        (KeyModifiers::CONTROL, KeyCode::Char('q'))
                        | (KeyModifiers::NONE, KeyCode::Esc) => {
                            cur_cmd.clean_all();
                            app.current_screen = Screen::Editor
                        }
                        (KeyModifiers::NONE, KeyCode::Char(ch)) => {
                            if !matches!(cur_cmd.status, CmdStatus::Exec(_)) {
                                cur_cmd.status = CmdStatus::Normal;
                            }

                            if let Ok(_) = cur_cmd.add_content_at(ch.to_string().as_str()) {
                                cur_cmd.mv_cursor_right();
                            }
                        }
                        (_, KeyCode::Left) => cur_cmd.mv_cursor_left(),
                        (_, KeyCode::Right) => cur_cmd.mv_cursor_right(),
                        (KeyModifiers::NONE, KeyCode::Enter) => {
                            let ret = cur_cmd
                                .handle_command(
                                    buffer_m,
                                    layout_m,
                                    cur_screen,
                                    &mut app.popups,
                                    &mut app.should_exit,
                                )
                                .unwrap();
                            if ret {
                                app.current_screen = Screen::Editor
                            }
                            cur_cmd.clean();
                        }
                        (KeyModifiers::NONE, KeyCode::Backspace) => {
                            cur_cmd.handle_backspace();
                        }

                        _ => {}
                    }
                }
            }
        }
    }
}
