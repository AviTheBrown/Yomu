mod app;
use app::App;
use app::AppScreen;
use crossterm::event::KeyEvent;
use crossterm::{
    ExecutableCommand,
    event::KeyCode,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::widgets::ListState;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};
use std::io::stdout;
use tokio::runtime::Runtime;
use yomu::MangaDexClient;
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let runtime = Runtime::new()?;
    let mut app = app::App::new();
    let client = MangaDexClient::new()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    loop {
        terminal.draw(|frame| {
            render(&app, frame);
        })?;
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.code == crossterm::event::KeyCode::Esc {
                    break;
                } else {
                    handle_event(&client, &mut app, &key, &runtime);
                }
            }
        }
    }
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

// will draw the UI based on AppState
fn render(app: &App, frame: &mut Frame<'_>) {
    match app.screen {
        AppScreen::Search => draw_search(app, frame),
        AppScreen::ChapterList => todo!(),
        AppScreen::Reading => todo!(),
    }
}

// reads keyboard input and updates appstate accordingly
fn draw_search(app: &App, frame: &mut Frame<'_>) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());
    let serarch_area = layout[0];
    let result_area = layout[1];
    let mut list_state = ListState::default();
    frame.render_widget(
        ratatui::widgets::Paragraph::new(app.search_input.as_str())
            .block(Block::default().borders(Borders::ALL).title_top("Search")),
        serarch_area,
    );
    let items: Vec<ratatui::widgets::ListItem> = app
        .search_result
        .iter()
        .map(|manga| {
            let title = manga
                .attributes
                .title
                .as_ref()
                // TODO USE app.preferred_lang
                .and_then(|t| t.get("en"))
                .map(|t| t.as_str())
                .unwrap_or("Unknown Title");
            ratatui::widgets::ListItem::new(title)
        })
        .collect();
    list_state.select(Some(app.selected_index));
    frame.render_stateful_widget(
        ratatui::widgets::List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title_top("Result(s)"),
            )
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black)),
        result_area,
        &mut list_state,
    );
}
fn handle_event(client: &MangaDexClient, app: &mut App, key: &KeyEvent, rt: &Runtime) {
    match app.screen {
        AppScreen::Search => match key.code {
            KeyCode::Char(c) => {
                app.search_input.push(c);
            }
            KeyCode::Backspace => {
                app.search_input.pop();
            }
            KeyCode::Enter => {
                if app.search_result.is_empty() {
                    let search_client = client.search_client();
                    let result =
                        rt.block_on(async { search_client.search(app.search_input.clone()).await });
                    if let Ok(result) = result {
                        app.search_result = result;
                    } else if let Err(e) = result {
                        eprintln!("Error: {}", e);
                    }
                } else {
                    app.selected_manga = Some(app.search_result[app.selected_index].clone());
                    app.screen = AppScreen::ChapterList;
                }
            }
            KeyCode::Up => app.selected_index = app.selected_index.saturating_sub(1),
            KeyCode::Down => {
                if app.selected_index + 1 < app.search_result.len() {
                    app.selected_index = app.selected_index + 1;
                }
            }
            _ => {}
        },
        _ => {} // AppScreen::ChapterList => match key.code todo!()
                // AppScreen::Reading => match key.code todo!()
    }
}
