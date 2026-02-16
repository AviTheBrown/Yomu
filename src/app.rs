use crate::error::Result;
use crate::{ChapterData, MangaData};

pub struct App {
    pub screen: AppScreen,
    pub search_input: String,
    pub search_result: Vec<MangaData>,
    pub selected_manga: Option<MangaData>,
    pub chapters: Vec<ChapterData>,
    pub selected_index: usize,
    pub current_page: usize,
    pub ascii_page: Option<String>,
}
pub enum AppScreen {
    Search,
    ChapterList,
    Reading,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: AppScreen::Search,
            search_input: String::new(),
            search_result: Vec::new(),
            selected_manga: None,
            chapters: Vec::new(),
            selected_index: 0,
            current_page: 0,
            ascii_page: None,
        }
    }
}
