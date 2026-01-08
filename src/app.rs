use crate::jvm::types::JvmInfo;
use crate::metrics::store::MetricsStore;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Memory,
    Threads,
    GC,
    Classes,
}

impl Tab {
    pub fn next(self) -> Self {
        match self {
            Tab::Overview => Tab::Memory,
            Tab::Memory => Tab::Threads,
            Tab::Threads => Tab::GC,
            Tab::GC => Tab::Classes,
            Tab::Classes => Tab::Overview,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Tab::Overview => Tab::Classes,
            Tab::Memory => Tab::Overview,
            Tab::Threads => Tab::Memory,
            Tab::GC => Tab::Threads,
            Tab::Classes => Tab::GC,
        }
    }

    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Tab::Overview),
            1 => Some(Tab::Memory),
            2 => Some(Tab::Threads),
            3 => Some(Tab::GC),
            4 => Some(Tab::Classes),
            _ => None,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Tab::Overview => "Overview",
            Tab::Memory => "Memory",
            Tab::Threads => "Threads",
            Tab::GC => "GC",
            Tab::Classes => "Classes",
        }
    }

    pub fn all() -> [Tab; 5] {
        [
            Tab::Overview,
            Tab::Memory,
            Tab::Threads,
            Tab::GC,
            Tab::Classes,
        ]
    }
}

pub enum AppMode {
    Normal,
    Help,
    ConfirmGc,
    ConfirmExport,
    Error(String),
    Loading(String),
    ExportSuccess(String),
    Search,
}

pub struct App {
    pub should_quit: bool,
    pub current_tab: Tab,
    pub jvm_info: Option<JvmInfo>,
    pub metrics_store: Arc<RwLock<MetricsStore>>,
    pub mode: AppMode,
    pub scroll_offset: usize,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub search_index: usize,
}

impl App {
    pub fn new(metrics_store: Arc<RwLock<MetricsStore>>) -> Self {
        Self {
            should_quit: false,
            current_tab: Tab::Overview,
            jvm_info: None,
            metrics_store,
            mode: AppMode::Normal,
            scroll_offset: 0,
            search_query: String::new(),
            search_results: Vec::new(),
            search_index: 0,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
        self.scroll_offset = 0;
    }

    pub fn previous_tab(&mut self) {
        self.current_tab = self.current_tab.previous();
        self.scroll_offset = 0;
    }

    pub fn select_tab(&mut self, index: usize) {
        if let Some(tab) = Tab::from_index(index) {
            self.current_tab = tab;
            self.scroll_offset = 0;
        }
    }

    pub fn set_jvm_info(&mut self, info: JvmInfo) {
        self.jvm_info = Some(info);
    }

    pub fn toggle_help(&mut self) {
        self.mode = match self.mode {
            AppMode::Help => AppMode::Normal,
            _ => AppMode::Help,
        };
    }

    pub fn show_gc_confirmation(&mut self) {
        self.mode = AppMode::ConfirmGc;
    }

    pub fn show_export_confirmation(&mut self) {
        self.mode = AppMode::ConfirmExport;
    }

    pub fn cancel_confirmation(&mut self) {
        self.mode = AppMode::Normal;
    }

    pub fn show_export_success(&mut self, path: String) {
        self.mode = AppMode::ExportSuccess(path);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn show_error(&mut self, message: String) {
        self.mode = AppMode::Error(message);
    }

    pub fn clear_error(&mut self) {
        if matches!(self.mode, AppMode::Error(_)) {
            self.mode = AppMode::Normal;
        }
    }

    pub fn show_loading(&mut self, message: String) {
        self.mode = AppMode::Loading(message);
    }

    pub fn clear_loading(&mut self) {
        if matches!(self.mode, AppMode::Loading(_)) {
            self.mode = AppMode::Normal;
        }
    }

    pub fn start_search(&mut self) {
        self.mode = AppMode::Search;
        self.search_query.clear();
        self.search_results.clear();
        self.search_index = 0;
    }

    pub fn cancel_search(&mut self) {
        self.mode = AppMode::Normal;
        self.search_query.clear();
        self.search_results.clear();
        self.search_index = 0;
    }

    pub fn push_search_char(&mut self, c: char) {
        self.search_query.push(c);
    }

    pub fn pop_search_char(&mut self) {
        self.search_query.pop();
    }

    pub fn update_search_results(&mut self, results: Vec<usize>) {
        self.search_results = results;
        self.search_index = 0;
    }

    pub fn next_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.search_index = (self.search_index + 1) % self.search_results.len();
            if let Some(&result_offset) = self.search_results.get(self.search_index) {
                self.scroll_offset = result_offset;
            }
        }
    }

    pub fn prev_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.search_index = if self.search_index == 0 {
                self.search_results.len() - 1
            } else {
                self.search_index - 1
            };
            if let Some(&result_offset) = self.search_results.get(self.search_index) {
                self.scroll_offset = result_offset;
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(Arc::new(RwLock::new(MetricsStore::new(300))))
    }
}
