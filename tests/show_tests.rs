mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn show_displays_task_with_description() {
    let temp = setup_temp_dir();

    // Initialize and add a task with description
    run_command(&["init"], &temp);
    let add_result = run_command(&["add", "Task title", "-d", "This is a detailed description", "-a", "Done"], &temp);
    let task_id = extract_task_id(&add_result.stdout);

    // Run show command
    let result = run_command(&["show", &format!("task-{}", task_id)], &temp);

    assert!(result.success, "show command should succeed");
    assert!(result.stdout.contains(&format!("task-{}", task_id)), "should show task ID");
    assert!(result.stdout.contains("Task title"), "should show title");
    assert!(result.stdout.contains("This is a detailed description"), "should show description");
    assert!(result.stdout.contains("open"), "should show status");

    cleanup_temp_dir(temp);
}

#[test]
fn show_displays_task_without_description() {
    let temp = setup_temp_dir();

    // Initialize and add a task without description
    run_command(&["init"], &temp);
    let add_result = run_command(&["add", "Simple task", "-a", "Done"], &temp);
    let task_id = extract_task_id(&add_result.stdout);

    // Run show command
    let result = run_command(&["show", &format!("task-{}", task_id)], &temp);

    assert!(result.success, "show command should succeed");
    assert!(result.stdout.contains(&format!("task-{}", task_id)), "should show task ID");
    assert!(result.stdout.contains("Simple task"), "should show title");
    assert!(result.stdout.contains("open"), "should show status");

    cleanup_temp_dir(temp);
}

#[test]
fn show_fails_on_nonexistent_task() {
    let temp = setup_temp_dir();

    // Initialize but don't add any tasks
    run_command(&["init"], &temp);

    // Try to show nonexistent task
    let result = run_command(&["show", "task-999"], &temp);

    assert!(!result.success, "show should fail for nonexistent task");
    assert!(result.stderr.contains("not found") || result.stderr.contains("Task 999"),
            "should indicate task not found");

    cleanup_temp_dir(temp);
}

#[test]
fn show_fails_with_invalid_task_id() {
    let temp = setup_temp_dir();

    run_command(&["init"], &temp);

    // Try with invalid task ID format
    let result = run_command(&["show", "invalid"], &temp);

    assert!(!result.success, "show should fail for invalid task ID");

    cleanup_temp_dir(temp);
}

#[test]
fn show_requires_task_id_argument() {
    with_initialized_repo(|temp| {
        // Try show without task ID
        let result = run_command(&["show"], &temp);

        assert!(!result.success, "show should fail without task ID");
        assert!(result.stderr.contains("Usage") || result.stderr.contains("usage"),
                "should show usage message");
    });
}

#[test]
fn show_displays_blockers() {
    with_initialized_repo(|temp| {
        // Create tasks
        let r1 = run_command(&["add", "Blocked Task", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Blocker Task", "-a", "Done"], &temp);
        let r3 = run_command(&["add", "Another Blocker", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        let id3 = extract_task_id(&r3.stdout);

        // Create blocker relationships
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id3)], &temp);

        // Check show output
        let result = run_command(&["show", &format!("task-{}", id1)], &temp);
        assert!(result.success, "show command should succeed");
        assert!(result.stdout.contains("Blocked by:"), "Should have 'Blocked by:' section");
        assert!(result.stdout.contains(&format!("task-{}", id2)), "Should show blocker task ID");
        assert!(result.stdout.contains(&format!("task-{}", id3)), "Should show blocker task ID");
        assert!(result.stdout.contains("Blocker Task"), "Should show blocker task title");
    });
}

#[test]
fn show_displays_what_task_blocks() {
    with_initialized_repo(|temp| {
        // Create tasks
        let r1 = run_command(&["add", "Blocker Task", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Blocked Task A", "-a", "Done"], &temp);
        let r3 = run_command(&["add", "Blocked Task B", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        let id3 = extract_task_id(&r3.stdout);

        // task-id1 blocks both task-id2 and task-id3
        run_command(&["block", &format!("task-{}", id2), "by", &format!("task-{}", id1)], &temp);
        run_command(&["block", &format!("task-{}", id3), "by", &format!("task-{}", id1)], &temp);

        // Check show output for task-id1
        let result = run_command(&["show", &format!("task-{}", id1)], &temp);
        assert!(result.success, "show command should succeed");
        assert!(result.stdout.contains("Blocks:"), "Should have 'Blocks:' section");
        assert!(result.stdout.contains(&format!("task-{}", id2)), "Should show blocked task ID");
        assert!(result.stdout.contains(&format!("task-{}", id3)), "Should show blocked task ID");
    });
}

#[test]
fn show_indicates_blocker_status() {
    with_initialized_repo(|temp| {
        // Create tasks
        let r1 = run_command(&["add", "Blocked Task", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Open Blocker", "-a", "Done"], &temp);
        let r3 = run_command(&["add", "Done Blocker", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        let id3 = extract_task_id(&r3.stdout);

        // Create blockers
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id3)], &temp);

        // Complete one blocker
        run_command(&["done", &format!("task-{}", id3)], &temp);

        // Check show output
        let result = run_command(&["show", &format!("task-{}", id1)], &temp);
        assert!(result.success);
        assert!(result.stdout.contains(&format!("task-{}", id2)) && result.stdout.contains("open"),
                "Should show blocker as open: {}", result.stdout);
        assert!(result.stdout.contains(&format!("task-{}", id3)) && result.stdout.contains("done"),
                "Should show blocker as done: {}", result.stdout);
    });
}

#[test]
fn show_handles_blockers_file_with_empty_lines_and_malformed_entries() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task B", "-a", "Done"], &temp);
        let r3 = run_command(&["add", "Task C", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        let _id3 = extract_task_id(&r3.stdout);

        // Create blockers file with empty lines and malformed entries
        let blockers_path = temp.join(".knecht/blockers");
        fs::write(&blockers_path, format!("task-{}|task-{}\n\nmalformed-line\ntask-{}|\n|task-{}\n", id1, id2, id1, id2)).unwrap();

        // Should still parse valid entries and ignore malformed ones
        let result = run_command(&["show", &format!("task-{}", id1)], &temp);
        assert!(result.success, "show should succeed with malformed blockers file");
        assert!(result.stdout.contains(&format!("task-{}", id2)), "Should show valid blocker");
    });
}

#[test]
fn show_handles_orphaned_blocks_reference() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Blocker Task", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Blocked Task", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Create blocker relationship
        run_command(&["block", &format!("task-{}", id2), "by", &format!("task-{}", id1)], &temp);

        // Delete the blocked task (orphan the reference in "Blocks" list)
        run_command(&["delete", &format!("task-{}", id2)], &temp);

        // Show should succeed and skip the orphaned reference
        let result = run_command(&["show", &format!("task-{}", id1)], &temp);
        assert!(result.success, "show should succeed with orphaned blocks reference: {}", result.stderr);
        // Should not crash or show error - just silently skip the orphaned reference
    });
}
