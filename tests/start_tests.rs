mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn start_succeeds_with_acceptance_criteria() {
    with_initialized_repo(|temp| {
        // Add a task WITH acceptance criteria
        let add_result = run_command(&["add", "Task with criteria", "-a", "Tests pass"], &temp);
        assert!(add_result.success, "Failed to add task");
        let task_id = extract_task_id(&add_result.stdout);

        // Start should succeed
        let result = run_command(&["start", &format!("task-{}", task_id)], &temp);
        assert!(result.success, "start should succeed with acceptance criteria: {}", result.stderr);
    });
}

#[test]
fn start_displays_task_details_with_description() {
    with_initialized_repo(|temp| {
        // Add a task with description and acceptance criteria
        let add_result = run_command(&["add", "Implement feature X", "-d", "This feature should do X, Y, and Z", "-a", "Feature works"], &temp);
        assert!(add_result.success, "Failed to add task");
        let task_id = extract_task_id(&add_result.stdout);

        // Start working on the task
        let result = run_command(&["start", &format!("task-{}", task_id)], &temp);

        assert!(result.success, "start command should succeed");
        assert!(result.stdout.contains(&format!("task-{}", task_id)), "should show task ID");
        assert!(result.stdout.contains("Implement feature X"), "should show task title");
        assert!(result.stdout.contains("This feature should do X, Y, and Z"), "should show task description");
    });
}

#[test]
fn start_displays_task_without_description() {
    with_initialized_repo(|temp| {
        // Add a task without description but with acceptance criteria
        let add_result = run_command(&["add", "Simple task", "-a", "Task complete"], &temp);
        assert!(add_result.success, "Failed to add task");
        let task_id = extract_task_id(&add_result.stdout);

        // Start working on the task
        let result = run_command(&["start", &format!("task-{}", task_id)], &temp);

        assert!(result.success, "start command should succeed");
        assert!(result.stdout.contains(&format!("task-{}", task_id)), "should show task ID");
        assert!(result.stdout.contains("Simple task"), "should show task title");
        assert!(!result.stdout.contains("Description:"), "should not show description label when no description");
    });
}

#[test]
fn start_requires_task_id_argument() {
    with_initialized_repo(|temp| {
        // Try start without task ID
        let result = run_command(&["start"], &temp);

        assert!(!result.success, "start should fail without task ID");
        assert!(result.stderr.contains("Usage") || result.stderr.contains("usage"),
                "should show usage message");
    });
}

#[test]
fn start_fails_on_nonexistent_task() {
    with_initialized_repo(|temp| {
        // Try to start a task that doesn't exist
        let result = run_command(&["start", "task-999"], &temp);

        assert!(!result.success, "start should fail on nonexistent task");
        assert!(result.stderr.contains("not found") || result.stderr.contains("Not found"),
                "should indicate task was not found");
    });
}

#[test]
fn start_fails_when_blocked_by_open_task() {
    with_initialized_repo(|temp| {
        // Create tasks with acceptance criteria
        let r1 = run_command(&["add", "Blocked Task", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Blocker Task", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Create blocker
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);

        // Try to start blocked task
        let result = run_command(&["start", &format!("task-{}", id1)], &temp);
        assert!(!result.success, "start should fail when task is blocked by open task");
        assert!(result.stderr.contains("Cannot start") || result.stderr.contains("blocked"),
                "Should explain why start failed: {}", result.stderr);
        assert!(result.stderr.contains(&format!("task-{}", id2)), "Should mention the blocking task");
    });
}

#[test]
fn start_succeeds_when_blocker_is_done() {
    with_initialized_repo(|temp| {
        // Create tasks with acceptance criteria
        let r1 = run_command(&["add", "Blocked Task", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Blocker Task", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Create blocker
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);

        // Complete the blocker
        run_command(&["done", &format!("task-{}", id2)], &temp);

        // Now start should succeed
        let result = run_command(&["start", &format!("task-{}", id1)], &temp);
        assert!(result.success, "start should succeed when blocker is done: {}", result.stderr);
    });
}

#[test]
fn start_succeeds_when_no_blockers() {
    with_initialized_repo(|temp| {
        let add_result = run_command(&["add", "Normal Task", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        let result = run_command(&["start", &format!("task-{}", task_id)], &temp);
        assert!(result.success, "start should succeed for task with no blockers");
    });
}

#[test]
fn start_succeeds_when_all_blockers_are_done() {
    with_initialized_repo(|temp| {
        // Create tasks with acceptance criteria
        let r1 = run_command(&["add", "Blocked Task", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Blocker 1", "-a", "Done"], &temp);
        let r3 = run_command(&["add", "Blocker 2", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        let id3 = extract_task_id(&r3.stdout);

        // Create multiple blockers
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id3)], &temp);

        // Complete both blockers
        run_command(&["done", &format!("task-{}", id2)], &temp);
        run_command(&["done", &format!("task-{}", id3)], &temp);

        // Start should succeed
        let result = run_command(&["start", &format!("task-{}", id1)], &temp);
        assert!(result.success, "start should succeed when all blockers are done: {}", result.stderr);
    });
}

#[test]
fn start_succeeds_when_blocker_task_is_deleted() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Blocked Task", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Blocker Task", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Create blocker
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);

        // Delete the blocker task (orphan the blocker reference)
        run_command(&["delete", &format!("task-{}", id2)], &temp);

        // Start should succeed (orphaned blockers are ignored)
        let result = run_command(&["start", &format!("task-{}", id1)], &temp);
        assert!(result.success, "start should succeed when blocker task is deleted: {}", result.stderr);
    });
}

#[test]
fn start_changes_task_status_to_claimed() {
    // When an agent starts a task, the status should change from "open" to "claimed"
    with_initialized_repo(|temp| {
        // Add a task with acceptance criteria
        let add_result = run_command(&["add", "Task to claim", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Start the task
        let result = run_command(&["start", &format!("task-{}", task_id)], &temp);
        assert!(result.success, "start should succeed: {}", result.stderr);

        // Verify the status changed to "claimed"
        let show_result = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show_result.stdout.contains("Status: claimed"),
            "Task status should be 'claimed' after start, got: {}", show_result.stdout);
    });
}
