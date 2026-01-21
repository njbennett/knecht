mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn can_create_and_list_a_task() {
    let temp = setup_temp_dir();

    // Initialize
    let init_result = run_command(&["init"], &temp);
    assert!(init_result.success, "init command failed: {}", init_result.stderr);
    assert!(temp.join(".knecht/tasks").exists(), ".knecht/tasks file was not created");

    // Add a task
    let add_result = run_command(&["add", "Write first test"], &temp);
    assert!(add_result.success, "add command failed: {}", add_result.stderr);
    let task_id = extract_task_id(&add_result.stdout);
    assert!(!task_id.is_empty(), "Expected task ID in output, got: {}", add_result.stdout);

    // List tasks
    let list_result = run_command(&["list"], &temp);
    assert!(list_result.success, "list command failed: {}", list_result.stderr);
    assert!(list_result.stdout.contains(&format!("task-{}", task_id)), "Expected task ID in list output");
    assert!(list_result.stdout.contains("Write first test"), "Expected task title in list output");
    assert!(list_result.stdout.contains("[ ]"), "Expected open checkbox [ ] in list output");

    cleanup_temp_dir(temp);
}

#[test]
fn add_creates_unique_alphanumeric_ids() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "First task"], &temp);
        let r2 = run_command(&["add", "Second task"], &temp);

        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // IDs are 6 alphanumeric characters
        assert_eq!(id1.len(), 6, "ID should be 6 chars, got: '{}'", id1);
        assert!(
            id1.chars().all(|c| c.is_ascii_alphanumeric()),
            "ID should be alphanumeric, got: '{}'",
            id1
        );

        assert_eq!(id2.len(), 6, "ID should be 6 chars, got: '{}'", id2);
        assert!(
            id2.chars().all(|c| c.is_ascii_alphanumeric()),
            "ID should be alphanumeric, got: '{}'",
            id2
        );

        assert_ne!(id1, id2, "IDs should be unique");
    });
}

#[test]
fn add_handles_missing_knecht_directory() {
    let temp = setup_temp_dir();
    // Don't run init - .knecht directory doesn't exist

    // add should create the directory or fail gracefully
    let result = run_command(&["add", "New task"], &temp);

    // Either it succeeds by creating the directory, or fails with a helpful error
    if !result.success {
        assert!(
            result.stderr.contains("knecht") || result.stderr.contains("directory") || result.stderr.contains("init"),
            "Error should mention knecht/directory/init, got: {}",
            result.stderr
        );
    } else {
        // If it succeeds, verify the task was created
        let list_result = run_command(&["list"], &temp);
        assert!(list_result.stdout.contains("New task"));
    }

    cleanup_temp_dir(temp);
}

#[test]
fn add_task_with_description() {
    with_initialized_repo(|temp| {
        // Add task with description using -d flag
        let result = run_command(&["add", "Implement feature X", "-d", "This is a longer description of the feature"], &temp);
        assert!(result.success, "add with description should succeed: {}", result.stderr);
        let task_id = extract_task_id(&result.stdout);
        assert!(!task_id.is_empty(), "Should create a task");

        // Verify task file contains description in proper CSV format: id,status,title,description,pain_count
        let tasks_content = fs::read_to_string(temp.join(".knecht/tasks"))
            .expect("Failed to read tasks file");

        // Expected CSV format: <id>,open,"Implement feature X","This is a longer description of the feature",
        assert!(tasks_content.contains(&format!("{},open", task_id)), "Should have CSV format with id and status");
        assert!(tasks_content.contains("Implement feature X"), "Should contain title");
        assert!(tasks_content.contains("This is a longer description of the feature"), "Should contain description");

        // List should work with tasks that have descriptions
        let list_result = run_command(&["list"], &temp);
        assert!(list_result.success, "list should work with descriptions");
        assert!(list_result.stdout.contains("Implement feature X"), "Should show task title");
    });
}

#[test]
fn add_task_without_description_still_works() {
    with_initialized_repo(|temp| {
        // Add task without description (backwards compatibility)
        let result = run_command(&["add", "Simple task"], &temp);
        assert!(result.success, "add without description should still work");

        let list_result = run_command(&["list"], &temp);
        assert!(list_result.stdout.contains("Simple task"), "Should show task");
    });
}

#[test]
fn add_with_no_args_shows_usage() {
    with_initialized_repo(|temp| {
        let result = run_command(&["add"], &temp);

        assert!(!result.success, "Should fail when add has no args");
        assert!(result.stderr.contains("Usage:") && result.stderr.contains("add"),
            "Should show add usage message, got: {}", result.stderr);
    });
}

#[test]
fn add_with_empty_title_fails() {
    with_initialized_repo(|temp| {
        // Try to add task with only description flag but no title
        let result = run_command(&["add", "-d", "some description"], &temp);

        assert!(!result.success, "Should fail when title is empty");
        // Clap requires title argument, so it shows required argument error
        assert!(result.stderr.contains("required") || result.stderr.contains("TITLE"),
            "Should show required title error, got: {}", result.stderr);
    });
}

#[test]
fn add_fails_when_tasks_file_cannot_be_written() {
    let temp = setup_temp_dir();

    // Create .knecht directory and tasks file
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");
    fs::File::create(&tasks_file).unwrap();

    // Make the tasks file read-only (no write permissions)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tasks_file).unwrap().permissions();
        perms.set_mode(0o444); // read-only
        fs::set_permissions(&tasks_file, perms).unwrap();
    }

    #[cfg(windows)]
    {
        let mut perms = fs::metadata(&tasks_file).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&tasks_file, perms).unwrap();
    }

    // Try to add a task - should fail with IO error
    let result = run_command(&["add", "This should fail"], &temp);

    assert!(!result.success, "Should fail when tasks file is not writable");
    assert!(result.stderr.contains("Error:"),
        "Should show error message, got: {}", result.stderr);

    // Clean up - restore permissions before cleanup
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tasks_file).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&tasks_file, perms).unwrap();
    }

    #[cfg(windows)]
    {
        let mut perms = fs::metadata(&tasks_file).unwrap().permissions();
        perms.set_readonly(false);
        fs::set_permissions(&tasks_file, perms).unwrap();
    }

    cleanup_temp_dir(temp);
}

#[test]
fn add_command_writes_csv_format() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Add a task with special characters
    let result = run_command(&["add", "Task with, comma and | pipe"], &temp);
    assert!(result.success, "add should succeed");

    // Verify the file is in CSV format
    let tasks_path = temp.join(".knecht/tasks");
    let content = fs::read_to_string(&tasks_path).expect("Failed to read tasks file");
    
    // Should use CSV format with quotes, not pipe-delimited with escapes
    assert!(content.contains(",open,"), "Should use CSV format with commas");
    assert!(content.contains("\"Task with, comma and | pipe\""), "Should quote fields with special chars");
    assert!(!content.contains("\\|"), "Should not use backslash escaping");

    cleanup_temp_dir(temp);
}

#[test]
fn add_output_shows_block_syntax() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    let result = run_command(&["add", "New task"], &temp);
    assert!(result.success, "add should succeed");
    let task_id = extract_task_id(&result.stdout);

    // Output should show how to make this task a blocker for another task
    assert!(
        result.stdout.contains("knecht block") && result.stdout.contains(&format!("by task-{}", task_id)),
        "add output should show block syntax, got: {}",
        result.stdout
    );

    cleanup_temp_dir(temp);
}
