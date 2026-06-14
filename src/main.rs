mod vfs;
mod ui;
mod state;
mod task_manager;

use crate::state::{AppStateManager, InputMode};
use crate::vfs::LocalVfs;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Get initial paths
    let current_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("/"))
        .to_string_lossy()
        .to_string();

    // 2. Initialize VFS and State
    let vfs = LocalVfs::default();
    let mut state = AppStateManager::new(current_dir.clone(), current_dir);
    state.init(&vfs).await?;

    // 3. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 4. Main Event Loop
    let loop_res = run_app(&mut terminal, &mut state, &vfs).await;

    // 5. Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Print error if the loop failed
    if let Err(e) = loop_res {
        eprintln!("Application Error: {:?}", e);
    } else {
        println!("CookieCommander shutdown cleanly.");
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: &mut AppStateManager,
    vfs: &LocalVfs,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render::render(f, state))?;

        // Wait for event
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Ensure we only process KeyPress events on mac/linux/windows to avoid double execution
                if key.kind == KeyEventKind::Press {
                    match state.mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                break;
                            }
                            KeyCode::Tab => {
                                state.switch_pane();
                            }
                            KeyCode::Char('g') => {
                                state.start_go_to_path();
                            }
                            KeyCode::Up => {
                                state.active_pane_mut().select_prev();
                            }
                            KeyCode::Down => {
                                state.active_pane_mut().select_next();
                            }
                            KeyCode::Enter => {
                                state.handle_enter(vfs).await?;
                            }
                            KeyCode::Backspace => {
                                // Navigate up standard shortcut
                                let mut active_pane = state.active_pane_mut().clone();
                                active_pane.selected_index = 0;
                                // We simulate entering ".." by setting selected_index to 0
                                // since ".." is always the first entry if it exists.
                                if let Some(first_entry) = active_pane.entries.first() {
                                    if first_entry.name == ".." {
                                        state.active_pane_mut().selected_index = 0;
                                        state.handle_enter(vfs).await?;
                                    }
                                }
                            }
                            _ => {}
                        },
                        InputMode::GoToPath => match key.code {
                            KeyCode::Enter => {
                                state.commit_go_to_path(vfs).await?;
                            }
                            KeyCode::Esc => {
                                state.cancel_input();
                            }
                            KeyCode::Backspace => {
                                state.input_buffer.pop();
                            }
                            KeyCode::Char(c) => {
                                state.input_buffer.push(c);
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }
    Ok(())
}
