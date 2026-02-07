use crate::config::{self, Config};
use crate::render;
use crate::segments;
use crate::themes;
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use std::io::{self, Stdout};

#[derive(Debug, Clone, Copy)]
pub enum MainMenuAction {
    Render,
    Configure,
    Init,
    Check,
    Patch,
    Exit,
}

#[derive(Debug, Clone, Copy)]
enum Focus {
    Themes,
    Segments,
    Actions,
}

pub fn run_main_menu() -> Result<MainMenuAction> {
    let mut guard = TerminalGuard::new()?;
    let mut selected = 0usize;
    let items = [
        ("Render Statusline", "Render one-line status output now"),
        ("Open Configurator", "Enter full TUI config editor"),
        ("Init Config", "Create default config and themes"),
        ("Check Config", "Validate current configuration"),
        (
            "Patch Diagnostics",
            "Run Codex patch compatibility diagnostics",
        ),
        ("Exit", "Quit without action"),
    ];

    loop {
        guard.terminal.draw(|frame| {
            let area = frame.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5),
                    Constraint::Min(8),
                    Constraint::Length(3),
                ])
                .split(area);

            let header = Paragraph::new(vec![
                Line::from(Span::styled(
                    "codexline",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "Interactive Main Menu",
                    Style::default().fg(Color::Gray),
                )),
            ])
            .block(Block::default().borders(Borders::ALL).title("Welcome"));
            frame.render_widget(header, chunks[0]);

            let list_items: Vec<ListItem> = items
                .iter()
                .map(|(title, desc)| {
                    ListItem::new(Line::from(vec![
                        Span::styled(*title, Style::default().fg(Color::White)),
                        Span::raw(" - "),
                        Span::styled(*desc, Style::default().fg(Color::DarkGray)),
                    ]))
                })
                .collect();

            let mut state = ListState::default();
            state.select(Some(selected));
            let list = List::new(list_items)
                .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black))
                .highlight_symbol("▶ ")
                .block(Block::default().borders(Borders::ALL).title("Actions"));
            frame.render_stateful_widget(list, chunks[1], &mut state);

            let help = Paragraph::new("↑/↓ select, Enter confirm, q exit")
                .block(Block::default().borders(Borders::ALL).title("Help"));
            frame.render_widget(help, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    if selected == 0 {
                        selected = items.len() - 1;
                    } else {
                        selected = selected.saturating_sub(1);
                    }
                }
                KeyCode::Down => {
                    selected = (selected + 1) % items.len();
                }
                KeyCode::Enter => {
                    return Ok(match selected {
                        0 => MainMenuAction::Render,
                        1 => MainMenuAction::Configure,
                        2 => MainMenuAction::Init,
                        3 => MainMenuAction::Check,
                        4 => MainMenuAction::Patch,
                        _ => MainMenuAction::Exit,
                    });
                }
                KeyCode::Esc => return Ok(MainMenuAction::Exit),
                KeyCode::Char(c) => {
                    if c.to_string().eq_ignore_ascii_case("q") {
                        return Ok(MainMenuAction::Exit);
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn run_configurator(base: &Config) -> Result<Option<Config>> {
    let mut guard = TerminalGuard::new()?;

    let themes_dir = config::themes_dir();
    let mut theme_names = themes::list_theme_names(&themes_dir)?;
    if theme_names.is_empty() {
        theme_names.push("default".to_string());
    }

    let mut theme_index = theme_names
        .iter()
        .position(|name| name == &base.theme)
        .unwrap_or(0);

    let mut base_config = base.clone();
    let mut selected_segment = 0usize;
    let mut selected_action = 0usize;
    let mut focus = Focus::Segments;
    let mut footer_message = String::from("Tab switch focus, Space toggle segment, J/K reorder, Enter run action, S save, R reset, Q quit");

    let actions = ["Save", "Reset", "Quit"];

    loop {
        let preview_config =
            themes::apply_theme(&base_config, &theme_names[theme_index], &themes_dir)
                .unwrap_or_else(|_| base_config.clone());
        let preview_context = crate::collect::collect(&preview_config)?.context;
        let preview_segments = segments::build_segments(&preview_config, &preview_context);
        let preview_text = render::render_line(&preview_config, &preview_segments, true);

        guard.terminal.draw(|frame| {
            let area = frame.size();
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(5),
                ])
                .split(area);

            let header = Paragraph::new(vec![Line::from(vec![
                Span::styled(
                    "codexline config",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("theme: {}", theme_names[theme_index]),
                    Style::default().fg(Color::Yellow),
                ),
            ])])
            .block(Block::default().borders(Borders::ALL).title("Configurator"));
            frame.render_widget(header, rows[0]);

            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(28),
                    Constraint::Percentage(44),
                    Constraint::Percentage(28),
                ])
                .split(rows[1]);

            let theme_items: Vec<ListItem> = theme_names
                .iter()
                .map(|name| ListItem::new(name.as_str()))
                .collect();
            let mut theme_state = ListState::default();
            theme_state.select(Some(theme_index));
            let theme_list = List::new(theme_items)
                .block(Block::default().borders(Borders::ALL).title(
                    if matches!(focus, Focus::Themes) {
                        "Themes *"
                    } else {
                        "Themes"
                    },
                ))
                .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
            frame.render_stateful_widget(theme_list, cols[0], &mut theme_state);

            let segment_items: Vec<ListItem> = base_config
                .segments
                .iter()
                .map(|segment| {
                    let mark = if segment.enabled { "[x]" } else { "[ ]" };
                    let label = format!("{} {:?}", mark, segment.id);
                    ListItem::new(label)
                })
                .collect();
            let mut segment_state = ListState::default();
            if !base_config.segments.is_empty() {
                segment_state.select(Some(selected_segment.min(base_config.segments.len() - 1)));
            }
            let segment_list = List::new(segment_items)
                .block(Block::default().borders(Borders::ALL).title(
                    if matches!(focus, Focus::Segments) {
                        "Segments *"
                    } else {
                        "Segments"
                    },
                ))
                .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black));
            frame.render_stateful_widget(segment_list, cols[1], &mut segment_state);

            let action_items: Vec<ListItem> = actions.iter().map(|v| ListItem::new(*v)).collect();
            let mut action_state = ListState::default();
            action_state.select(Some(selected_action));
            let action_list = List::new(action_items)
                .block(Block::default().borders(Borders::ALL).title(
                    if matches!(focus, Focus::Actions) {
                        "Actions *"
                    } else {
                        "Actions"
                    },
                ))
                .highlight_style(Style::default().bg(Color::Magenta).fg(Color::White));
            frame.render_stateful_widget(action_list, cols[2], &mut action_state);

            let footer = Paragraph::new(vec![
                Line::from(Span::styled(
                    format!("Preview: {}", preview_text),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::styled(
                    footer_message.as_str(),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(Block::default().borders(Borders::ALL).title("Preview"));
            frame.render_widget(Clear, rows[2]);
            frame.render_widget(footer, rows[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            if handle_global_key(&key, &mut focus) {
                continue;
            }

            match focus {
                Focus::Themes => {
                    if handle_theme_keys(&key, &mut theme_index, theme_names.len()) {
                        continue;
                    }
                }
                Focus::Segments => {
                    if handle_segment_keys(&key, &mut base_config, &mut selected_segment) {
                        continue;
                    }
                }
                Focus::Actions => {
                    if handle_action_nav(&key, &mut selected_action, actions.len()) {
                        continue;
                    }
                }
            }

            match key.code {
                KeyCode::Enter => {
                    if matches!(focus, Focus::Actions) {
                        match actions[selected_action] {
                            "Save" => {
                                let merged = themes::apply_theme(
                                    &base_config,
                                    &theme_names[theme_index],
                                    &themes_dir,
                                )?;
                                config::save(&merged)?;
                                return Ok(Some(merged));
                            }
                            "Reset" => {
                                base_config = base.clone();
                                theme_index = theme_names
                                    .iter()
                                    .position(|name| name == &base.theme)
                                    .unwrap_or(0);
                                selected_segment = 0;
                                footer_message = "Configuration reset to original".to_string();
                            }
                            "Quit" => {
                                return Ok(None);
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Esc => return Ok(None),
                KeyCode::Char(c) => {
                    if c.to_string().eq_ignore_ascii_case("q") {
                        return Ok(None);
                    }
                    if c.to_string().eq_ignore_ascii_case("s") {
                        let merged = themes::apply_theme(
                            &base_config,
                            &theme_names[theme_index],
                            &themes_dir,
                        )?;
                        config::save(&merged)?;
                        return Ok(Some(merged));
                    }
                    if c.to_string().eq_ignore_ascii_case("r") {
                        base_config = base.clone();
                        theme_index = theme_names
                            .iter()
                            .position(|name| name == &base.theme)
                            .unwrap_or(0);
                        selected_segment = 0;
                        footer_message = "Configuration reset to original".to_string();
                    }
                }
                _ => {}
            }
        }
    }
}

fn handle_global_key(key: &KeyEvent, focus: &mut Focus) -> bool {
    match key.code {
        KeyCode::Tab => {
            *focus = match focus {
                Focus::Themes => Focus::Segments,
                Focus::Segments => Focus::Actions,
                Focus::Actions => Focus::Themes,
            };
            true
        }
        _ => false,
    }
}

fn handle_theme_keys(key: &KeyEvent, selected: &mut usize, total: usize) -> bool {
    if total == 0 {
        return false;
    }

    match key.code {
        KeyCode::Up => {
            if *selected == 0 {
                *selected = total - 1;
            } else {
                *selected = selected.saturating_sub(1);
            }
            true
        }
        KeyCode::Down => {
            *selected = (*selected + 1) % total;
            true
        }
        _ => false,
    }
}

fn handle_segment_keys(key: &KeyEvent, cfg: &mut Config, selected: &mut usize) -> bool {
    if cfg.segments.is_empty() {
        return false;
    }

    match key.code {
        KeyCode::Up => {
            if *selected == 0 {
                *selected = cfg.segments.len() - 1;
            } else {
                *selected = selected.saturating_sub(1);
            }
            true
        }
        KeyCode::Down => {
            *selected = (*selected + 1) % cfg.segments.len();
            true
        }
        KeyCode::Char(c) => {
            let text = c.to_string();
            if text == " " {
                let idx = (*selected).min(cfg.segments.len() - 1);
                cfg.segments[idx].enabled = !cfg.segments[idx].enabled;
                return true;
            }
            if text.eq_ignore_ascii_case("j") {
                let idx = (*selected).min(cfg.segments.len() - 1);
                if idx + 1 < cfg.segments.len() {
                    cfg.segments.swap(idx, idx + 1);
                    *selected = idx + 1;
                }
                return true;
            }
            if text.eq_ignore_ascii_case("k") {
                let idx = (*selected).min(cfg.segments.len() - 1);
                if idx > 0 {
                    cfg.segments.swap(idx, idx - 1);
                    *selected = idx - 1;
                }
                return true;
            }
            false
        }
        _ => false,
    }
}

fn handle_action_nav(key: &KeyEvent, selected: &mut usize, total: usize) -> bool {
    if total == 0 {
        return false;
    }

    match key.code {
        KeyCode::Up => {
            if *selected == 0 {
                *selected = total - 1;
            } else {
                *selected = selected.saturating_sub(1);
            }
            true
        }
        KeyCode::Down => {
            *selected = (*selected + 1) % total;
            true
        }
        _ => false,
    }
}

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode().context("failed to enable raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).context("failed to create terminal")?;
        terminal.clear().context("failed to clear terminal")?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
