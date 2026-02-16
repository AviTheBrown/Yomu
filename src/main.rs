use crossterm::{
    ExecutableCommand,
    event::KeyCode,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, RatatuiLogo},
};
use std::io::stdout;
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    loop {
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(frame.area());
            let search_area = layout[0];
            let result_area = layout[1];
            frame.render_widget(
                ratatui::widgets::Paragraph::new("Hello to YOMU")
                    .block(Block::default().borders(Borders::ALL).title("Search")),
                search_area,
            );
            frame.render_widget(
                ratatui::widgets::Paragraph::new("Hello to YOMU")
                    .block(Block::default().borders(Borders::ALL).title("Result")),
                result_area,
            );
            // TODO ui
        })?;
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.code == crossterm::event::KeyCode::Char('q') {
                    break;
                }
            }
        }
    }
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
