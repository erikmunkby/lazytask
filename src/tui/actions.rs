use crate::domain::TaskType;

#[derive(Debug, Clone)]
pub enum Action {
    RefreshTasks,
    CheckLearningHint,
    MoveSelectionUp,
    MoveSelectionDown,
    CreateTaskRequested,
    CreateTaskSubmitted {
        title: String,
        task_type: TaskType,
        details: String,
    },
    DeleteSelected,
    UndoDelete,
    StartSelected,
    DoneSelected,
    TaskOperationSucceeded {
        message: String,
    },
    TaskOperationFailed {
        message: String,
    },
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateField {
    Title,
    Type,
    Details,
}

impl CreateField {
    pub fn next(self) -> Self {
        match self {
            Self::Title => Self::Type,
            Self::Type => Self::Details,
            Self::Details => Self::Details,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Title => Self::Title,
            Self::Type => Self::Title,
            Self::Details => Self::Type,
        }
    }
}
