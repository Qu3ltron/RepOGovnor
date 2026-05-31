use super::*;
use crate::validation::validate_transition;

#[test]
fn transition_planned_to_active_succeeds() {
    assert!(validate_transition(TaskStatus::Planned, TaskStatus::Active).is_ok());
}

#[test]
fn transition_active_to_completed_succeeds() {
    assert!(validate_transition(TaskStatus::Active, TaskStatus::Completed).is_ok());
}

#[test]
fn transition_active_to_blocked_succeeds() {
    assert!(validate_transition(TaskStatus::Active, TaskStatus::Blocked).is_ok());
}

#[test]
fn transition_active_to_deferred_succeeds() {
    assert!(validate_transition(TaskStatus::Active, TaskStatus::Deferred).is_ok());
}

#[test]
fn transition_deferred_to_planned_succeeds() {
    assert!(validate_transition(TaskStatus::Deferred, TaskStatus::Planned).is_ok());
}

#[test]
fn transition_blocked_to_active_succeeds() {
    assert!(validate_transition(TaskStatus::Blocked, TaskStatus::Active).is_ok());
}

#[test]
fn transition_completed_to_active_fails() {
    let error = validate_transition(TaskStatus::Completed, TaskStatus::Active).unwrap_err();
    assert!(error.contains("terminal"), "{error}");
}

#[test]
fn transition_cancelled_to_planned_fails() {
    let error = validate_transition(TaskStatus::Cancelled, TaskStatus::Planned).unwrap_err();
    assert!(error.contains("terminal"), "{error}");
}

#[test]
fn transition_planned_to_completed_fails() {
    let error = validate_transition(TaskStatus::Planned, TaskStatus::Completed).unwrap_err();
    assert!(error.contains("allowed transitions"), "{error}");
}

#[test]
fn transition_same_status_succeeds() {
    assert!(validate_transition(TaskStatus::Active, TaskStatus::Active).is_ok());
}
