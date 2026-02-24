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
use ratatui::widgets::Paragraph;
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
        AppScreen::ChapterList => draw_chapter_list(app, frame),
        AppScreen::Reading => draw_reading_page(app, frame),
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
fn draw_chapter_list(app: &App, frame: &mut Frame<'_>) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    let manga_area = layout[0];
    let chapter_list_area = layout[1];

    let mut list_state = ListState::default();
    let manga_name = app
        .selected_manga
        .as_ref()
        .and_then(|m| m.attributes.title.as_ref())
        .and_then(|t| t.get("en"))
        .map(|s| s.as_str())
        .unwrap_or("Unknown Manga");
    frame.render_widget(
        ratatui::widgets::Paragraph::new(manga_name)
            .block(Block::default().borders(Borders::ALL).title_top("Manga")),
        manga_area,
    );
    let items: Vec<ratatui::widgets::ListItem> = app
        .chapters
        .iter()
        .map(|chapter| {
            let chpt_name = chapter.attributes.title.as_deref().unwrap_or("No Title");
            let chpt = chapter
                .attributes
                .chapter
                .as_deref()
                .unwrap_or("No Chapter Info");
            let vol = chapter.attributes.volume.as_deref().unwrap_or("N/A");
            let pages = chapter
                .attributes
                .pages
                .map(|p| p.to_string())
                .unwrap_or_else(|| "0".to_string());

            ratatui::widgets::ListItem::new(format!(
                " Chapter {} {}, (Vol.{},{} pages)",
                chpt, chpt_name, vol, pages
            ))
        })
        .collect();

    list_state.select(Some(app.selected_index));
    frame.render_stateful_widget(
        ratatui::widgets::List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title_top("Chapters(s)"),
            )
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black)),
        chapter_list_area,
        &mut list_state,
    );
}
fn draw_reading_page(app: &App, frame: &mut Frame<'_>) {
    let Some(image_data)= app.image_data.as_ref() else {
        eprintln!("There was an error fetching the chapter pages");
        return;
    };
    let [header, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(frame.area());

    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(body);

    Paragraph::new("headers spread")
        .centered()
        .render(header, frame.buffer_mut());

    frame.render_widget(Block::bordered().title("page<l>"), left);
    frame.render_widget(Block::bordered().title("page<r>"), right);
}
fn handle_event(client: &MangaDexClient, app: &mut App, key: &KeyEvent, runtime: &Runtime) {
    match app.screen {
        AppScreen::Search => match key.code {
            KeyCode::Char(c) => {
                app.search_input.push(c);
            }
            KeyCode::Backspace => {
                app.search_input.pop();
            }
            KeyCode::Enter => {
                if !app.search_input.is_empty() && app.search_input != app.last_search_query {
                    let search_client = client.search_client();
                    let result = runtime
                        .block_on(async { search_client.search(app.search_input.clone()).await });
                    if let Ok(result) = result {
                        app.search_result = result;
                        app.last_search_query = app.search_input.clone();
                    } else if let Err(e) = result {
                        eprintln!("Error: {}", e);
                    } else if !app.search_result.is_empty()
                        && app.search_input == app.last_search_query
                    {
                        // TODO add better handling
                        eprintln!("Error")
                    }
                } else if !app.search_result.is_empty() {
                    app.selected_manga = Some(app.search_result[app.selected_index].clone());
                    let chapter_client = client.chapter_client();

                    let manga_id = app.selected_manga.as_ref().map(|manga| &manga.id);
                    let Some(manga_id_str) = manga_id else {
                        eprintln!("Value of id was none. exiting..");
                        return;
                    };
                    let chapter_result = runtime.block_on(async {
                        let chapter_data_result = chapter_client
                            .fetch_chapter(manga_id_str.as_str(), Some("en"))
                            .await;
                        chapter_data_result
                    });
                    let Ok(chapter_data) = chapter_result else {
                        eprint!("There was an error fetching the chapter data");
                        return;
                    };
                    app.chapters = chapter_data;
                    app.screen = AppScreen::ChapterList;
                    app.selected_index = 0;
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
        AppScreen::ChapterList => match key.code {
            KeyCode::Char('b') => {
                app.screen = AppScreen::Search;
            }
            KeyCode::Up => app.selected_index = app.selected_index.saturating_sub(1),
            KeyCode::Down => {
                if app.selected_index + 1 < app.chapters.len() {
                    app.selected_index = app.selected_index + 1;
                }
            }
            KeyCode::Enter => {
                // create client
                // get chapter id
                // get imageData
                let image_client = client.image_client();
                let chapter_id = &app.chapters[app.selected_index].id;
                let result =
                    runtime.block_on(async { image_client.fetch_image_data(chapter_id).await });
                let Ok(image_data) = result else {
                    eprintln!("There was an error getting the image data: {:?}", result);
                    return;
                };
                app.image_data = Some(image_data);
                let img_data = app
                    .image_data
                    .as_ref()
                    .expect("Image data should be present");
                if img_data.chapter.data.is_empty() {
                    // TODO add a display for users to kmow there are
                    // no pages available
                    eprint!("there are not pages to display");
                    return;
                }
                let full_url = format!(
                    "{}/data/{}/{}",
                    img_data.base_url, img_data.chapter.hash, img_data.chapter.data[0]
                );
                let _img_bytes = match runtime
                    .block_on(async { image_client.download_image_bytes(&full_url).await })
                {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        eprintln!("there was an error downloading the bytes from the images: {e}");
                        return;
                    }
                };
                app.screen = AppScreen::Reading;
            }
            _ => {}
        },
        AppScreen::Reading => match key.code {
            KeyCode::Char('b') => {
                app.screen = AppScreen::ChapterList;
            }
            _ => {}
        },
    }
}
