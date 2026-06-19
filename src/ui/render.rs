use crate::state::{AppStateManager, InputMode, PaneState};
use crate::vfs::FileType;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, state: &AppStateManager) {
    let size = f.size();

    // Vertical layout: Header (ASCII Art / Title) -> Content (Dual Pane) -> Footer (Status / Key Help)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Header with ASCII banner
            Constraint::Min(3),    // Main dual pane view
            Constraint::Length(1), // Footer status/key help
        ])
        .split(size);

    render_header(f, chunks[0]);
    render_panes(f, chunks[1], state);
    render_footer(f, chunks[2], state);

    if state.mode == InputMode::GoToPath {
        render_go_to_popup(f, size, state);
    } else if state.mode == InputMode::FileViewer {
        render_file_viewer(f, size, state);
    } else if state.mode == InputMode::FileViewerSavePrompt {
        render_file_viewer(f, size, state);
        render_save_prompt_popup(f, size);
    } else if state.mode == InputMode::DeleteConfirm {
        render_delete_confirm_popup(f, size, state);
    } else if state.mode == InputMode::CreateFolder {
        render_create_folder_popup(f, size, state);
    } else if state.mode == InputMode::CreateFile {
        render_create_file_popup(f, size, state);
    } else if state.mode == InputMode::RenameOrMove {
        render_rename_or_move_popup(f, size, state);
    }
}

fn render_header(f: &mut Frame, area: Rect) {
    let ascii_art = vec![
        r#"  ____             _    _         ____                                          _"#,
        r#" / ___|___   ___  | | _(_) ___   / ___|___  _ __ ___  _ __ ___   __ _ _ __   __| | ___ _ __"#,
        r#"| |   / _ \ / _ \ | |/ / |/ _ \ | |   / _ \| '_ ` _ \| '_ ` _ \ / _` | '_ \ / _` |/ _ \ '__|"#,
        r#"| |__| (_) | (_) ||   <| |  __/ | |__| (_) | | | | | | | | | | | (_| | | | | (_| |  __/ |"#,
        r#" \____\___/ \___/ |_|\_\_|\___|  \____\___/|_| |_| |_|_| |_| |_|\__,_|_| |_|\__,_|\___|_|"#,
    ];

    let header_content = if area.width >= 100 {
        let max_len = ascii_art.iter().map(|s| s.len()).max().unwrap_or(0);
        let mut lines: Vec<Line> = ascii_art
            .into_iter()
            .map(|l| {
                let padded = format!("{:width$}", l, width = max_len);
                Line::from(Span::styled(
                    padded,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            })
            .collect();
        lines.push(Line::from(Span::styled(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        )));
        lines
    } else {
        vec![
            Line::from(vec![
                Span::styled(
                    "CookieCommander",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" v{}", env!("CARGO_PKG_VERSION")),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(Span::styled(
                "Orthodox File Manager",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    };

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));

    let header_paragraph = Paragraph::new(header_content)
        .block(header_block)
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(header_paragraph, area);
}

fn render_panes(f: &mut Frame, area: Rect, state: &AppStateManager) {
    let pane_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_pane(f, pane_chunks[0], &state.left_pane, state.active_left);
    render_pane(f, pane_chunks[1], &state.right_pane, !state.active_left);
}

fn render_pane(f: &mut Frame, area: Rect, pane: &PaneState, is_active: bool) {
    let border_color = if is_active {
        Color::LightGreen
    } else {
        Color::DarkGray
    };

    let title_style = if is_active {
        Style::default()
            .fg(Color::LightGreen)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let pane_title = format!(" Pane: {} ", pane.current_path);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(pane_title, title_style));

    let inner_area = block.inner(area);

    let items: Vec<ListItem> = pane
        .entries
        .iter()
        .map(|entry| {
            let (prefix, color, is_dir) = match entry.file_type {
                FileType::Directory => ("📁 ", Color::Blue, true),
                FileType::Symlink => ("🔗 ", Color::Magenta, false),
                _ => ("📄 ", Color::White, false),
            };

            let is_selected = pane.selections.contains(&entry.name);
            let display_prefix = if is_selected {
                format!("✔ {}", prefix)
            } else {
                format!("  {}", prefix)
            };

            let name_style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color).add_modifier(if is_dir {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                })
            };

            let name_span = Span::styled(
                format!("{}{}", display_prefix, entry.name),
                name_style,
            );

            // Calculate spacing to push size to the right side of the pane
            let width = inner_area.width as usize;
            let size_str = format_size(entry.size, is_dir);
            let display_name_len = display_prefix.chars().count() + entry.name.chars().count();

            let spaces = if width > display_name_len + size_str.len() + 2 {
                width - display_name_len - size_str.len() - 2
            } else {
                1
            };

            let space_span = Span::raw(" ".repeat(spaces));
            let size_span = Span::styled(size_str, Style::default().fg(Color::DarkGray));

            ListItem::new(Line::from(vec![name_span, space_span, size_span]))
        })
        .collect();

    let highlight_style = if is_active {
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight_style);

    let mut list_state = ListState::default();
    if !pane.entries.is_empty() {
        list_state.select(Some(pane.selected_index));
    }

    f.render_stateful_widget(list, area, &mut list_state);
}

fn format_size(bytes: u64, is_dir: bool) -> String {
    if is_dir {
        return "  <DIR>".to_string();
    }
    if bytes == 0 {
        return "    0 B".to_string();
    }
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, units[unit_idx])
}

fn render_footer(f: &mut Frame, area: Rect, state: &AppStateManager) {
    let text = if let Some(msg) = &state.status_message {
        Line::from(vec![
            Span::styled(
                " STATUS: ",
                Style::default()
                    .bg(Color::Red)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(msg, Style::default().fg(Color::LightRed)),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                " Tab ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Switch  "),
            Span::styled(
                " Space ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Select  "),
            Span::styled(
                " F5/c ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Copy  "),
            Span::styled(
                " F6 ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Move/Rename  "),
            Span::styled(
                " F7 ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" NewFolder  "),
            Span::styled(
                " n ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" NewFile  "),
            Span::styled(
                " Del/d ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Delete  "),
            Span::styled(
                " Enter ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Open  "),
            Span::styled(
                " g ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" GoTo  "),
            Span::styled(
                " Esc/q ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Quit"),
        ])
    };

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, area);
}

fn render_go_to_popup(f: &mut Frame, area: Rect, state: &AppStateManager) {
    let popup_area = centered_rect(60, 20, area);

    let popup_block = Block::default()
        .title(Span::styled(
            " Go to Directory Path ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let input_para = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::LightGreen)),
            Span::raw(&state.input_buffer),
            Span::styled("█", Style::default().fg(Color::LightGreen)), // simulated blinking cursor
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to navigate | Esc to cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(popup_block);

    f.render_widget(Clear, popup_area); // clears the background behind popup
    f.render_widget(input_para, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_file_viewer(f: &mut Frame, area: Rect, state: &AppStateManager) {
    let viewer = match &state.file_viewer {
        Some(v) => v,
        None => return,
    };

    let viewer_area = centered_rect(95, 95, area);
    f.render_widget(Clear, viewer_area); // Clears the background

    // Split the viewer area into Content and a small Footer help bar
    let viewer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(1), // Help bar
        ])
        .split(viewer_area);

    let total_lines = viewer.lines.len();

    // Main block for the content
    let block = Block::default()
        .title(Span::styled(
            format!(" File Viewer: {} ", viewer.file_name),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner_area = block.inner(viewer_chunks[0]);
    let height = inner_area.height as usize;

    let start = viewer.scroll_offset;
    let end = (start + height).min(total_lines);

    let display_lines: Vec<Line> = viewer.lines[start..end]
        .iter()
        .map(|line| Line::from(line.as_str()))
        .collect();

    let paragraph = if total_lines == 0 {
        Paragraph::new(vec![Line::from(Span::styled(
            "~ Empty File ~",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ))])
    } else {
        Paragraph::new(display_lines)
    };

    f.render_widget(paragraph.block(block), viewer_chunks[0]);

    // Help bar rendering
    let percentage = (end * 100).checked_div(total_lines).unwrap_or(100);
    let scroll_info = format!(
        "Line {}-{} of {} ({}%)",
        if total_lines == 0 { 0 } else { start + 1 },
        end,
        total_lines,
        percentage
    );

    let path = std::path::Path::new(&viewer.file_path);
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut help_spans = vec![
        Span::styled(
            " Esc/q ",
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Close Viewer  "),
        Span::styled(
            " e ",
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Edit  "),
    ];

    if crate::prettify::get_prettifier(&ext).is_some() {
        help_spans.push(Span::styled(
            " p ",
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ));
        help_spans.push(Span::raw(" Prettify  "));
    }

    help_spans.extend(vec![
        Span::styled(
            " Up/Down ",
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Scroll Line  "),
        Span::styled(
            " PgUp/PgDn ",
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Scroll Page  "),
        Span::styled(
            format!("  {}  ", scroll_info),
            Style::default().fg(Color::LightCyan),
        ),
        Span::styled(
            format!("  {}  ", viewer.file_path),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let help_line = Line::from(help_spans);

    f.render_widget(Paragraph::new(help_line), viewer_chunks[1]);
}

fn render_save_prompt_popup(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(50, 15, area);

    let popup_block = Block::default()
        .title(Span::styled(" Save Confirmation ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let prompt_para = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled("  Save prettified JSON changes?", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("  Press: [y] Yes | [n] No | [Esc] Cancel", Style::default().fg(Color::DarkGray))),
    ])
    .block(popup_block);

    f.render_widget(Clear, popup_area); // Clears the background behind the popup
    f.render_widget(prompt_para, popup_area);
}

fn render_delete_confirm_popup(f: &mut Frame, area: Rect, state: &AppStateManager) {
    let popup_area = centered_rect(50, 15, area);

    let popup_block = Block::default()
        .title(Span::styled(" Delete Confirmation ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let active = state.active_pane();
    let message = if !active.selections.is_empty() {
        format!("  Delete {} selected item(s)?", active.selections.len())
    } else if let Some(entry) = active.selected_entry() {
        if entry.name != ".." {
            format!("  Delete {}?", entry.name)
        } else {
            "  No item selected to delete.".to_string()
        }
    } else {
        "  No item selected to delete.".to_string()
    };

    let prompt_para = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("  Press: [y] Yes | [n] No | [Esc] Cancel", Style::default().fg(Color::DarkGray))),
    ])
    .block(popup_block);

    f.render_widget(Clear, popup_area);
    f.render_widget(prompt_para, popup_area);
}

fn render_create_folder_popup(f: &mut Frame, area: Rect, state: &AppStateManager) {
    let popup_area = centered_rect(60, 20, area);

    let popup_block = Block::default()
        .title(Span::styled(
            " Create New Folder ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let input_para = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::LightGreen)),
            Span::raw(&state.input_buffer),
            Span::styled("█", Style::default().fg(Color::LightGreen)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to create | Esc to cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(popup_block);

    f.render_widget(Clear, popup_area);
    f.render_widget(input_para, popup_area);
}

fn render_create_file_popup(f: &mut Frame, area: Rect, state: &AppStateManager) {
    let popup_area = centered_rect(60, 20, area);

    let popup_block = Block::default()
        .title(Span::styled(
            " Create New File ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let input_para = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::LightGreen)),
            Span::raw(&state.input_buffer),
            Span::styled("█", Style::default().fg(Color::LightGreen)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to create empty file | Esc to cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(popup_block);

    f.render_widget(Clear, popup_area);
    f.render_widget(input_para, popup_area);
}

fn render_rename_or_move_popup(f: &mut Frame, area: Rect, state: &AppStateManager) {
    let popup_area = centered_rect(65, 20, area);

    let title = if state.rename_target.is_some() {
        " Rename / Move File or Folder "
    } else {
        " Move Selected Items "
    };

    let popup_block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let input_para = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::LightGreen)),
            Span::raw(&state.input_buffer),
            Span::styled("█", Style::default().fg(Color::LightGreen)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to execute | Esc to cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(popup_block);

    f.render_widget(Clear, popup_area);
    f.render_widget(input_para, popup_area);
}
