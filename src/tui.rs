// ChromaAI Dev - ChromaTUI application. Without ChromaTUI there is no application.
// Copyright (c) 2026 ChromaAI Dev Team

use chromatui::{DeterministicRuntime, Event, Key, OutputMode, TerminalWriter};
use chromatui_core::{Cell, Color, Cmd, Effect, Frame, Model, TerminalSession};
use std::time::Duration;

/// Application model for the ChromaAI Dev TUI.
#[derive(Debug, Default)]
pub struct ChromaAppModel {
    pub ready_count: u32,
    pub in_progress_count: u32,
    pub open_count: u32,
    pub next_ticket: Option<String>,
}

impl Model for ChromaAppModel {
    fn update(&mut self, event: Event) -> Cmd<Self> {
        match event {
            Event::Key(Key::Char('q')) | Event::Key(Key::Escape) => Some(Effect::Quit),
            _ => None,
        }
    }
}

/// Renders the ChromaAI Dev home screen.
pub fn chroma_view(model: &ChromaAppModel) -> Frame {
    let width = 80;
    let height = 24;
    let mut frame = Frame::new(width, height);

    let title = format!("ChromaAI Dev v{}", env!("CARGO_PKG_VERSION"));
    let subtitle = "ChromaTUI-based AI development, evaluation, and release tool.";
    let hint = "Press q or Esc to quit";

    write_line(&mut frame, &title, 0, 0, Color::cyan(), width);
    write_line(&mut frame, subtitle, 0, 1, Color::white(), width);
    write_line(&mut frame, "", 0, 2, Color::white(), width);
    write_line(
        &mut frame,
        "Issues",
        0,
        3,
        Color::yellow(),
        width,
    );
    let stats = format!(
        "  Ready: {}  |  In progress: {}  |  Open: {}",
        model.ready_count, model.in_progress_count, model.open_count
    );
    write_line(&mut frame, &stats, 0, 4, Color::white(), width);
    if let Some(ref next) = model.next_ticket {
        write_line(&mut frame, &format!("  Next: {}", next), 0, 5, Color::green(), width);
    }
    write_line(&mut frame, "", 0, 6, Color::white(), width);
    write_line(&mut frame, hint, 0, height.saturating_sub(1), Color::white(), width);

    frame
}

fn write_line(
    frame: &mut Frame,
    s: &str,
    col_start: u16,
    row: u16,
    fg: Color,
    width: u16,
) {
    let chars: Vec<char> = s.chars().collect();
    for (i, &ch) in chars.iter().take(width.saturating_sub(col_start) as usize).enumerate() {
        if let Some(cell) = frame.cell(col_start + i as u16, row) {
            *cell = Cell::default().with_char(ch).with_fg(fg);
        }
    }
}

/// Runs the ChromaTUI application (blocking). Starts terminal session, runs the deterministic
/// runtime until quit, then restores the terminal.
pub fn run_tui() -> std::io::Result<()> {
    let mut session = TerminalSession::new();
    session.start()?;

    let (width, height) = session.get_size();
    let writer = TerminalWriter::try_stdout(OutputMode::AltScreen)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let model = load_issue_status_into_model();
    let mut runtime = DeterministicRuntime::new(model, writer, width, height);

    let _steps = runtime.run_with_source(
        &mut session,
        Duration::from_millis(16),
        10_000_000,
        chroma_view,
    )?;

    drop(runtime);
    session.stop()?;
    Ok(())
}

fn load_issue_status_into_model() -> ChromaAppModel {
    let cwd = match std::env::current_dir() {
        Ok(p) => p,
        Err(_) => return ChromaAppModel::default(),
    };
    let store = match crate::tickets::TicketStore::find(&cwd) {
        Ok(s) => s,
        Err(_) => return ChromaAppModel::default(),
    };
    let ready = store.ready().unwrap_or_default();
    let in_progress = store
        .list(
            Some(crate::tickets::TicketStatus::InProgress),
            None,
            None,
            None,
        )
        .unwrap_or_default();
    let open = store
        .list(
            Some(crate::tickets::TicketStatus::Open),
            None,
            None,
            None,
        )
        .unwrap_or_default();

    let next_ticket = ready
        .first()
        .map(|t| format!("{}  {}", t.id, t.title));

    ChromaAppModel {
        ready_count: ready.len() as u32,
        in_progress_count: in_progress.len() as u32,
        open_count: open.len() as u32,
        next_ticket,
    }
}
