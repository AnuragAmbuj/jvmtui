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

pub struct App {
    pub should_quit: bool,
    pub current_tab: Tab,
    pub jvm_info: Option<JvmInfo>,
    pub metrics_store: Arc<RwLock<MetricsStore>>,
}

impl App {
    pub fn new(metrics_store: Arc<RwLock<MetricsStore>>) -> Self {
        Self {
            should_quit: false,
            current_tab: Tab::Overview,
            jvm_info: None,
            metrics_store,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.current_tab = self.current_tab.previous();
    }

    pub fn select_tab(&mut self, index: usize) {
        if let Some(tab) = Tab::from_index(index) {
            self.current_tab = tab;
        }
    }

    pub fn set_jvm_info(&mut self, info: JvmInfo) {
        self.jvm_info = Some(info);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(Arc::new(RwLock::new(MetricsStore::new(300))))
    }
}
