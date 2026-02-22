use super::{App, CreateState, Mode};
use crate::domain::TITLE_CHAR_LIMIT;
use crate::tui::actions::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

impl App {
    pub fn on_key(&mut self, key: KeyEvent) {
        let mode = self.state.mode.clone();
        match mode {
            Mode::Normal => self.on_key_normal(key),
            Mode::Creating(create) => self.on_key_creating(key, create),
        }
    }

    fn on_key_normal(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.dispatch(Action::MoveSelectionUp),
            KeyCode::Down => self.dispatch(Action::MoveSelectionDown),
            KeyCode::Char('c') => self.dispatch(Action::CreateTaskRequested),
            KeyCode::Char('d') => self.dispatch(Action::DeleteSelected),
            KeyCode::Char('u') => self.dispatch(Action::UndoDelete),
            KeyCode::Char('s') => self.dispatch(Action::StartSelected),
            KeyCode::Char('x') => self.dispatch(Action::DoneSelected),
            KeyCode::Char('q') | KeyCode::Esc => self.dispatch(Action::Quit),
            _ => {}
        }
    }

    fn on_key_creating(&mut self, key: KeyEvent, mut create: CreateState) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        if ctrl && key.code == KeyCode::Char('c') {
            self.dispatch(Action::Quit);
            return;
        }

        if ctrl && (key.code == KeyCode::Char('s') || key.code == KeyCode::Enter) {
            if create.title.trim().len() > TITLE_CHAR_LIMIT {
                self.push_log("title must be at most 50 characters".to_string(), true);
                self.state.mode = Mode::Creating(create);
                return;
            }
            self.state.mode = Mode::Normal;
            self.dispatch(Action::CreateTaskSubmitted {
                title: create.title,
                task_type: create.task_type,
                details: create.details,
            });
            return;
        }

        match key.code {
            KeyCode::Esc => {
                self.state.mode = Mode::Normal;
            }
            KeyCode::Tab | KeyCode::Down => {
                let next = create.active_field.next();
                create.switch_to(next);
                self.state.mode = Mode::Creating(create);
            }
            KeyCode::BackTab | KeyCode::Up => {
                let prev = create.active_field.prev();
                create.switch_to(prev);
                self.state.mode = Mode::Creating(create);
            }
            KeyCode::Left => {
                create.move_cursor_left();
                self.state.mode = Mode::Creating(create);
            }
            KeyCode::Right => {
                create.move_cursor_right();
                self.state.mode = Mode::Creating(create);
            }
            KeyCode::Backspace => {
                create.delete_char();
                self.state.mode = Mode::Creating(create);
            }
            KeyCode::Enter => match create.active_field {
                crate::tui::actions::CreateField::Title
                | crate::tui::actions::CreateField::Type => {
                    let next = create.active_field.next();
                    create.switch_to(next);
                    self.state.mode = Mode::Creating(create);
                }
                crate::tui::actions::CreateField::Details => {
                    create.insert_char('\n');
                    self.state.mode = Mode::Creating(create);
                }
            },
            KeyCode::Char(ch) => {
                create.insert_char(ch);
                self.state.mode = Mode::Creating(create);
            }
            _ => {
                self.state.mode = Mode::Creating(create);
            }
        }
    }
}
