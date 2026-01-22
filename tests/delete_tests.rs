mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn delete_removes_existing_task() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task to delete", "-a", "Done"], &temp);
        run_command(&["add", "Task to keep", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);

        let result = run_command(&["delete", &format!("task-{}", id1)], &temp);
        assert!(result.success, "delete command should succeed");
        assert!(
            result.stdout.contains(&format!("Deleted task-{}", id1)),
            "Should show confirmation message, got: {}",
            result.stdout
        );

        // Verify deleted task is gone and other task remains
        let list = run_command(&["list"], &temp);
        assert!(!list.stdout.contains("Task to delete"), "Deleted task should not appear in list");
        assert!(list.stdout.contains("Task to keep"), "Other tasks should remain");
    });
}

#[test]
fn delete_accepts_id_without_prefix() {
    with_initialized_repo(|temp| {
        let add_result = run_command(&["add", "Task to delete", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Delete should accept ID without 'task-' prefix
        let result = run_command(&["delete", &task_id], &temp);
        assert!(result.success, "delete should accept ID without 'task-' prefix");
        assert!(
            result.stdout.contains(&format!("Deleted task-{}", task_id)),
            "Should show confirmation with task- prefix, got: {}",
            result.stdout
        );
    });
}

#[test]
fn delete_preserves_other_tasks() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "First task", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Second task", "-a", "Done"], &temp);
        let r3 = run_command(&["add", "Third task", "-a", "Done"], &temp);
        let _id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        let _id3 = extract_task_id(&r3.stdout);

        run_command(&["delete", &format!("task-{}", id2)], &temp);

        let list = run_command(&["list"], &temp);
        assert!(list.stdout.contains("First task"), "First task should remain");
        assert!(!list.stdout.contains("Second task"), "Second task should be deleted");
        assert!(list.stdout.contains("Third task"), "Third task should remain");
    });
}

#[test]
fn delete_works_for_done_tasks() {
    with_initialized_repo(|temp| {
        let add_result = run_command(&["add", "Completed task", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);
        run_command(&["done", &format!("task-{}", task_id)], &temp);

        let result = run_command(&["delete", &format!("task-{}", task_id)], &temp);
        assert!(result.success, "Should be able to delete done tasks");
        assert!(result.stdout.contains(&format!("Deleted task-{}", task_id)));
    });
}

#[test]
fn delete_fails_on_nonexistent_task() {
    with_initialized_repo(|temp| {
        let result = run_command(&["delete", "task-999"], &temp);
        assert!(!result.success, "delete on nonexistent task should fail");
        assert!(
            result.stderr.contains("not found") || result.stderr.contains("doesn't exist"),
            "Should have helpful error message, got: {}",
            result.stderr
        );
    });
}

#[test]
fn delete_fails_with_invalid_task_id() {
    with_initialized_repo(|temp| {
        // With alphanumeric IDs, "abc" is a valid format but will be "not found"
        let result = run_command(&["delete", "task-abc"], &temp);
        assert!(!result.success, "delete with nonexistent ID should fail");
        assert!(
            result.stderr.contains("not found") || result.stderr.contains("Not found"),
            "Should mention not found, got: {}",
            result.stderr
        );
    });
}

#[test]
fn delete_requires_task_id_argument() {
    with_initialized_repo(|temp| {
        let result = run_command(&["delete"], &temp);
        assert!(!result.success, "delete without task ID should fail");
        assert!(
            result.stderr.contains("Usage") || result.stderr.contains("required"),
            "Should show usage or mention required argument, got: {}",
            result.stderr
        );
    });
}

#[test]
fn delete_can_delete_first_task() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "First", "-a", "Done"], &temp);
        run_command(&["add", "Second", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);

        let result = run_command(&["delete", &format!("task-{}", id1)], &temp);
        assert!(result.success, "Should be able to delete first task");

        let list = run_command(&["list"], &temp);
        assert!(!list.stdout.contains("First"));
        assert!(list.stdout.contains("Second"));
    });
}

#[test]
fn delete_can_delete_last_task() {
    with_initialized_repo(|temp| {
        run_command(&["add", "First", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Last", "-a", "Done"], &temp);
        let id2 = extract_task_id(&r2.stdout);

        let result = run_command(&["delete", &format!("task-{}", id2)], &temp);
        assert!(result.success, "Should be able to delete last task");

        let list = run_command(&["list"], &temp);
        assert!(list.stdout.contains("First"));
        assert!(!list.stdout.contains("Last"));
    });
}

#[test]
fn delete_can_delete_only_task() {
    with_initialized_repo(|temp| {
        let add_result = run_command(&["add", "Only task", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        let result = run_command(&["delete", &format!("task-{}", task_id)], &temp);
        assert!(result.success, "Should be able to delete when only one task exists");

        let list = run_command(&["list"], &temp);
        assert!(!list.stdout.contains("Only task"));
    });
}

#[test]
fn delete_maintains_file_format() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task one", "-d", "Description with | pipe", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task two", "-a", "Done"], &temp);
        let r3 = run_command(&["add", "Task three", "-d", "Another description", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        let id3 = extract_task_id(&r3.stdout);
        run_command(&["done", &format!("task-{}", id2)], &temp);

        run_command(&["delete", &format!("task-{}", id2)], &temp);

        // Verify remaining tasks are still properly formatted
        let show1 = run_command(&["show", &format!("task-{}", id1)], &temp);
        assert!(show1.success);
        assert!(show1.stdout.contains("Description with | pipe"));

        let show3 = run_command(&["show", &format!("task-{}", id3)], &temp);
        assert!(show3.success);
        assert!(show3.stdout.contains("Another description"));
    });
}
