use crate::error::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use similar::{ChangeTag, TextDiff};
use std::io;

pub fn show_diff_tui(title: &str, remote: &str, local: &str) -> Result<bool> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, title, remote, local);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    title: &str,
    remote: &str,
    local: &str,
) -> Result<bool> {
    let mut scroll: u16 = 0;
    let diff = TextDiff::from_lines(remote, local);

    loop {
        terminal.draw(|f| {
            ui(f, title, &diff, scroll);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
                KeyCode::Enter => return Ok(true),
                KeyCode::Down | KeyCode::Char('j') => {
                    scroll = scroll.saturating_add(1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    scroll = scroll.saturating_sub(1);
                }
                KeyCode::PageDown => {
                    scroll = scroll.saturating_add(10);
                }
                KeyCode::PageUp => {
                    scroll = scroll.saturating_sub(10);
                }
                _ => {}
            }
        }
    }
}

fn ui<'a>(f: &mut Frame, title: &str, diff: &TextDiff<'a, 'a, 'a, str>, scroll: u16) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Title
    let title_block = Block::default().borders(Borders::ALL);
    let title_text = Paragraph::new(title).block(title_block);
    f.render_widget(title_text, chunks[0]);

    // Diff content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    render_diff_panel(f, content_chunks[0], diff, scroll, true);
    render_diff_panel(f, content_chunks[1], diff, scroll, false);

    // Help text
    let help = Paragraph::new("[↑/↓] Scroll  [Enter] Publish  [q] Cancel")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}

fn render_diff_panel<'a>(
    f: &mut Frame,
    area: Rect,
    diff: &TextDiff<'a, 'a, 'a, str>,
    scroll: u16,
    is_remote: bool,
) {
    let title = if is_remote {
        "Remote (Note.com)"
    } else {
        "Local File"
    };

    let mut lines: Vec<Line> = Vec::new();

    for change in diff.iter_all_changes() {
        let (should_show, style) = match (change.tag(), is_remote) {
            (ChangeTag::Delete, true) => (
                true,
                Style::default()
                    .fg(Color::Red)
                    .bg(Color::Rgb(60, 0, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            (ChangeTag::Insert, false) => (
                true,
                Style::default()
                    .fg(Color::Green)
                    .bg(Color::Rgb(0, 60, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            (ChangeTag::Delete, false) => (false, Style::default()),
            (ChangeTag::Insert, true) => (false, Style::default()),
            (ChangeTag::Equal, _) => (true, Style::default()),
        };

        if should_show {
            let line_text = change.as_str().unwrap_or("").trim_end();
            lines.push(Line::from(Span::styled(line_text, style)));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_diff() {
        let old = "line 1\nline 2\nline 3";
        let new = "line 1\nmodified line 2\nline 3";

        let diff = TextDiff::from_lines(old, new);
        let changes: Vec<_> = diff.iter_all_changes().collect();

        assert!(!changes.is_empty());
    }
}
