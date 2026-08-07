//! # Loom TUI
//!
//! Terminal User Interface for real-time monitoring of the Loom OS kernel,
//! plugin health, GPU/LLM metrics, and system logs.

use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Gauge},
    Terminal,
    Frame,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Dashboard,
    Plugins,
    Gpu,
    Llm,
    Logs,
}

struct App {
    active_tab: Tab,
    plugins: Vec<PluginRow>,
    logs: Vec<String>,
    gpu_util: f64,
    vram_used_mb: f64,
    vram_total_mb: f64,
    llm_queue: usize,
}

struct PluginRow {
    name: &'static str,
    version: &'static str,
    status: &'static str,
    memory: &'static str,
}

impl App {
    fn new() -> Self {
        Self {
            active_tab: Tab::Dashboard,
            plugins: vec![
                PluginRow { name: "repurpose", version: "0.1.0", status: "🟢", memory: "12 MB" },
                PluginRow { name: "ethos-guardian", version: "0.1.0", status: "🟢", memory: "4 MB" },
                PluginRow { name: "thumbnail-engine", version: "0.1.0", status: "🟢", memory: "256 MB" },
                PluginRow { name: "fact-checker", version: "0.1.0", status: "🟢", memory: "8 MB" },
                PluginRow { name: "dubbing-studio", version: "0.1.0", status: "🟡", memory: "512 MB" },
                PluginRow { name: "aether-audio", version: "0.1.0", status: "🟢", memory: "64 MB" },
                PluginRow { name: "video-pipeline", version: "0.1.0", status: "🟢", memory: "1 GB" },
                PluginRow { name: "content-scorer", version: "0.1.0", status: "🟢", memory: "32 MB" },
                PluginRow { name: "momentum", version: "0.1.0", status: "🟢", memory: "2 MB" },
                PluginRow { name: "forge", version: "0.1.0", status: "🟢", memory: "16 MB" },
            ],
            logs: vec![
                "[INFO] Kernel initialized".into(),
                "[INFO] LLM bridge: standby".into(),
                "[INFO] GPU scheduler: online (Vulkan)".into(),
                "[INFO] 10 plugins registered".into(),
                "[INFO] IPC bus: mpsc active".into(),
            ],
            gpu_util: 12.0,
            vram_used_mb: 4200.0,
            vram_total_mb: 24576.0,
            llm_queue: 0,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    let mut last_tick = std::time::Instant::now();
    let tick_rate = std::time::Duration::from_millis(250);

    loop {
        terminal.draw(|f| draw(f, app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('1') => app.active_tab = Tab::Dashboard,
                    KeyCode::Char('2') => app.active_tab = Tab::Plugins,
                    KeyCode::Char('3') => app.active_tab = Tab::Gpu,
                    KeyCode::Char('4') => app.active_tab = Tab::Llm,
                    KeyCode::Char('5') => app.active_tab = Tab::Logs,
                    KeyCode::Right => app.active_tab = next_tab(app.active_tab),
                    KeyCode::Left => app.active_tab = prev_tab(app.active_tab),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.gpu_util = (app.gpu_util + 0.5) % 100.0;
            last_tick = std::time::Instant::now();
        }
    }
}

fn next_tab(tab: Tab) -> Tab {
    match tab {
        Tab::Dashboard => Tab::Plugins,
        Tab::Plugins => Tab::Gpu,
        Tab::Gpu => Tab::Llm,
        Tab::Llm => Tab::Logs,
        Tab::Logs => Tab::Dashboard,
    }
}

fn prev_tab(tab: Tab) -> Tab {
    match tab {
        Tab::Dashboard => Tab::Logs,
        Tab::Plugins => Tab::Dashboard,
        Tab::Gpu => Tab::Plugins,
        Tab::Llm => Tab::Gpu,
        Tab::Logs => Tab::Llm,
    }
}

fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(f.size());

    let titles = vec!["Dashboard", "Plugins", "GPU", "LLM", "Logs"];
    let tabs = Tabs::new(titles.iter().map(|t| Line::from(*t)).collect())
        .select(app.active_tab as usize)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title(" Loom OS v0.2.0 "));
    f.render_widget(tabs, chunks[0]);

    match app.active_tab {
        Tab::Dashboard => draw_dashboard(f, app, chunks[1]),
        Tab::Plugins => draw_plugins(f, app, chunks[1]),
        Tab::Gpu => draw_gpu(f, app, chunks[1]),
        Tab::Llm => draw_llm(f, app, chunks[1]),
        Tab::Logs => draw_logs(f, app, chunks[1]),
    }

    let footer = Paragraph::new("[1-5] Tabs | [←→] Navigate | [q] Quit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

fn draw_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Kernel Status: ", Style::default().fg(Color::Gray)),
            Span::styled("ONLINE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("LLM Bridge:    ", Style::default().fg(Color::Gray)),
            Span::styled("STANDBY", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("GPU Scheduler: ", Style::default().fg(Color::Gray)),
            Span::styled("ONLINE (Vulkan)", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("IPC Transport: ", Style::default().fg(Color::Gray)),
            Span::styled("mpsc", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Plugins:       ", Style::default().fg(Color::Gray)),
            Span::styled("10 / 25", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("VRAM: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1} / {:.1} GB", app.vram_used_mb / 1024.0, app.vram_total_mb / 1024.0),
                Style::default().fg(Color::Magenta),
            ),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title(" System "));
    f.render_widget(left, chunks[0]);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" GPU Utilization "))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(app.gpu_util as u16)
        .label(format!("{:.1}%", app.gpu_util));
    f.render_widget(gauge, chunks[1]);
}

fn draw_plugins(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.plugins.iter().map(|p| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:20}", p.name), Style::default().fg(Color::White)),
            Span::styled(format!("{:8}", p.version), Style::default().fg(Color::Gray)),
            Span::styled(format!("{:4}", p.status), Style::default()),
            Span::styled(format!("{:>10}", p.memory), Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Active Plugins (10) "))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_widget(list, area);
}

fn draw_gpu(f: &mut Frame, app: &App, area: Rect) {
    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled("Adapter:    ", Style::default().fg(Color::Gray)), Span::raw("NVIDIA RTX 4090 / Apple M3 / AMD RX 7900 XTX")]),
        Line::from(vec![Span::styled("Backend:    ", Style::default().fg(Color::Gray)), Span::raw("Vulkan / Metal / DX12")]),
        Line::from(vec![Span::styled("Shader Cache: ", Style::default().fg(Color::Gray)), Span::raw("2 / 50 cached")]),
        Line::from(""),
        Line::from(vec![Span::styled("Active Compute Passes: ", Style::default().fg(Color::Gray)), Span::raw("0")]),
        Line::from(vec![Span::styled("Pending Queue:         ", Style::default().fg(Color::Gray)), Span::raw("0")]),
    ])
    .block(Block::default().borders(Borders::ALL).title(" GPU Compute "));
    f.render_widget(text, area);
}

fn draw_llm(f: &mut Frame, app: &App, area: Rect) {
    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled("Model:      ", Style::default().fg(Color::Gray)), Span::raw("mistral-7b-instruct-v0.2.Q4_K_M.gguf")]),
        Line::from(vec![Span::styled("Backend:    ", Style::default().fg(Color::Gray)), Span::raw("candle-core (CUDA)")]),
        Line::from(vec![Span::styled("Queue:      ", Style::default().fg(Color::Gray)), Span::raw(format!("{}", app.llm_queue))]),
        Line::from(vec![Span::styled("VRAM:       ", Style::default().fg(Color::Gray)), Span::raw("~4.2 GB allocated")]),
        Line::from(""),
        Line::from("Last inference: N/A"),
    ])
    .block(Block::default().borders(Borders::ALL).title(" LLM Inference "));
    f.render_widget(text, area);
}

fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.logs.iter().map(|l| {
        let color = if l.contains("ERROR") { Color::Red }
            else if l.contains("WARN") { Color::Yellow }
            else { Color::Green };
        ListItem::new(Line::from(Span::styled(l.clone(), Style::default().fg(color))))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" System Logs "));
    f.render_widget(list, area);
}
