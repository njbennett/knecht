mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn update_title_only() {
    with_initialized_repo(|temp| {
        // Add a task
        let add_result = run_command(&["add", "Old Title"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Update the title
        let result = run_command(&["update", &format!("task-{}", task_id), "--title", "New Title"], &temp);
        assert!(result.success, "update command should succeed: {}", result.stderr);
        assert!(result.stdout.contains(&format!("Updated task-{}", task_id)), "Should show success message");

        // Verify the title was updated
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("New Title"), "Title should be updated");
        assert!(!show.stdout.contains("Old Title"), "Old title should be gone");

        // Verify status is preserved (should still be open)
        let list = run_command(&["list"], &temp);
        assert!(list.stdout.contains("[ ]"), "Task should still be open");
        assert!(list.stdout.contains("New Title"), "New title should appear in list");
    });
}

#[test]
fn update_description_only() {
    with_initialized_repo(|temp| {
        // Add a task with a description
        let add_result = run_command(&["add", "Task Title", "-d", "Old description"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Update only the description
        let result = run_command(&["update", &format!("task-{}", task_id), "--description", "New description"], &temp);
        assert!(result.success, "update command should succeed: {}", result.stderr);

        // Verify the description was updated but title unchanged
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("Task Title"), "Title should be unchanged");
        assert!(show.stdout.contains("New description"), "Description should be updated");
        assert!(!show.stdout.contains("Old description"), "Old description should be gone");
    });
}

#[test]
fn update_add_description_to_task_without_one() {
    with_initialized_repo(|temp| {
        // Add a task without description
        let add_result = run_command(&["add", "Task without description"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Add a description
        let result = run_command(&["update", &format!("task-{}", task_id), "--description", "New description added"], &temp);
        assert!(result.success, "update command should succeed: {}", result.stderr);

        // Verify the description was added
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("New description added"), "Description should be added");
    });
}

#[test]
fn update_both_title_and_description() {
    with_initialized_repo(|temp| {
        // Add a task with both
        let add_result = run_command(&["add", "Old Title", "-d", "Old description"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Update both
        let result = run_command(&["update", &format!("task-{}", task_id), "--title", "New Title", "--description", "New description"], &temp);
        assert!(result.success, "update command should succeed: {}", result.stderr);

        // Verify both were updated
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("New Title"), "Title should be updated");
        assert!(show.stdout.contains("New description"), "Description should be updated");
        assert!(!show.stdout.contains("Old Title"), "Old title should be gone");
        assert!(!show.stdout.contains("Old description"), "Old description should be gone");
    });
}

#[test]
fn update_with_short_flags() {
    with_initialized_repo(|temp| {
        // Add a task
        let add_result = run_command(&["add", "Old Title"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Update using short flags
        let result = run_command(&["update", &format!("task-{}", task_id), "-t", "New Title", "-d", "New description"], &temp);
        assert!(result.success, "update with short flags should succeed: {}", result.stderr);

        // Verify updates
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("New Title"), "Title should be updated");
        assert!(show.stdout.contains("New description"), "Description should be updated");
    });
}

#[test]
fn update_clear_description() {
    with_initialized_repo(|temp| {
        // Add a task with description
        let add_result = run_command(&["add", "Task Title", "-d", "Description to remove"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Clear the description
        let result = run_command(&["update", &format!("task-{}", task_id), "--description", ""], &temp);
        assert!(result.success, "update should succeed: {}", result.stderr);

        // Verify description is gone
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("Task Title"), "Title should remain");
        assert!(!show.stdout.contains("Description to remove"), "Description should be removed");
        assert!(!show.stdout.contains("Description:"), "Description field should not appear");
    });
}

#[test]
fn update_nonexistent_task() {
    with_initialized_repo(|temp| {
        // Try to update a task that doesn't exist
        let result = run_command(&["update", "task-999", "--title", "New Title"], &temp);
        assert!(!result.success, "update should fail for nonexistent task");
        assert!(result.stderr.contains("task-999"), "Error should mention the task ID");
        assert!(result.stderr.contains("not found") || result.stderr.contains("not found"), "Error should say not found");
    });
}

#[test]
fn update_no_flags_provided() {
    with_initialized_repo(|temp| {
        // Add a task
        let add_result = run_command(&["add", "Task Title"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Try to update without providing any flags
        let result = run_command(&["update", &format!("task-{}", task_id)], &temp);
        assert!(!result.success, "update should fail when no flags provided");
        assert!(result.stderr.contains("title") || result.stderr.contains("description"), "Error should mention required flags");
    });
}

#[test]
fn update_preserves_status() {
    with_initialized_repo(|temp| {
        // Add and complete a task
        let add_result = run_command(&["add", "Done Task"], &temp);
        let task_id = extract_task_id(&add_result.stdout);
        run_command(&["done", &format!("task-{}", task_id)], &temp);

        // Update the title
        let result = run_command(&["update", &format!("task-{}", task_id), "--title", "Updated Done Task"], &temp);
        assert!(result.success, "update should succeed: {}", result.stderr);

        // Verify status is still done
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("done"), "Status should still be done");
        assert!(show.stdout.contains("Updated Done Task"), "Title should be updated");
    });
}

#[test]
fn update_only_affects_target_task() {
    with_initialized_repo(|temp| {
        // Add multiple tasks
        let r1 = run_command(&["add", "Task One"], &temp);
        let r2 = run_command(&["add", "Task Two"], &temp);
        let r3 = run_command(&["add", "Task Three"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        let id3 = extract_task_id(&r3.stdout);

        // Update only task-2
        let result = run_command(&["update", &format!("task-{}", id2), "--title", "Updated Task Two"], &temp);
        assert!(result.success, "update should succeed: {}", result.stderr);

        // Verify only task-2 was changed
        let show1 = run_command(&["show", &format!("task-{}", id1)], &temp);
        assert!(show1.stdout.contains("Task One"), "Task 1 should be unchanged");

        let show2 = run_command(&["show", &format!("task-{}", id2)], &temp);
        assert!(show2.stdout.contains("Updated Task Two"), "Task 2 should be updated");

        let show3 = run_command(&["show", &format!("task-{}", id3)], &temp);
        assert!(show3.stdout.contains("Task Three"), "Task 3 should be unchanged");
    });
}

#[test]
fn update_fails_when_no_task_id_provided() {
    with_initialized_repo(|temp| {
        // Try to update without providing a task ID
        let result = run_command(&["update"], &temp);
        assert!(!result.success, "update should fail when no task ID provided");
        assert!(result.stderr.contains("Usage") || result.stderr.contains("task-id"), "Error should show usage");
    });
}

#[test]
fn update_fails_when_title_flag_has_no_value() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task Title"], &temp);
        
        // Try to update with --title but no value
        let result = run_command(&["update", "task-1", "--title"], &temp);
        assert!(!result.success, "update should fail when --title has no value");
        assert!(result.stderr.contains("title") && result.stderr.contains("value"), "Error should mention title requires value");
    });
}

#[test]
fn update_fails_when_description_flag_has_no_value() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task Title"], &temp);
        
        // Try to update with --description but no value
        let result = run_command(&["update", "task-1", "--description"], &temp);
        assert!(!result.success, "update should fail when --description has no value");
        assert!(result.stderr.contains("description") && result.stderr.contains("value"), "Error should mention description requires value");
    });
}

#[test]
fn update_fails_with_unknown_flag() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task Title"], &temp);
        
        // Try to update with an unknown flag
        let result = run_command(&["update", "task-1", "--invalid-flag", "value"], &temp);
        assert!(!result.success, "update should fail with unknown flag");
        assert!(result.stderr.contains("Unknown") || result.stderr.contains("invalid"), "Error should mention unknown flag");
    });
}

#[test]
fn update_handles_special_characters() {
    with_initialized_repo(|temp| {
        // Add a task
        let add_result = run_command(&["add", "Simple Title"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Update with special characters (pipe is tricky for our format)
        let result = run_command(&["update", &format!("task-{}", task_id), "--title", "Title with | pipe", "--description", "Description with special chars: | and newlines"], &temp);
        assert!(result.success, "update should handle special characters: {}", result.stderr);

        // Verify the special characters are preserved
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("Title with | pipe"), "Pipe in title should be preserved");
        assert!(show.stdout.contains("Description with special chars: | and newlines"), "Pipe in description should be preserved");
    });
}
