use super::{DomainError, TITLE_CHAR_LIMIT, normalize_file_name};

pub fn validate_title(title: &str) -> Result<(), DomainError> {
    if title.trim().is_empty() {
        return Err(DomainError::ValidationError(
            "title is required".to_string(),
        ));
    }

    if title.trim().len() > TITLE_CHAR_LIMIT {
        return Err(DomainError::ValidationError(
            "title must be at most 50 characters".to_string(),
        ));
    }

    if normalize_file_name(title)?.is_empty() {
        return Err(DomainError::ValidationError(
            "title must produce a valid file name".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_details(details: &str, required: bool) -> Result<(), DomainError> {
    if required && details.trim().is_empty() {
        return Err(DomainError::ValidationError(
            "details is required".to_string(),
        ));
    }

    Ok(())
}
