use crate::domain::TaskType;

#[derive(Debug, Clone)]
pub enum Action {
    RefreshTasks,
    CheckLearningHint,
    MoveSelectionUp,
    MoveSelectionDown,
    CreateTaskRequested,
    EditSelectedRequested,
    CreateTaskSubmitted {
        title: String,
        task_type: TaskType,
        details: String,
    },
    EditTaskSubmitted {
        file_name: String,
        title: String,
        task_type: TaskType,
        details: String,
    },
    DeleteSelected,
    UndoDelete,
    StartSelected,
    DoneSelected,
    OpenSelectedInEditor,
    TaskOperationSucceeded {
        message: String,
    },
    TaskOperationFailed {
        message: String,
    },
    UpdateAvailable {
        version: String,
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
    /// Advances focus to the next editable field without wrapping.
    pub fn next(self) -> Self {
        match self {
            Self::Title => Self::Type,
            Self::Type => Self::Details,
            Self::Details => Self::Details,
        }
    }

    /// Moves focus to the previous editable field without wrapping.
    pub fn prev(self) -> Self {
        match self {
            Self::Title => Self::Title,
            Self::Type => Self::Title,
            Self::Details => Self::Type,
        }
    }
}
