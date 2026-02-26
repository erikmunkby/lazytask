use crate::services::ServiceError;
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, Serialize)]
struct SuccessEnvelope<T: Serialize> {
    ok: bool,
    data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    ok: bool,
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: String,
    message: String,
    details: Value,
}

/// Maps service-layer errors to stable machine-readable JSON error codes.
pub(super) fn map_error_code(err: &ServiceError) -> (String, Value) {
    match err {
        ServiceError::TasksRootMissing => ("tasks_root_missing".to_string(), json!({})),
        ServiceError::TaskNotFound(query) => {
            ("task_not_found".to_string(), json!({ "query": query }))
        }
        ServiceError::TaskAmbiguous { query, matches } => (
            "task_ambiguous".to_string(),
            json!({ "query": query, "matches": matches }),
        ),
        ServiceError::TaskAlreadyExists(title) => {
            ("task_already_exists".to_string(), json!({ "title": title }))
        }
        ServiceError::StatusLimitReached(message) => (
            "status_limit_reached".to_string(),
            json!({ "reason": message }),
        ),
        ServiceError::ValidationError(message) => {
            ("validation_error".to_string(), json!({ "reason": message }))
        }
        ServiceError::Io(message) => ("io_error".to_string(), json!({ "reason": message })),
        ServiceError::ParseError(message) => {
            ("parse_error".to_string(), json!({ "reason": message }))
        }
    }
}

/// Detects whether parse failures should be emitted as AI JSON envelopes.
pub(super) fn wants_ai_json_error() -> bool {
    let first_arg = std::env::args().nth(1);
    matches!(
        first_arg.as_deref(),
        Some("list")
            | Some("get")
            | Some("create")
            | Some("start")
            | Some("done")
            | Some("discard")
            | Some("delete")
            | Some("learn")
    )
}

/// Prints a standard success envelope as one-line JSON.
pub(super) fn print_json_success(data: Value, hint: Option<String>) {
    let payload = SuccessEnvelope {
        ok: true,
        data,
        hint,
    };
    println!(
        "{}",
        serde_json::to_string(&payload).expect("failed to serialize success payload")
    );
}

/// Prints a standard error envelope as one-line JSON.
pub(super) fn print_json_error(code: &str, message: &str, details: Value) {
    let payload = ErrorEnvelope {
        ok: false,
        error: ErrorBody {
            code: code.to_string(),
            message: message.to_string(),
            details,
        },
    };
    println!(
        "{}",
        serde_json::to_string(&payload).expect("failed to serialize error payload")
    );
}
