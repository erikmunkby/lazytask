use crate::services::{ServiceError, TaskService};
use chrono::{Duration, Utc};

impl TaskService {
    pub fn cleanup_expired_terminal_tasks(&self) -> Result<usize, ServiceError> {
        let ttl_days = self.config.retention.done_discard_ttl_days as i64;
        let cutoff = Utc::now() - Duration::days(ttl_days);
        Ok(self.storage.delete_terminal_tasks_updated_before(cutoff)?)
    }
}
