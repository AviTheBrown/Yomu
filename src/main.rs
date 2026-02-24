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
    terminal.clear()?;
    
    // Initialize picker after terminal is in raw mode and alternate screen
    app.picker = ratatui_image::picker::Picker::from_query_stdio().ok();

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
    let [header, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(frame.area());

    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(body);

    let page_info = if let Some(img_data) = &app.image_data {
        format!("Pages {} & {} / {} - Press 'b' to go back", app.current_page + 1, app.current_page + 2, img_data.chapter.data.len())
    } else {
        "Reading Mode - Press 'b' to go back".to_string()
    };

    Paragraph::new(page_info)
        .centered()
        .render(header, frame.buffer_mut());

    let mut fallback_picker;
    let picker = if let Some(p) = app.picker.as_ref() {
        fallback_picker = p.clone();
        &mut fallback_picker
    } else {
        fallback_picker = ratatui_image::picker::Picker::halfblocks();
        &mut fallback_picker
    };

    // Right Panel (Page N) - Aligned to the LEFT of this panel (the spine)
    frame.render_widget(Block::default().bg(Color::Reset), right);
    if let Some(img) = &app.page_right {
        if let Ok(p) = picker.new_protocol(img.clone(), right, ratatui_image::Resize::Fit(Some(ratatui_image::FilterType::Lanczos3))) {
            let image_widget = ratatui_image::Image::new(&p);
            let actual_area = p.area();
            // Align to left of panel (spine)
            let x_offset = 0; 
            let y_offset = (right.height.saturating_sub(actual_area.height)) / 2;
            let centered_area = Rect::new(right.x + x_offset, right.y + y_offset, actual_area.width, actual_area.height);
            frame.render_widget(image_widget, centered_area);
        }
    } else {
        frame.render_widget(Paragraph::new("Loading...").centered(), right);
    }

    // Left Panel (Page N+1) - Aligned to the RIGHT of this panel (the spine)
    frame.render_widget(Block::default().bg(Color::Reset), left);
    if let Some(img) = &app.page_left {
        if let Ok(p) = picker.new_protocol(img.clone(), left, ratatui_image::Resize::Fit(Some(ratatui_image::FilterType::Lanczos3))) {
            let image_widget = ratatui_image::Image::new(&p);
            let actual_area = p.area();
            // Align to right of panel (spine)
            let x_offset = left.width.saturating_sub(actual_area.width);
            let y_offset = (left.height.saturating_sub(actual_area.height)) / 2;
            let centered_area = Rect::new(left.x + x_offset, left.y + y_offset, actual_area.width, actual_area.height);
            frame.render_widget(image_widget, centered_area);
        }
    } else {
        frame.render_widget(Paragraph::new("Loading...").centered(), left);
    }
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
                app.current_page = 0;
                app.page_cache.clear();
                app.page_left = None;
                app.page_right = None;
                
                // Optimized spread loading (Concurrent + Cache + Prefetch)
                if let Some(img_data) = app.image_data.clone() {
                    runtime.block_on(load_spread(app, &image_client, &img_data));
                }

                app.screen = AppScreen::Reading;
            }
            _ => {}
        },
        AppScreen::Reading => match key.code {
            KeyCode::Char('b') => {
                app.screen = AppScreen::ChapterList;
            }
            KeyCode::Char('l') | KeyCode::Right => {
                // Next spread
                if let Some(img_data) = app.image_data.clone() {
                    if app.current_page + 2 < img_data.chapter.data.len() {
                        app.current_page += 2;
                        
                        let image_client = client.image_client();
                        runtime.block_on(load_spread(app, &image_client, &img_data));
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                // Previous spread
                if let Some(img_data) = app.image_data.clone() {
                    if app.current_page >= 2 {
                        app.current_page -= 2;

                        let image_client = client.image_client();
                        runtime.block_on(load_spread(app, &image_client, &img_data));
                    }
                }
            }
            _ => {}
        },
    }
}

async fn load_spread(
    app: &mut App,
    image_client: &yomu::image::ImageClient<'_>,
    img_data: &yomu::image::ImageDataResponse,
) {
    let current = app.current_page;
    let next = current + 1;

    // 1. Check Cache
    let has_right = app.page_cache.contains_key(&current);
    let has_left = img_data.chapter.data.len() > next && app.page_cache.contains_key(&next);

    if has_right {
        app.page_right = app.page_cache.get(&current).cloned();
    }
    if has_left {
        app.page_left = app.page_cache.get(&next).cloned();
    }

    // 2. Fetch missing pages concurrently
    if !has_right || (!has_left && img_data.chapter.data.len() > next) {
        let fut_right = async {
            if !has_right {
                let url = format!("{}/data/{}/{}", img_data.base_url, img_data.chapter.hash, img_data.chapter.data[current]);
                if let Ok(bytes) = image_client.download_image_bytes(&url).await {
                    return image::load_from_memory(&bytes).ok();
                }
            }
            None
        };

        let fut_left = async {
            if !has_left && img_data.chapter.data.len() > next {
                let url = format!("{}/data/{}/{}", img_data.base_url, img_data.chapter.hash, img_data.chapter.data[next]);
                if let Ok(bytes) = image_client.download_image_bytes(&url).await {
                    return image::load_from_memory(&bytes).ok();
                }
            }
            None
        };

        let (res_right, res_left) = tokio::join!(fut_right, fut_left);

        if let Some(img) = res_right {
            app.page_cache.insert(current, img.clone());
            app.page_right = Some(img);
        }
        if let Some(img) = res_left {
            app.page_cache.insert(next, img.clone());
            app.page_left = Some(img);
        } else if img_data.chapter.data.len() <= next {
            app.page_left = None;
        }
    }

    // 3. Prefetch next spread
    prefetch_spread(app, image_client, img_data).await;
}

async fn prefetch_spread(
    app: &mut App,
    image_client: &yomu::image::ImageClient<'_>,
    img_data: &yomu::image::ImageDataResponse,
) {
    let prefetch_idx = app.current_page + 2;
    let prefetch_next_idx = prefetch_idx + 1;

    if prefetch_idx < img_data.chapter.data.len() && !app.page_cache.contains_key(&prefetch_idx) {
        let fut_p1 = async {
            let url = format!("{}/data/{}/{}", img_data.base_url, img_data.chapter.hash, img_data.chapter.data[prefetch_idx]);
            if let Ok(bytes) = image_client.download_image_bytes(&url).await {
                return image::load_from_memory(&bytes).ok();
            }
            None
        };

        let fut_p2 = async {
            if prefetch_next_idx < img_data.chapter.data.len() && !app.page_cache.contains_key(&prefetch_next_idx) {
                let url = format!("{}/data/{}/{}", img_data.base_url, img_data.chapter.hash, img_data.chapter.data[prefetch_next_idx]);
                if let Ok(bytes) = image_client.download_image_bytes(&url).await {
                    return image::load_from_memory(&bytes).ok();
                }
            }
            None
        };

        let (p1, p2) = tokio::join!(fut_p1, fut_p2);
        if let Some(img) = p1 {
            app.page_cache.insert(prefetch_idx, img);
        }
        if let Some(img) = p2 {
            app.page_cache.insert(prefetch_next_idx, img);
        }
    }
}
