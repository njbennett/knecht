mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn init_creates_tasks_directory() {
    let temp = setup_temp_dir();
    let result = run_command(&["init"], &temp);

    assert!(result.success, "init should succeed");
    assert!(temp.join(".knecht/tasks").exists(), ".knecht/tasks should exist");
    assert!(temp.join(".knecht/tasks").is_dir(), ".knecht/tasks should be a directory");

    cleanup_temp_dir(temp);
}

#[test]
fn init_fails_when_cannot_create_directory() {
    let temp = setup_temp_dir();

    // Create .knecht as a file instead of directory to cause create_dir_all to fail
    fs::write(temp.join(".knecht"), "").unwrap();

    let result = run_command(&["init"], &temp);

    assert!(!result.success, "Should fail when cannot create .knecht/tasks directory");
    assert!(result.stderr.contains("Failed to create .knecht/tasks directory"),
        "Should show directory creation error, got: {}", result.stderr);

    cleanup_temp_dir(temp);
}

#[test]
fn init_succeeds_when_tasks_directory_exists() {
    let temp = setup_temp_dir();

    // Create .knecht/tasks directory already
    fs::create_dir_all(temp.join(".knecht/tasks")).unwrap();

    let result = run_command(&["init"], &temp);

    // With directory-based storage, init succeeds even if directory exists (idempotent)
    assert!(result.success, "Should succeed when tasks directory already exists");

    cleanup_temp_dir(temp);
}
