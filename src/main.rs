use ratatui::crossterm::event::{EnableMouseCapture, Event, KeyModifiers};
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use ratatui::crossterm::event::DisableMouseCapture;
use ratatui::crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::{CrosstermBackend, Backend};
use std::io;
use color_eyre::eyre::Result;
use clap::Parser;

mod app;
use app::{App, Screen};

mod buffer;
use buffer::*;

mod ui;
use ui::ui;

mod error;

mod cli;
use cli::Args;

mod kaomoji;

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
    let init_screen = if let Some(p) =  args.path {
        let buf = Buffer::from_file(p)?;
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
    <B as Backend>::Error: Sync + Send + 'static 
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
                Screen::Editor => match (key.modifiers, key.code) {
                    // exit
                    (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                        return Ok(());
                    }
                    // save file
                    (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                        if let Some(buf) = app.buf_manager.get_current_buffer_mut() {
                            if buf.path.is_some() {
                                let _ = buf.save(); 
                            } else {
                                let _ = buf.save_to("new_file.txt");
                            }
                        }
                    }
                    (_, KeyCode::Left) => app.mv_cursor_left(),
                    (_, KeyCode::Right) => app.mv_cursor_right(),
                    (_, KeyCode::Up) => app.mv_cursor_up(),
                    (_, KeyCode::Down) => app.mv_cursor_down(),
                    (KeyModifiers::NONE, KeyCode::Char(ch)) => {
                        if let Ok(_) = app.insert_char(ch) {
                            app.mv_cursor_right();
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Enter) => {
                        app.handle_enter();
                    }
                    (KeyModifiers::NONE, KeyCode::Backspace) => {
                        app.handle_backspace();
                    }
                    _ => {}
                },
                Screen::Popup => {
                    if key.code == KeyCode::Esc {
                        app.current_screen = Screen::Welcome;
                    }
                }
            }
        }
    }
}
