mod app;
use app::App;
use app::AppScreen;
use crossterm::event::KeyEvent;
use crossterm::{
    ExecutableCommand,
    event::KeyCode,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};
use ratatui_image::protocol::Protocol;
use std::io::stdout;
use tokio::sync::mpsc;
use yomu::MangaDexClient;
use yomu::image::ImageDataResponse;

/// Message from a background image-download task.
type PageMsg = (usize, image::DynamicImage);
/// Message from a background protocol-build task: (page_idx, is_left_panel, protocol).
type ProtoMsg = (usize, bool, Protocol);

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let mut app = app::App::new();
    let client = MangaDexClient::new()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    app.picker = ratatui_image::picker::Picker::from_query_stdio().ok();

    let (page_tx, mut page_rx) = mpsc::channel::<PageMsg>(64);
    let (proto_tx, mut proto_rx) = mpsc::channel::<ProtoMsg>(32);

    loop {
        // 1. Drain newly downloaded images
        while let Ok((idx, img)) = page_rx.try_recv() {
            if idx == app.current_page {
                app.page_right = Some(img.clone());
            } else if idx == app.current_page + 1 {
                app.page_left = Some(img.clone());
            }
            // Pre-build the protocol for this page in the background now that the image is ready.
            // Determines which panel(s) this page will appear on.
            if idx == app.current_page && app.last_right_area != Rect::default() {
                spawn_proto(
                    app.picker.clone(),
                    img.clone(),
                    app.last_right_area,
                    idx,
                    false,
                    proto_tx.clone(),
                );
            }
            if idx == app.current_page + 1 && app.last_left_area != Rect::default() {
                spawn_proto(
                    app.picker.clone(),
                    img.clone(),
                    app.last_left_area,
                    idx,
                    true,
                    proto_tx.clone(),
                );
            }
            // Also pre-build for when this prefetched page becomes the current spread
            // (current+2 will be right, current+3 will be left on next navigation)
            if idx == app.current_page + 2 && app.last_right_area != Rect::default() {
                spawn_proto(
                    app.picker.clone(),
                    img.clone(),
                    app.last_right_area,
                    idx,
                    false,
                    proto_tx.clone(),
                );
            }
            if idx == app.current_page + 3 && app.last_left_area != Rect::default() {
                spawn_proto(
                    app.picker.clone(),
                    img.clone(),
                    app.last_left_area,
                    idx,
                    true,
                    proto_tx.clone(),
                );
            }
            app.page_cache.insert(idx, img);
            app.fetched.insert(idx); // mark complete so re-entering load_spread never re-fetches
        }

        // 2. Drain pre-built protocols into the cache
        while let Ok((idx, is_left, proto)) = proto_rx.try_recv() {
            let area = if is_left {
                app.last_left_area
            } else {
                app.last_right_area
            };
            app.proto_cache.insert((idx, is_left), (area, proto));
        }

        terminal.draw(|frame| {
            render(&mut app, frame);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(16))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.code == crossterm::event::KeyCode::Esc {
                    break;
                } else {
                    handle_event(&client, &mut app, &key, &page_tx, &proto_tx).await;
                }
            }
        }
    }
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

/// Dispatches the render call to the appropriate screen drawing function.
fn render(app: &mut App, frame: &mut Frame<'_>) {
    match app.screen {
        AppScreen::Splash => draw_splash(app, frame),
        AppScreen::Search => draw_search(app, frame),
        AppScreen::ChapterList => draw_chapter_list(app, frame),
        AppScreen::Reading => draw_reading_page(app, frame),
    }
}

/// Renders the anime-themed splash screen.
fn draw_splash(_app: &App, frame: &mut Frame<'_>) {
    let area = frame.area();
    
    // Background color
    frame.render_widget(Block::default().bg(Color::Rgb(10, 10, 20)), area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10), // Spacer
            Constraint::Length(7),      // Title/ASCII
            Constraint::Length(3),      // Subtitle
            Constraint::Min(10),        // Info Boxes
            Constraint::Length(3),      // Footer
        ])
        .split(area);

    // ASCII Art Title (Colorful Pink/Magenta)
    let ascii_art = r#"
  __     ______  __  __ _    _ 
  \ \   / / __ \|  \/  | |  | |
   \ \_/ / |  | | \  / | |  | |
    \   /| |  | | |\/| | |  | |
     | | | |__| | |  | | |__| |
     |_|  \____/|_|  |_|\____/ 
    "#;
    Paragraph::new(ascii_art)
        .style(Style::default().fg(Color::Rgb(255, 105, 180)).add_modifier(Modifier::BOLD))
        .centered()
        .render(layout[1], frame.buffer_mut());

    Paragraph::new("The Ultimate Manga Reader for your Terminal ✨")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC))
        .centered()
        .render(layout[2], frame.buffer_mut());

    // Three side-by-side boxes
    let info_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .margin(2)
        .split(layout[3]);

    // Directions Box (Blue)
    let directions = r#"
• [Enter] Select / Read
• [Esc]   Back / Quit
• [Arrows] Navigate
• [l / r]  Next / Prev Spread
• [b]      To Chapter List
    "#;
    frame.render_widget(
        Paragraph::new(directions)
            .block(Block::default().borders(Borders::ALL).title(" 📖 Directions ").border_style(Style::default().fg(Color::Blue)))
            .style(Style::default().fg(Color::White)),
        info_layout[0],
    );

    // Use Cases Box (Green)
    let use_cases = r#"
• High-quality MangaDex Reading
• Instant Pre-built Protocols
• Blazing-fast Async Engine
• Smart Image Caching
• Native Sixel & Kitty Support
    "#;
    frame.render_widget(
        Paragraph::new(use_cases)
            .block(Block::default().borders(Borders::ALL).title(" 🚀 Use Cases ").border_style(Style::default().fg(Color::Green)))
            .style(Style::default().fg(Color::White)),
        info_layout[1],
    );

    // Limitations Box (Yellow/Orange)
    let limitations = r#"
• API Rate Limits apply
• Requires Network access
• Terminal must support gfx
• Encoding is CPU-bound
• Some protocols are experimental
    "#;
    frame.render_widget(
        Paragraph::new(limitations)
            .block(Block::default().borders(Borders::ALL).title(" ⚠️ Limitations ").border_style(Style::default().fg(Color::Yellow)))
            .style(Style::default().fg(Color::White)),
        info_layout[2],
    );

    // Footer
    Paragraph::new("Press [Any Key] to start searching!")
        .style(Style::default().fg(Color::Rgb(200, 200, 200)).add_modifier(Modifier::SLOW_BLINK))
        .centered()
        .render(layout[4], frame.buffer_mut());
}

/// Renders the manga search and results screen.
fn draw_search(app: &App, frame: &mut Frame<'_>) {
    let area = frame.area();
    // Background
    frame.render_widget(Block::default().bg(Color::Rgb(10, 10, 20)), area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);
    let search_area = layout[0];
    let result_area = layout[1];

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    // Search Box
    frame.render_widget(
        Paragraph::new(app.search_input.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 🔍 Search ")
                .border_style(Style::default().fg(Color::Cyan)),
        ),
        search_area,
    );

    // Results List
    let items: Vec<ratatui::widgets::ListItem> = app
        .search_result
        .iter()
        .map(|m| {
            let title = m
                .attributes
                .title
                .as_ref()
                .and_then(|t| t.get("en"))
                .map(|t| t.as_str())
                .unwrap_or("Unknown Title");
            ratatui::widgets::ListItem::new(title)
        })
        .collect();

    let list = ratatui::widgets::List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 📚 Results ")
                .border_style(Style::default().fg(Color::Rgb(255, 105, 180))),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(255, 105, 180))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, result_area, &mut list_state);
}

/// Renders the list of chapters for a selected manga.
fn draw_chapter_list(app: &App, frame: &mut Frame<'_>) {
    let area = frame.area();
    // Background
    frame.render_widget(Block::default().bg(Color::Rgb(10, 10, 20)), area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let header_area = layout[0];
    let list_area = layout[1];

    let manga_title = app
        .selected_manga
        .as_ref()
        .and_then(|m| m.attributes.title.as_ref())
        .and_then(|t| t.get("en"))
        .map(|t| t.as_str())
        .unwrap_or("Unknown Manga");

    // Header
    frame.render_widget(
        Paragraph::new(format!(" Chapters for: {}", manga_title)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        ),
        header_area,
    );

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    // Chapter List
    let items: Vec<ratatui::widgets::ListItem> = app
        .chapters
        .iter()
        .map(|c| {
            let title = c
                .attributes
                .title
                .as_deref()
                .unwrap_or("Untitled Chapter");
            let vol = c.attributes.volume.as_deref().unwrap_or("?");
            let chap = c.attributes.chapter.as_deref().unwrap_or("?");
            ratatui::widgets::ListItem::new(format!(" Vol. {} Ch. {} - {}", vol, chap, title))
        })
        .collect();

    let list = ratatui::widgets::List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 📑 Select Chapter ")
                .border_style(Style::default().fg(Color::Rgb(255, 105, 180))),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(255, 105, 180))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, list_area, &mut list_state);
}

/// Renders the high-resolution two-page reading spread.
///
/// This function records the current layout areas so background tasks can
/// accurately pre-build protocols for upcoming pages.
fn draw_reading_page(app: &mut App, frame: &mut Frame<'_>) {
    let area = frame.area();
    // Background
    frame.render_widget(Block::default().bg(Color::Rgb(10, 10, 20)), area);

    let [header, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(body);

    // Record the current panel areas so background tasks can pre-build protocols at the right size.
    app.last_right_area = right;
    app.last_left_area = left;

    let page_info = if let Some(img_data) = &app.image_data {
        format!(
            "Pages {} & {} / {} - Press 'b' to go back",
            app.current_page + 1,
            app.current_page + 2,
            img_data.chapter.data.len()
        )
    } else {
        "Reading Mode - Press 'b' to go back".to_string()
    };

    let manga_display = match app.chapters.get(app.selected_index) {
        Some(chapter) => chapter
            .attributes
            .title
            .clone()
            .unwrap_or_else(|| "Untitled".to_string()),
        None => "No chapters available".to_string(),
    };
    let display = format!(" Currently reading: {}", manga_display);
    Paragraph::new(display)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .left_aligned()
        .render(header, frame.buffer_mut());
    Paragraph::new(page_info)
        .style(Style::default().fg(Color::Rgb(255, 105, 180)))
        .centered()
        .render(header, frame.buffer_mut());

    let current = app.current_page;
    let next = current + 1;

    render_panel(app, frame, right, current, false);
    render_panel(app, frame, left, next, true);
}

/// Renders one panel. Looks up the pre-built protocol from the cache; if stale or missing,
/// builds it synchronously as a fallback (rare after first render of each page).
fn render_panel(app: &mut App, frame: &mut Frame<'_>, area: Rect, page_idx: usize, is_left: bool) {
    frame.render_widget(Block::default().bg(Color::Reset), area);

    // Check if the cached protocol is still valid for this area size.
    let cache_valid = app
        .proto_cache
        .get(&(page_idx, is_left))
        .map_or(false, |(cached_area, _)| {
            cached_area.width == area.width && cached_area.height == area.height
        });

    if !cache_valid {
        // Fallback: build synchronously. This path is only hit on first display of a page
        // (before the background task finishes) or after a terminal resize.
        let img = if is_left {
            app.page_left.clone()
        } else {
            app.page_right.clone()
        };
        if let Some(img) = img {
            let picker = app
                .picker
                .clone()
                .unwrap_or_else(ratatui_image::picker::Picker::halfblocks);
            if let Ok(p) = picker.new_protocol(
                img,
                area,
                ratatui_image::Resize::Fit(Some(ratatui_image::FilterType::Triangle)),
            ) {
                app.proto_cache.insert((page_idx, is_left), (area, p));
            }
        }
    }

    if let Some((_, p)) = app.proto_cache.get(&(page_idx, is_left)) {
        let image_widget = ratatui_image::Image::new(p);
        let actual_area = p.area();
        let x_offset = if is_left {
            area.width.saturating_sub(actual_area.width) // spine-align right
        } else {
            0 // spine-align left
        };
        let y_offset = (area.height.saturating_sub(actual_area.height)) / 2;
        let render_area = Rect::new(
            area.x + x_offset,
            area.y + y_offset,
            actual_area.width,
            actual_area.height,
        );
        frame.render_widget(image_widget, render_area);
    } else {
        frame.render_widget(Paragraph::new("Loading...").centered(), area);
    }
}

/// Handles keyboard events and updates the application state.
///
/// Most transitions involve asynchronous operations (search, fetch) which are
/// managed within this function.
async fn handle_event(
    client: &MangaDexClient,
    app: &mut App,
    key: &KeyEvent,
    page_tx: &mpsc::Sender<PageMsg>,
    proto_tx: &mpsc::Sender<ProtoMsg>,
) {
    match app.screen {
        AppScreen::Splash => {
            // Transition to Search on any key
            app.screen = AppScreen::Search;
            app.selected_index = 0;
        }
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
                    let result = search_client.search(app.search_input.clone()).await;
                    if let Ok(result) = result {
                        app.search_result = result;
                        app.last_search_query = app.search_input.clone();
                    } else if let Err(e) = result {
                        eprintln!("Error: {}", e);
                    }
                } else if !app.search_result.is_empty() {
                    app.selected_manga = Some(app.search_result[app.selected_index].clone());
                    let chapter_client = client.chapter_client();

                    let manga_id = app.selected_manga.as_ref().map(|manga| &manga.id);
                    let Some(manga_id_str) = manga_id else {
                        eprintln!("Value of id was none. exiting..");
                        return;
                    };
                    let chapter_result = chapter_client
                        .fetch_chapter(manga_id_str.as_str(), Some("en"))
                        .await;
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
                    app.selected_index += 1;
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
                    app.selected_index += 1;
                }
            }
            KeyCode::Enter => {
                let image_client = client.image_client();
                let chapter_id = &app.chapters[app.selected_index].id;
                let result = image_client.fetch_image_data(chapter_id).await;
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
                    eprint!("there are no pages to display");
                    return;
                }
                app.current_page = 0;
                app.page_cache.clear();
                app.proto_cache.clear();
                app.fetched.clear();
                app.page_left = None;
                app.page_right = None;

                let img_data = img_data.clone();
                load_spread(app, &client.http_client, &img_data, page_tx, proto_tx).await;
                app.screen = AppScreen::Reading;
            }
            _ => {}
        },
        AppScreen::Reading => match key.code {
            KeyCode::Char('b') => {
                app.screen = AppScreen::ChapterList;
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if let Some(img_data) = app.image_data.clone() {
                    if app.current_page + 2 < img_data.chapter.data.len() {
                        app.current_page += 2;
                        load_spread(app, &client.http_client, &img_data, page_tx, proto_tx).await;
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if let Some(img_data) = app.image_data.clone() {
                    if app.current_page >= 2 {
                        app.current_page -= 2;
                        load_spread(app, &client.http_client, &img_data, page_tx, proto_tx).await;
                    }
                }
            }
            _ => {}
        },
    }
}

/// Serves the current spread from cache when available, spawns background fetches for
/// anything missing, and fire-and-forgets prefetch + protocol pre-building for the next spread.
///
/// # Logic
/// 1. Immediately serves right/left pages if in `app.page_cache`.
/// 2. If an image is cached but its protocol is missing or stale (due to resize),
///    spawns a `spawn_proto` task to re-build it.
/// 3. For any missing images, spawns a `spawn_fetch` task.
/// 4. Spawns background fetches for all remaining pages in the chapter.
async fn load_spread(
    app: &mut App,
    http: &reqwest::Client,
    img_data: &ImageDataResponse,
    page_tx: &mpsc::Sender<PageMsg>,
    proto_tx: &mpsc::Sender<ProtoMsg>,
) {
    let current = app.current_page;
    let next = current + 1;

    // Right panel (current page)
    if let Some(img) = app.page_cache.get(&current) {
        app.page_right = Some(img.clone());
        // Kick off proto build if the cached proto is stale or missing for this area
        if app.last_right_area != Rect::default() {
            let needs_proto = app
                .proto_cache
                .get(&(current, false))
                .map_or(true, |(a, _)| {
                    a.width != app.last_right_area.width || a.height != app.last_right_area.height
                });
            if needs_proto {
                spawn_proto(
                    app.picker.clone(),
                    img.clone(),
                    app.last_right_area,
                    current,
                    false,
                    proto_tx.clone(),
                );
            }
        }
    } else {
        app.page_right = None;
        spawn_fetch(
            http.clone(),
            build_url(img_data, current),
            current,
            page_tx.clone(),
        );
    }

    // Left panel (next page)
    if next < img_data.chapter.data.len() {
        if let Some(img) = app.page_cache.get(&next) {
            app.page_left = Some(img.clone());
            if app.last_left_area != Rect::default() {
                let needs_proto = app.proto_cache.get(&(next, true)).map_or(true, |(a, _)| {
                    a.width != app.last_left_area.width || a.height != app.last_left_area.height
                });
                if needs_proto {
                    spawn_proto(
                        app.picker.clone(),
                        img.clone(),
                        app.last_left_area,
                        next,
                        true,
                        proto_tx.clone(),
                    );
                }
            }
        } else {
            app.page_left = None;
            spawn_fetch(
                http.clone(),
                build_url(img_data, next),
                next,
                page_tx.clone(),
            );
        }
    } else {
        app.page_left = None;
    }

    // Download every remaining page in the chapter in the background.
    // `fetched` tracks which pages already have an in-flight or completed download
    // so we never duplicate work across multiple navigations.
    for i in 0..img_data.chapter.data.len() {
        if !app.page_cache.contains_key(&i) && app.fetched.insert(i) {
            spawn_fetch(http.clone(), build_url(img_data, i), i, page_tx.clone());
        }
    }
}

fn build_url(img_data: &ImageDataResponse, idx: usize) -> String {
    format!(
        "{}/data/{}/{}",
        img_data.base_url, img_data.chapter.hash, img_data.chapter.data[idx]
    )
}

/// Downloads an image in a background async task.
///
/// Image decoding is CPU-bound and is performed in a dedicated blocking thread
/// via `tokio::task::spawn_blocking` to avoid stalling the async reactor. Redundant
/// decodes are avoided by checking `app.page_cache` before calling this.
fn spawn_fetch(http: reqwest::Client, url: String, idx: usize, page_tx: mpsc::Sender<PageMsg>) {
    tokio::spawn(async move {
        let Ok(resp) = http.get(&url).send().await else {
            return;
        };
        let Ok(bytes) = resp.bytes().await else {
            return;
        };
        // Decode on a blocking thread — image::load_from_memory is CPU-intensive
        let bytes = bytes.to_vec();
        let Ok(Ok(img)) =
            tokio::task::spawn_blocking(move || image::load_from_memory(&bytes)).await
        else {
            return;
        };
        let _ = page_tx.send((idx, img)).await;
    });
}

/// Encodes a `DynamicImage` into a terminal graphics protocol in a background task.
///
/// Uses the `Triangle` filter for scaling, which provides a significant speedup (3-5x)
/// over `Lanczos3` with negligible quality loss at typical terminal resolutions.
///
/// # Arguments
/// * `picker` - The graphics engine.
/// * `img` - The decoded image to encode.
/// * `area` - The target terminal area (Rect).
/// * `idx` - The page index.
/// * `is_left` - Whether this is for the left or right panel.
/// * `proto_tx` - Channel to send the result back to the main loop.
fn spawn_proto(
    picker: Option<ratatui_image::picker::Picker>,
    img: image::DynamicImage,
    area: Rect,
    idx: usize,
    is_left: bool,
    proto_tx: mpsc::Sender<ProtoMsg>,
) {
    let picker = picker.unwrap_or_else(ratatui_image::picker::Picker::halfblocks);
    tokio::spawn(async move {
        let Ok(Ok(p)) = tokio::task::spawn_blocking(move || {
            picker.new_protocol(
                img,
                area,
                ratatui_image::Resize::Fit(Some(ratatui_image::FilterType::Triangle)),
            )
        })
        .await
        else {
            return;
        };
        let _ = proto_tx.send((idx, is_left, p)).await;
    });
}
