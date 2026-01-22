mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn add_task_with_acceptance_criteria() {
    with_initialized_repo(|temp| {
        // Add task with acceptance criteria using -a flag
        let result = run_command(&["add", "Implement feature X", "-a", "1. Users can do Y\n2. Tests pass"], &temp);
        assert!(result.success, "add with acceptance criteria should succeed: {}", result.stderr);
        let task_id = extract_task_id(&result.stdout);
        assert!(!task_id.is_empty(), "Should create a task");

        // Verify acceptance criteria appears in show output
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("Acceptance Criteria:"), "Should show acceptance criteria label");
        assert!(show.stdout.contains("1. Users can do Y"), "Should show acceptance criteria content");
        assert!(show.stdout.contains("2. Tests pass"), "Should show all acceptance criteria");
    });
}

#[test]
fn add_task_with_both_description_and_acceptance_criteria() {
    with_initialized_repo(|temp| {
        // Add task with both description and acceptance criteria
        let result = run_command(&["add", "Feature X", "-d", "Description here", "-a", "1. Criteria 1\n2. Criteria 2"], &temp);
        assert!(result.success, "add should succeed: {}", result.stderr);
        let task_id = extract_task_id(&result.stdout);

        // Verify both appear in show output
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("Description: Description here"), "Should show description");
        assert!(show.stdout.contains("Acceptance Criteria:"), "Should show acceptance criteria label");
        assert!(show.stdout.contains("1. Criteria 1"), "Should show criteria");
    });
}

#[test]
fn add_task_fails_without_acceptance_criteria() {
    with_initialized_repo(|temp| {
        // Add task without acceptance criteria should fail
        let result = run_command(&["add", "Simple task"], &temp);
        assert!(!result.success, "add without acceptance criteria should fail");
        assert!(
            result.stderr.contains("Acceptance criteria") || result.stderr.contains("acceptance criteria"),
            "Error should mention acceptance criteria: {}",
            result.stderr
        );
    });
}

#[test]
fn update_modify_acceptance_criteria_via_update() {
    with_initialized_repo(|temp| {
        // Add a task with acceptance criteria
        let add_result = run_command(&["add", "Task with criteria", "-a", "Original criteria"], &temp);
        assert!(add_result.success, "add should succeed: {}", add_result.stderr);
        let task_id = extract_task_id(&add_result.stdout);

        // Update acceptance criteria via update
        let result = run_command(&["update", &format!("task-{}", task_id), "--acceptance-criteria", "Must pass tests"], &temp);
        assert!(result.success, "update should succeed: {}", result.stderr);

        // Verify criteria was updated
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("Acceptance Criteria:"), "Should show criteria label");
        assert!(show.stdout.contains("Must pass tests"), "Should show updated criteria content");
        assert!(!show.stdout.contains("Original criteria"), "Should not show old criteria");
    });
}

#[test]
fn update_modify_acceptance_criteria() {
    with_initialized_repo(|temp| {
        // Add a task with acceptance criteria
        let add_result = run_command(&["add", "Task", "-a", "Old criteria"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Update to new criteria
        let result = run_command(&["update", &format!("task-{}", task_id), "--acceptance-criteria", "New criteria"], &temp);
        assert!(result.success, "update should succeed: {}", result.stderr);

        // Verify criteria was updated
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("New criteria"), "Should show new criteria");
        assert!(!show.stdout.contains("Old criteria"), "Should not show old criteria");
    });
}

#[test]
fn update_acceptance_criteria_with_short_flag() {
    with_initialized_repo(|temp| {
        // Add a task with criteria
        let add_result = run_command(&["add", "Task", "-a", "Original criteria"], &temp);
        assert!(add_result.success, "add should succeed: {}", add_result.stderr);
        let task_id = extract_task_id(&add_result.stdout);

        // Update using short flag -a
        let result = run_command(&["update", &format!("task-{}", task_id), "-a", "Criteria via short flag"], &temp);
        assert!(result.success, "update with -a should succeed: {}", result.stderr);

        // Verify criteria was updated
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("Criteria via short flag"), "Should show new criteria");
    });
}

#[test]
fn update_clear_acceptance_criteria() {
    with_initialized_repo(|temp| {
        // Add a task with acceptance criteria
        let add_result = run_command(&["add", "Task", "-a", "Criteria to remove"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Clear the criteria
        let result = run_command(&["update", &format!("task-{}", task_id), "--acceptance-criteria", ""], &temp);
        assert!(result.success, "update should succeed: {}", result.stderr);

        // Verify criteria is gone
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.success);
        assert!(!show.stdout.contains("Acceptance Criteria:"), "Criteria section should not appear");
        assert!(!show.stdout.contains("Criteria to remove"), "Old criteria should be gone");
    });
}

#[test]
fn acceptance_criteria_preserved_in_csv() {
    with_initialized_repo(|temp| {
        // Add task with acceptance criteria containing special chars
        let result = run_command(&["add", "Task", "-a", "Test with, comma and | pipe"], &temp);
        assert!(result.success, "add should succeed");

        // Verify CSV format handles it correctly
        let tasks_content = fs::read_to_string(temp.join(".knecht/tasks"))
            .expect("Failed to read tasks file");

        // Should properly escape/quote the content
        assert!(tasks_content.contains("Test with, comma and | pipe"), "Criteria should be in file");

        // Verify it can be read back correctly
        let task_id = extract_task_id(&result.stdout);
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
        assert!(show.stdout.contains("Test with, comma and | pipe"), "Criteria should be preserved");
    });
}
