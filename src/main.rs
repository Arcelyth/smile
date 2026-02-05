use clap::Parser;
use color_eyre::eyre::Result;
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::DisableMouseCapture;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::crossterm::event::{EnableMouseCapture, Event, KeyModifiers};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use ratatui::crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use std::io;

mod app;
use app::{App, Screen};

mod buffer;
use buffer::*;

mod ui;
use ui::ui;

mod error;
use error::*;

mod cli;
use cli::Args;

mod utils;

mod command;

fn main() -> Result<()> {
    // setup terminal
    color_eyre::install()?;
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let args = Args::parse();
    let mut bm = BufferManager::new();
    let init_screen = if let Some(p) = args.path {
        let buf = match Buffer::from_file(p) {
            Err(e) => {
                match e {
                    BufferError::NotAFile => {
                        println!("The input is not a file.");
                    }
                    _ => println!("Unknown Error"),
                }
                return Ok(());
            }
            Ok(b) => b,
        };
        bm.add_buffer(buf);
        Screen::Editor
    } else {
        Screen::Welcome
    };
    // create app and run
    let mut app = App::from(init_screen, bm);
    run_app(&mut terminal, &mut app)?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    <B as Backend>::Error: Sync + Send + 'static,
{
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match app.current_screen {
                Screen::Welcome => match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('a') => {
                        app.buf_manager.add_new_buffer("Untitled");
                        app.current_screen = Screen::Editor;
                    }
                    _ => {}
                },
                Screen::Editor => {
                    let cur_buf = if let Some(b) = app.buf_manager.get_current_buffer_mut() {
                        b
                    } else {
                        return Ok(());
                    };

                    match (key.modifiers, key.code) {
                        // exit
                        (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                            return Ok(());
                        }
                        // save file
                        (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                            if cur_buf.path.is_some() {
                                let _ = cur_buf.save();
                            } else {
                                let _ = cur_buf.save_to("new_file.txt");
                            }
                        }
                        // active the command line
                        (KeyModifiers::CONTROL, KeyCode::Char('x')) => {
                            app.current_screen = Screen::Command;
                        }

                        (_, KeyCode::Left) => cur_buf.mv_cursor_left(),
                        (_, KeyCode::Right) => cur_buf.mv_cursor_right(),
                        (_, KeyCode::Up) => cur_buf.mv_cursor_up(),
                        (_, KeyCode::Down) => cur_buf.mv_cursor_down(),
                        (KeyModifiers::NONE, KeyCode::Char(ch)) => {
                            if let Ok(_) = cur_buf.add_content_at(ch.to_string().as_str()) {
                                cur_buf.mv_cursor_right();
                            }
                        }
                        (KeyModifiers::NONE, KeyCode::Enter) => {
                            cur_buf.handle_enter();
                        }
                        (KeyModifiers::NONE, KeyCode::Backspace) => {
                            cur_buf.handle_backspace();
                        }
                        _ => {}
                    }
                }
                Screen::Command => {
                    let cur_cmd = &mut app.command;
                    match (key.modifiers, key.code) {
                        // exit
                        (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                            cur_cmd.clean();
                            app.current_screen = Screen::Editor
                        }
                        (KeyModifiers::NONE, KeyCode::Char(ch)) => {
                            if let Ok(_) = cur_cmd.add_content_at(ch.to_string().as_str()) {
                                cur_cmd.mv_cursor_right();
                            }
                        }
                        (_, KeyCode::Left) => cur_cmd.mv_cursor_left(),
                        (_, KeyCode::Right) => cur_cmd.mv_cursor_right(),
                        (KeyModifiers::NONE, KeyCode::Enter) => {
                            cur_cmd.handle_command();
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
