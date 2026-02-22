mod dispatch;
mod input;
mod state;

use crate::services::TaskService;

pub use state::{AppState, CreateState, LogEntry, Mode};

pub(super) const LOG_CAPACITY: usize = 30;

pub struct App {
    pub(super) service: TaskService,
    pub(super) learn_threshold: usize,
    pub state: AppState,
}

impl App {
    pub fn new(service: TaskService, learn_threshold: usize) -> Self {
        Self {
            service,
            learn_threshold,
            state: AppState {
                tasks: Vec::new(),
                selected_index: 0,
                preview_text: "No tasks".to_string(),
                log_entries: std::collections::VecDeque::new(),
                last_deleted: None,
                mode: Mode::Normal,
                should_quit: false,
            },
        }
    }
}
