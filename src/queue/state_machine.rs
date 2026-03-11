//! Job state machine
//!
//! Defines valid state transitions for download jobs.

use super::JobStatus;

/// State machine for job status transitions.
pub struct StateMachine;

impl StateMachine {
    /// Check if a state transition is valid.
    pub fn can_transition(from: &JobStatus, to: &JobStatus) -> bool {
        use JobStatus::*;

        matches!(
            (from, to),
            // From Queued
            (Queued, Active) |
            (Queued, Stopped) |

            // From Active
            (Active, Paused) |
            (Active, Complete) |
            (Active, Failed) |
            (Active, Stopped) |

            // From Paused
            (Paused, Active) |
            (Paused, Stopped) |

            // From Failed (retry)
            (Failed, Queued) |
            (Failed, Active) |

            // From Stopped (restart)
            (Stopped, Queued) |
            (Stopped, Active)
        )
    }

    /// Get all valid next states from a given state.
    pub fn valid_transitions(from: &JobStatus) -> Vec<JobStatus> {
        use JobStatus::*;

        match from {
            Queued => vec![Active, Stopped],
            Active => vec![Paused, Complete, Failed, Stopped],
            Paused => vec![Active, Stopped],
            Complete => vec![],
            Failed => vec![Queued, Active],
            Stopped => vec![Queued, Active],
        }
    }

    /// Check if a state is a terminal state.
    pub fn is_terminal(status: &JobStatus) -> bool {
        matches!(status, JobStatus::Complete)
    }

    /// Check if a state can be retried.
    pub fn can_retry(status: &JobStatus) -> bool {
        matches!(status, JobStatus::Failed | JobStatus::Stopped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        assert!(StateMachine::can_transition(&JobStatus::Queued, &JobStatus::Active));
        assert!(StateMachine::can_transition(&JobStatus::Active, &JobStatus::Paused));
        assert!(StateMachine::can_transition(&JobStatus::Paused, &JobStatus::Active));
        assert!(StateMachine::can_transition(&JobStatus::Active, &JobStatus::Complete));
        assert!(StateMachine::can_transition(&JobStatus::Failed, &JobStatus::Queued));
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(!StateMachine::can_transition(&JobStatus::Queued, &JobStatus::Complete));
        assert!(!StateMachine::can_transition(&JobStatus::Complete, &JobStatus::Active));
        assert!(!StateMachine::can_transition(&JobStatus::Paused, &JobStatus::Complete));
    }
}
