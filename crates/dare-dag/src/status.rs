//! Canonical task status wire values (microplano 026).

use dare_core::{CoreError, CoreResult};
use std::fmt;

/// Runtime task status; wire strings are case-sensitive uppercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    Running,
    Done,
    Failed,
    Skipped,
}

impl TaskStatus {
    /// Exact wire form: `PENDING` | `RUNNING` | `DONE` | `FAILED` | `SKIPPED`.
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "PENDING",
            TaskStatus::Running => "RUNNING",
            TaskStatus::Done => "DONE",
            TaskStatus::Failed => "FAILED",
            TaskStatus::Skipped => "SKIPPED",
        }
    }

    /// Parse a wire status string (case-sensitive).
    pub fn parse(s: &str) -> CoreResult<Self> {
        match s {
            "PENDING" => Ok(TaskStatus::Pending),
            "RUNNING" => Ok(TaskStatus::Running),
            "DONE" => Ok(TaskStatus::Done),
            "FAILED" => Ok(TaskStatus::Failed),
            "SKIPPED" => Ok(TaskStatus::Skipped),
            _ => Err(CoreError::invalid_input("unknown task status")),
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_roundtrip() {
        let all = [
            TaskStatus::Pending,
            TaskStatus::Running,
            TaskStatus::Done,
            TaskStatus::Failed,
            TaskStatus::Skipped,
        ];
        for st in all {
            let wire = st.as_str();
            assert_eq!(TaskStatus::parse(wire).unwrap(), st);
            assert_eq!(st.to_string(), wire);
        }
        assert_eq!(TaskStatus::Pending.as_str(), "PENDING");
        assert_eq!(TaskStatus::Running.as_str(), "RUNNING");
        assert_eq!(TaskStatus::Done.as_str(), "DONE");
        assert_eq!(TaskStatus::Failed.as_str(), "FAILED");
        assert_eq!(TaskStatus::Skipped.as_str(), "SKIPPED");

        let err = TaskStatus::parse("pending").unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)));
        assert!(err.to_string().contains("unknown task status"));

        let err = TaskStatus::parse("UNKNOWN").unwrap_err();
        assert!(err.to_string().contains("unknown task status"));
    }
}
