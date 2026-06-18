mod prettify;
mod state;
mod task_manager;
mod ui;
mod vfs;

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
    let vfs = LocalVfs;
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
                                state.navigate_up_directory(vfs).await?;
                            }
                            KeyCode::Char(' ') => {
                                state.active_pane_mut().toggle_selection();
                            }
                            KeyCode::Delete | KeyCode::Char('d') => {
                                state.mode = InputMode::DeleteConfirm;
                            }
                            KeyCode::F(5) | KeyCode::Char('c') => {
                                state.copy_selected(vfs).await?;
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
                        InputMode::FileViewer => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                let is_dirty = state.file_viewer.as_ref().map(|v| v.is_dirty).unwrap_or(false);
                                if is_dirty {
                                    state.mode = InputMode::FileViewerSavePrompt;
                                } else {
                                    state.close_file_viewer();
                                }
                            }
                            KeyCode::Char('p') => {
                                state.prettify_current_file()?;
                            }
                            KeyCode::Char('e') => {
                                state.open_in_editor();
                            }
                            KeyCode::Up => {
                                state.scroll_viewer_up(1);
                            }
                            KeyCode::Down => {
                                let height = terminal.size().map(|s| s.height).unwrap_or(24);
                                let visible_height = (height.saturating_sub(10) as usize).max(1);
                                state.scroll_viewer_down(1, visible_height);
                            }
                            KeyCode::PageUp => {
                                let height = terminal.size().map(|s| s.height).unwrap_or(24);
                                let visible_height = (height.saturating_sub(10) as usize).max(1);
                                state.scroll_viewer_up(visible_height.saturating_sub(2).max(1));
                            }
                            KeyCode::PageDown => {
                                let height = terminal.size().map(|s| s.height).unwrap_or(24);
                                let visible_height = (height.saturating_sub(10) as usize).max(1);
                                state.scroll_viewer_down(
                                    visible_height.saturating_sub(2).max(1),
                                    visible_height,
                                );
                            }
                            _ => {}
                        },
                        InputMode::FileViewerSavePrompt => match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                state.save_viewer_content(vfs).await?;
                                state.close_file_viewer();
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                state.close_file_viewer();
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                state.mode = InputMode::FileViewer;
                            }
                            _ => {}
                        },
                        InputMode::DeleteConfirm => match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                state.delete_selected(vfs).await?;
                                state.mode = InputMode::Normal;
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                state.mode = InputMode::Normal;
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
