use std::collections::HashMap;
use std::sync::Arc;
use ratatui::layout::Rect;
use ratatui_image::protocol::Protocol;
use tokio::sync::Semaphore;
use yomu::{ChapterData, ImageDataResponse, MangaData};

/// Maximum number of concurrent background image downloads.
pub const MAX_CONCURRENT_FETCHES: usize = 8;
/// Maximum number of decoded pages to keep in memory at once.
pub const MAX_CACHE_PAGES: usize = 20;

/// The main application state.
pub struct App {
    /// The current screen being displayed.
    pub screen: AppScreen,
    /// The current search query input by the user.
    pub search_input: String,
    /// The results of the most recent manga search.
    pub search_result: Vec<MangaData>,
    /// The last search query performed to avoid redundant searches.
    pub last_search_query: String,
    /// The manga currently selected by the user.
    pub selected_manga: Option<MangaData>,
    /// The list of chapters for the selected manga.
    pub chapters: Vec<ChapterData>,
    /// The index of the currently selected item in a list (search results or chapters).
    pub selected_index: usize,
    /// Metadata for the current chapter's images.
    pub image_data: Option<ImageDataResponse>,
    /// The current page index when reading a chapter.
    pub current_page: usize,
    /// The currently decoded image for the left panel.
    pub page_left: Option<image::DynamicImage>,
    /// The currently decoded image for the right panel.
    pub page_right: Option<image::DynamicImage>,
    /// The image rendering engine for the terminal.
    pub picker: Option<ratatui_image::picker::Picker>,
    /// In-memory cache for decoded manga pages to enable instant navigation.
    pub page_cache: HashMap<usize, image::DynamicImage>,
    /// Pre-built protocol cache keyed by (page_index, is_left_panel).
    /// Stores (area_used_for_encoding, protocol) so stale entries can be detected
    /// when the terminal is resized.
    pub proto_cache: HashMap<(usize, bool), (Rect, Protocol)>,
    /// The right panel area from the most recent render frame.
    pub last_right_area: Rect,
    /// The left panel area from the most recent render frame.
    pub last_left_area: Rect,
    /// Tracks which page indices have an in-flight or completed download
    /// to prevent duplicate network requests.
    pub fetched: std::collections::HashSet<usize>,
    /// Tracks page indices whose download permanently failed so the UI can
    /// show an error instead of a perpetual "Loading…" spinner.
    pub failed: std::collections::HashSet<usize>,
    /// Limits the number of concurrent background image downloads to avoid
    /// flooding the CDN and triggering rate-limiting.
    pub fetch_semaphore: Arc<Semaphore>,
}

/// The different screens in the application.
pub enum AppScreen {
    /// The opening splash screen.
    Splash,
    /// The search input and results screen.
    Search,
    /// The list of chapters for a selected manga.
    ChapterList,
    /// The screen for reading a specific chapter.
    Reading,
}

impl App {
    /// Creates a new `App` with default state.
    pub fn new() -> Self {
        Self {
            screen: AppScreen::Splash,
            search_input: String::new(),
            search_result: Vec::new(),
            last_search_query: String::new(),
            selected_manga: None,
            chapters: Vec::new(),
            image_data: None,
            selected_index: 0,
            current_page: 0,
            page_left: None,
            page_right: None,
            picker: None,
            page_cache: HashMap::new(),
            proto_cache: HashMap::new(),
            last_right_area: Rect::default(),
            last_left_area: Rect::default(),
            fetched: std::collections::HashSet::new(),
            failed: std::collections::HashSet::new(),
            fetch_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_FETCHES)),
        }
    }
}
