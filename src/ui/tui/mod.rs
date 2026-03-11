//! Terminal UI module
//!
//! Ratatui-based terminal interface.

mod views;
mod widgets;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;

use crate::aria2::Aria2;
use crate::config::Config;
use crate::queue::JobQueue;
use crate::scheduler::Scheduler;

/// Application state.
pub struct App {
    config: Config,
    queue: JobQueue,
    scheduler: Scheduler,
    aria2: Aria2,
    selected_tab: usize,
    selected_job: usize,
    should_quit: bool,
}

impl App {
    pub fn new(config: Config, queue: JobQueue, scheduler: Scheduler, aria2: Aria2) -> Self {
        Self {
            config,
            queue,
            scheduler,
            aria2,
            selected_tab: 0,
            selected_job: 0,
            should_quit: false,
        }
    }
}

/// Run the TUI.
pub async fn run(
    config: Config,
    queue: JobQueue,
    scheduler: Scheduler,
    aria2: Aria2,
) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(config, queue, scheduler, aria2);

    // Main loop
    let tick_rate = Duration::from_millis(250);

    loop {
        // Draw UI
        terminal.draw(|f| draw_ui(f, &app))?;

        // Handle input
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                match (key.modifiers, key.code) {
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                        app.should_quit = true;
                    }
                    (_, KeyCode::Char('q')) => {
                        app.should_quit = true;
                    }
                    (_, KeyCode::Tab) => {
                        app.selected_tab = (app.selected_tab + 1) % 4;
                    }
                    (_, KeyCode::BackTab) => {
                        app.selected_tab = app.selected_tab.saturating_sub(1);
                    }
                    (_, KeyCode::Up) | (_, KeyCode::Char('k')) => {
                        app.selected_job = app.selected_job.saturating_sub(1);
                    }
                    (_, KeyCode::Down) | (_, KeyCode::Char('j')) => {
                        app.selected_job = app.selected_job.saturating_add(1);
                    }
                    (_, KeyCode::Char('p')) => {
                        // Pause/resume selected
                    }
                    (_, KeyCode::Char('d')) => {
                        // Delete selected
                    }
                    (_, KeyCode::Char('a')) => {
                        // Add new download
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Draw the UI.
fn draw_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Status bar
        ])
        .split(f.area());

    // Header
    draw_header(f, chunks[0]);

    // Tabs
    draw_tabs(f, chunks[1], app.selected_tab);

    // Content
    match app.selected_tab {
        0 => draw_downloads(f, chunks[2], app),
        1 => draw_completed(f, chunks[2], app),
        2 => draw_settings(f, chunks[2], app),
        3 => draw_help(f, chunks[2]),
        _ => {}
    }

    // Status bar
    draw_status_bar(f, chunks[3], app);
}

fn draw_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            " ferrum",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("-dl", Style::default().fg(Color::Green)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::raw("Rust download manager"),
    ])])
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(header, area);
}

fn draw_tabs(f: &mut Frame, area: Rect, selected: usize) {
    let titles = vec!["Downloads", "Completed", "Settings", "Help"];
    let tabs = Tabs::new(titles)
        .select(selected)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" │ ");

    f.render_widget(tabs, area);
}

fn draw_downloads(f: &mut Frame, area: Rect, _app: &App) {
    let items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::styled("▶ ", Style::default().fg(Color::Green)),
            Span::raw("example-file.zip"),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled("45%", Style::default().fg(Color::Yellow)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::raw("2.3 MB/s"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("⏸ ", Style::default().fg(Color::Yellow)),
            Span::raw("another-download.tar.gz"),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled("12%", Style::default().fg(Color::Yellow)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::raw("Paused"),
        ])),
    ];

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Active Downloads "),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_widget(list, area);
}

fn draw_completed(f: &mut Frame, area: Rect, _app: &App) {
    let items: Vec<ListItem> = vec![ListItem::new(Line::from(vec![
        Span::styled("✓ ", Style::default().fg(Color::Green)),
        Span::raw("completed-file.mp4"),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::raw("1.2 GB"),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::raw("2 hours ago"),
    ]))];

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Completed "),
    );

    f.render_widget(list, area);
}

fn draw_settings(f: &mut Frame, area: Rect, app: &App) {
    let text = vec![
        Line::from(vec![
            Span::styled("Download Directory: ", Style::default().fg(Color::Gray)),
            Span::raw(&app.config.general.download_dir),
        ]),
        Line::from(vec![
            Span::styled("Max Concurrent: ", Style::default().fg(Color::Gray)),
            Span::raw(app.config.general.max_concurrent.to_string()),
        ]),
        Line::from(vec![
            Span::styled("aria2 RPC URL: ", Style::default().fg(Color::Gray)),
            Span::raw(&app.config.aria2.rpc_url),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Settings "),
    );

    f.render_widget(paragraph, area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  j/↓  ", Style::default().fg(Color::Green)),
            Span::raw("Move down"),
        ]),
        Line::from(vec![
            Span::styled("  k/↑  ", Style::default().fg(Color::Green)),
            Span::raw("Move up"),
        ]),
        Line::from(vec![
            Span::styled("  Tab  ", Style::default().fg(Color::Green)),
            Span::raw("Next tab"),
        ]),
        Line::from(vec![
            Span::styled("  p    ", Style::default().fg(Color::Green)),
            Span::raw("Pause/Resume"),
        ]),
        Line::from(vec![
            Span::styled("  d    ", Style::default().fg(Color::Green)),
            Span::raw("Delete"),
        ]),
        Line::from(vec![
            Span::styled("  a    ", Style::default().fg(Color::Green)),
            Span::raw("Add download"),
        ]),
        Line::from(vec![
            Span::styled("  q    ", Style::default().fg(Color::Green)),
            Span::raw("Quit"),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Keybindings "),
    );

    f.render_widget(paragraph, area);
}

fn draw_status_bar(f: &mut Frame, area: Rect, _app: &App) {
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" ● ", Style::default().fg(Color::Green)),
        Span::raw("aria2 connected"),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::raw("2 active"),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::raw("5 queued"),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::raw("↓ 4.5 MB/s"),
    ]))
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(status, area);
}
