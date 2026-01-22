mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn deliver_command_is_recognized() {
    with_initialized_repo(|temp| {
        // Add a task first
        let add_result = run_command(&["add", "Test task", "-a", "Done"], temp);
        assert!(add_result.success);
        let task_id = extract_task_id(&add_result.stdout);
        assert!(!task_id.is_empty(), "Should have created a task with an ID");

        // Try to deliver it
        let deliver_result = run_command(&["deliver", &format!("task-{}", task_id)], temp);

        // The command should be recognized (not "Unknown command")
        assert!(
            !deliver_result.stderr.contains("Unknown command"),
            "deliver command should be recognized, got stderr: {}",
            deliver_result.stderr
        );
    });
}

#[test]
fn deliver_requires_task_id_argument() {
    with_initialized_repo(|temp| {
        let result = run_command(&["deliver"], temp);

        assert!(!result.success);
        assert!(result.stderr.contains("Usage:") && result.stderr.contains("deliver"),
            "Should show deliver usage, got: {}", result.stderr);
    });
}

#[test]
fn deliver_changes_task_status_to_delivered() {
    with_initialized_repo(|temp| {
        let add_result = run_command(&["add", "Task to deliver", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        let result = run_command(&["deliver", &format!("task-{}", task_id)], &temp);
        assert!(result.success, "deliver command should succeed");

        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(
            show.stdout.contains("delivered"),
            "Task status should be 'delivered', got: {}",
            show.stdout
        );
    });
}

#[test]
fn deliver_fails_for_already_delivered_task() {
    with_initialized_repo(|temp| {
        let add_result = run_command(&["add", "Task to deliver twice", "-a", "Done"], temp);
        let task_id = extract_task_id(&add_result.stdout);

        // First delivery should succeed
        let first = run_command(&["deliver", &format!("task-{}", task_id)], temp);
        assert!(first.success, "First deliver should succeed");

        // Second delivery should fail
        let second = run_command(&["deliver", &format!("task-{}", task_id)], temp);
        assert!(!second.success, "Second deliver should fail");
        assert!(
            second.stderr.contains("already delivered"),
            "Error should mention task is already delivered, got: {}",
            second.stderr
        );
    });
}

#[test]
fn deliver_fails_for_already_done_task() {
    with_initialized_repo(|temp| {
        let add_result = run_command(&["add", "Task that is done", "-a", "Done"], temp);
        let task_id = extract_task_id(&add_result.stdout);
        run_command(&["done", &format!("task-{}", task_id)], temp);

        // Trying to deliver a done task should fail
        let result = run_command(&["deliver", &format!("task-{}", task_id)], temp);
        assert!(!result.success, "Deliver of done task should fail");
        assert!(
            result.stderr.contains("already done") || result.stderr.contains("already completed"),
            "Error should mention task is already done, got: {}",
            result.stderr
        );
    });
}

#[test]
fn deliver_success_message_matches_done_format() {
    // Task-191: deliver and done should have consistent success message format
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task one", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task two", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        let done_result = run_command(&["done", &format!("task-{}", id1)], &temp);
        let deliver_result = run_command(&["deliver", &format!("task-{}", id2)], &temp);

        // Both should start with "✓ task-N: Title" format
        assert!(
            done_result.stdout.contains(&format!("✓ task-{}: Task one", id1)),
            "done should output '✓ task-{}: Task one', got: {}",
            id1, done_result.stdout
        );
        assert!(
            deliver_result.stdout.contains(&format!("✓ task-{}: Task two", id2)),
            "deliver should output '✓ task-{}: Task two' (matching done format), got: {}",
            id2, deliver_result.stdout
        );
    });
}
