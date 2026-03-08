pub mod actions;
pub mod app;
pub mod components;
pub mod render;
mod update_check;

use crate::services::TaskService;
use anyhow::Result;
use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use crate::tui::app::App;
use crate::tui::render::render;

const REFRESH_INTERVAL: Duration = Duration::from_secs(3);

/// Runs the interactive TUI loop until quit.
pub fn run(service: TaskService, learn_threshold: usize) -> Result<()> {
    let mut guard = TerminalGuard::setup()?;
    let mut app = App::new(service, learn_threshold);
    app.dispatch(actions::Action::RefreshTasks);
    app.dispatch(actions::Action::CheckLearningHint);
    let update_rx = update_check::spawn_update_check();
    let mut last_refresh = Instant::now();

    while !app.state.should_quit {
        guard.terminal.draw(|frame| render(frame, &app.state))?;

        if let Ok(version) = update_rx.try_recv() {
            app.dispatch(actions::Action::UpdateAvailable { version });
        }

        if last_refresh.elapsed() >= REFRESH_INTERVAL {
            app.dispatch(actions::Action::RefreshTasks);
            last_refresh = Instant::now();
        }

        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
        {
            if is_open_hotkey_in_normal_mode(
                matches!(app.state.mode, crate::tui::app::Mode::Normal),
                key,
            ) {
                if let Err(err) = guard.run_suspended(|| {
                    app.dispatch(actions::Action::OpenSelectedInEditor);
                }) {
                    app.dispatch(actions::Action::TaskOperationFailed {
                        message: format!("open failed: {err}"),
                    });
                }
                continue;
            }
            app.on_key(key);
        }
    }

    Ok(())
}

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    /// Enables raw mode and switches into the alternate screen buffer.
    fn setup() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    /// Temporarily suspends the TUI to run an external terminal-bound action.
    fn run_suspended<T>(&mut self, action: impl FnOnce() -> T) -> Result<T> {
        self.suspend()?;
        let result = action();
        self.resume()?;
        Ok(result)
    }

    /// Restores normal terminal mode before launching an external action.
    fn suspend(&mut self) -> Result<()> {
        disable_raw_mode()?;
        self.terminal.backend_mut().execute(LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    /// Re-enters alternate-screen raw mode after a suspended action completes.
    fn resume(&mut self) -> Result<()> {
        enable_raw_mode()?;
        self.terminal.backend_mut().execute(EnterAlternateScreen)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }
}

/// True when the open-in-editor hotkey is pressed in normal mode.
fn is_open_hotkey_in_normal_mode(is_normal_mode: bool, key: KeyEvent) -> bool {
    is_normal_mode && key.code == KeyCode::Char('o') && key.modifiers == KeyModifiers::NONE
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = self.terminal.backend_mut().execute(LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

#[cfg(test)]
mod tests {
    use super::is_open_hotkey_in_normal_mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn open_hotkey_requires_normal_mode() {
        let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
        assert!(!is_open_hotkey_in_normal_mode(false, key));
    }

    #[test]
    fn open_hotkey_requires_exact_o_without_modifiers() {
        let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
        assert!(is_open_hotkey_in_normal_mode(true, key));

        let shifted = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::SHIFT);
        assert!(!is_open_hotkey_in_normal_mode(true, shifted));

        let other = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!is_open_hotkey_in_normal_mode(true, other));
    }
}
