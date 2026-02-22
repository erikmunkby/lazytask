pub mod actions;
pub mod app;
pub mod components;
pub mod render;

use crate::services::TaskService;
use anyhow::Result;
use crossterm::ExecutableCommand;
use crossterm::event::{self, Event};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Stdout};
use std::time::Duration;

use crate::tui::app::App;
use crate::tui::render::render;

pub fn run(service: TaskService, learn_threshold: usize) -> Result<()> {
    let mut guard = TerminalGuard::setup()?;
    let mut app = App::new(service, learn_threshold);
    app.dispatch(actions::Action::RefreshTasks);
    app.dispatch(actions::Action::CheckLearningHint);

    while !app.state.should_quit {
        guard.terminal.draw(|frame| render(frame, &app.state))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key);
            }
        }
    }

    Ok(())
}

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn setup() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = self.terminal.backend_mut().execute(LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
