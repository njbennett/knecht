mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn list_shows_all_tasks() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task one", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task two", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        let result = run_command(&["list"], &temp);
        assert!(result.stdout.contains(&format!("task-{}", id1)), "Should show first task ID");
        assert!(result.stdout.contains(&format!("task-{}", id2)), "Should show second task ID");
        assert!(result.stdout.contains("Task one"), "Should show first task title");
        assert!(result.stdout.contains("Task two"), "Should show second task title");
    });
}

#[test]
fn list_handles_malformed_task_file() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Write task data (now as individual files in directory)
    let tasks_dir = temp.join(".knecht/tasks");
    fs::write(tasks_dir.join("1"), "1,open,\"Good task\",,\n").expect("Failed to write test file");
    fs::write(tasks_dir.join("malformed"), "BAD LINE WITHOUT PIPES\n").expect("Failed to write test file");
    fs::write(tasks_dir.join("2"), "2,open,\"Another good task\",,\n").expect("Failed to write test file");

    // list should handle malformed files gracefully
    let result = run_command(&["list"], &temp);
    assert!(result.success, "list should succeed even with malformed files");
    assert!(result.stdout.contains("task-1"), "Should show task-1");
    assert!(result.stdout.contains("task-2"), "Should show task-2");
    assert!(result.stdout.contains("Good task"), "Should show good task");
    assert!(result.stdout.contains("Another good task"), "Should show another good task");

    cleanup_temp_dir(temp);
}

#[test]
fn list_works_with_empty_tasks_directory() {
    with_initialized_repo(|temp| {
        // Verify tasks directory exists
        let tasks_path = temp.join(".knecht/tasks");
        assert!(tasks_path.exists());
        assert!(tasks_path.is_dir(), ".knecht/tasks should be a directory");

        // list should succeed with no tasks
        let result = run_command(&["list"], &temp);
        assert!(result.success, "list should succeed with empty directory");

        // Should show usage instructions even with no tasks (helpful for agents)
        assert!(result.stdout.contains("Usage instructions:"), "Should show usage instructions");
        assert!(result.stdout.contains("knecht show task-N"), "Should mention show command");
    });
}

#[test]
fn read_tasks_with_and_without_descriptions() {
    with_initialized_repo(|temp| {
        // Create mixed tasks: some with descriptions, some without (now as individual files)
        let tasks_dir = temp.join(".knecht/tasks");
        fs::write(tasks_dir.join("1"), "1,open,\"Old task without description\",,\n").expect("Failed to write");
        fs::write(tasks_dir.join("2"), "2,open,\"New task\",\"This has a description\",\n").expect("Failed to write");
        fs::write(tasks_dir.join("3"), "3,done,\"Another old task\",,\n").expect("Failed to write");

        // list should handle both formats
        let result = run_command(&["list"], &temp);
        assert!(result.success, "list should handle mixed format");
        assert!(result.stdout.contains("task-1"), "Should show task-1");
        assert!(result.stdout.contains("task-2"), "Should show task-2");
        assert!(result.stdout.contains("task-3"), "Should show task-3");
        assert!(result.stdout.contains("Old task without description"), "Should show old format task");
        assert!(result.stdout.contains("New task"), "Should show new format task");
    });
}

#[test]
fn list_succeeds_with_empty_directory() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // After init, .knecht/tasks is already a directory
    let result = run_command(&["list"], &temp);

    // Empty directory means no tasks
    assert!(result.success, "Should succeed with empty tasks directory");
    assert!(result.stdout.contains("Usage instructions"),
        "Should show usage instructions when no tasks");

    cleanup_temp_dir(temp);
}

#[test]
fn list_handles_tasks_with_empty_lines_in_content() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Create individual task files (some may have trailing empty lines)
    let tasks_dir = temp.join(".knecht/tasks");
    fs::write(tasks_dir.join("1"), "1,open,\"First task\",,\n\n").expect("Failed to write");
    fs::write(tasks_dir.join("2"), "2,open,\"Second task\",,\n").expect("Failed to write");
    fs::write(tasks_dir.join("3"), "3,done,\"Third task\",,\n").expect("Failed to write");

    // list should skip empty lines within files
    let result = run_command(&["list"], &temp);
    assert!(result.success, "list should handle empty lines");
    assert!(result.stdout.contains("task-1"), "Should show task-1");
    assert!(result.stdout.contains("task-2"), "Should show task-2");
    assert!(result.stdout.contains("task-3"), "Should show task-3");

    cleanup_temp_dir(temp);
}

#[test]
fn list_includes_usage_instructions_for_agents() {
    with_initialized_repo(|temp| {
        // Add a task with a description
        run_command(&["add", "Test task", "-d", "Task description here", "-a", "Done"], &temp);
        
        let result = run_command(&["list"], &temp);
        assert!(result.success);
        
        // Should include instructions on how to view full details
        assert!(result.stdout.contains("knecht show task-N"),
            "list output should guide agents to use 'knecht show' for details, got: {}", result.stdout);
        
        // Should mention how to start work on a task
        assert!(result.stdout.contains("knecht start task-N"),
            "list output should guide agents to use 'knecht start', got: {}", result.stdout);
        
        // Should mention how to mark tasks complete
        assert!(result.stdout.contains("knecht done task-N"),
            "list output should guide agents to use 'knecht done', got: {}", result.stdout);
    });
}

#[test]
fn list_shows_delivered_tasks_with_distinct_marker() {
    // task-178: Delivered tasks should have a visual marker different from open tasks
    with_initialized_repo(|temp| {
        // Create tasks with all three statuses (as individual files)
        let tasks_dir = temp.join(".knecht/tasks");
        fs::write(tasks_dir.join("1"), "1,open,\"Open task\",,\n").unwrap();
        fs::write(tasks_dir.join("2"), "2,delivered,\"Delivered task\",,\n").unwrap();
        fs::write(tasks_dir.join("3"), "3,done,\"Done task\",,\n").unwrap();

        let result = run_command(&["list"], &temp);
        assert!(result.success, "list should succeed: {}", result.stderr);

        // Find the lines for each task
        let lines: Vec<&str> = result.stdout.lines().collect();
        let open_line = lines.iter().find(|l| l.contains("Open task")).expect("Should have open task line");
        let delivered_line = lines.iter().find(|l| l.contains("Delivered task")).expect("Should have delivered task line");
        let done_line = lines.iter().find(|l| l.contains("Done task")).expect("Should have done task line");

        // Open tasks should show [ ]
        assert!(open_line.contains("[ ]"), "Open task should show [ ], got: {}", open_line);

        // Done tasks should show [x]
        assert!(done_line.contains("[x]"), "Done task should show [x], got: {}", done_line);

        // Delivered tasks should NOT show [ ] (the same as open)
        // They should have a distinct marker like [>]
        assert!(!delivered_line.contains("[ ]"),
            "Delivered task should NOT show [ ] (same as open). Should have distinct marker. Got: {}",
            delivered_line);
        assert!(delivered_line.contains("[>]"),
            "Delivered task should show [>] marker, got: {}",
            delivered_line);
    });
}

#[test]
fn list_shows_claimed_tasks_with_distinct_marker() {
    // Claimed tasks should have a visual marker distinct from open/done/delivered
    with_initialized_repo(|temp| {
        // Add a task and claim it (needs acceptance criteria for start to succeed)
        let add_result = run_command(&["add", "Claimed task", "-a", "Can be started"], &temp);
        let task_id = extract_task_id(&add_result.stdout);
        run_command(&["start", &format!("task-{}", task_id)], &temp);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "list should succeed: {}", result.stderr);

        // Claimed tasks should show [~] to indicate in-progress
        let claimed_line = result.stdout.lines()
            .find(|l| l.contains("Claimed task"))
            .expect("Should have claimed task line");
        assert!(claimed_line.contains("[~]"),
            "Claimed task should show [~] marker, got: {}", claimed_line);
    });
}
