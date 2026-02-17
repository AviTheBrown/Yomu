use yomu::ChapterData;
use yomu::MangaData;

/// The main application state.
pub struct App {
    /// The current screen being displayed.
    pub screen: AppScreen,
    /// The current search query input by the user.
    pub search_input: String,
    /// The results of the most recent manga search.
    pub search_result: Vec<MangaData>,
    /// The manga currently selected by the user.
    pub selected_manga: Option<MangaData>,
    /// The user's preferred language for manga content.
    pub peferred_lang: String,
    /// The list of chapters for the selected manga.
    pub chapters: Vec<ChapterData>,
    /// The index of the currently selected item in a list (search results or chapters).
    pub selected_index: usize,
    /// The current page index when reading a chapter.
    pub current_page: usize,
    /// The ASCII art representation of the current page.
    pub ascii_page: Option<String>,
}
/// The different screens in the application.
pub enum AppScreen {
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
            screen: AppScreen::Search,
            search_input: String::new(),
            search_result: Vec::new(),
            peferred_lang: String::new(),
            selected_manga: None,
            chapters: Vec::new(),
            selected_index: 0,
            current_page: 0,
            ascii_page: None,
        }
    }
}
