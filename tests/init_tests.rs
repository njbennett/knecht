mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn init_creates_tasks_file() {
    let temp = setup_temp_dir();
    let result = run_command(&["init"], &temp);

    assert!(result.success, "init should succeed");
    assert!(temp.join(".knecht/tasks").exists(), ".knecht/tasks should exist");

    cleanup_temp_dir(temp);
}

#[test]
fn init_fails_when_cannot_create_directory() {
    let temp = setup_temp_dir();

    // Create .knecht as a file instead of directory to cause create_dir_all to fail
    fs::write(temp.join(".knecht"), "").unwrap();

    let result = run_command(&["init"], &temp);

    assert!(!result.success, "Should fail when cannot create .knecht directory");
    assert!(result.stderr.contains("Failed to create .knecht directory"),
        "Should show directory creation error");

    cleanup_temp_dir(temp);
}

#[test]
fn init_fails_when_cannot_create_tasks_file() {
    let temp = setup_temp_dir();

    // Create .knecht directory, then create tasks as a directory to cause write to fail
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    fs::create_dir(temp.join(".knecht/tasks")).unwrap();

    let result = run_command(&["init"], &temp);

    assert!(!result.success, "Should fail when cannot create tasks file");
    assert!(result.stderr.contains("Failed to create tasks file"),
        "Should show tasks file creation error");

    cleanup_temp_dir(temp);
}
