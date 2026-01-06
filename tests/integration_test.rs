use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct TestResult {
    success: bool,
    stdout: String,
    stderr: String,
}

fn setup_temp_dir() -> PathBuf {
    let temp = std::env::temp_dir().join(format!("knecht-test-{}", rand_string()));
    fs::create_dir_all(&temp).unwrap();
    temp
}

fn rand_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}", nanos)
}

fn cleanup_temp_dir(path: PathBuf) {
    let _ = fs::remove_dir_all(path);
}

fn run_command(args: &[&str], working_dir: &PathBuf) -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_knecht"))
        .args(args)
        .current_dir(working_dir)
        .output()
        .expect("Failed to execute command");

    TestResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

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
    assert!(add_result.stdout.contains("task-1"), "Expected 'task-1' in output, got: {}", add_result.stdout);

    // List tasks
    let list_result = run_command(&["list"], &temp);
    assert!(list_result.success, "list command failed: {}", list_result.stderr);
    assert!(list_result.stdout.contains("task-1"), "Expected 'task-1' in list output");
    assert!(list_result.stdout.contains("Write first test"), "Expected task title in list output");
    assert!(list_result.stdout.contains("[ ]"), "Expected open checkbox [ ] in list output");

    cleanup_temp_dir(temp);
}

#[test]
fn init_creates_tasks_file() {
    let temp = setup_temp_dir();
    let result = run_command(&["init"], &temp);
    
    assert!(result.success, "init should succeed");
    assert!(temp.join(".knecht/tasks").exists(), ".knecht/tasks should exist");
    
    cleanup_temp_dir(temp);
}

#[test]
fn add_creates_sequential_ids() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    let r1 = run_command(&["add", "First task"], &temp);
    assert!(r1.stdout.contains("task-1"), "First task should be task-1");

    let r2 = run_command(&["add", "Second task"], &temp);
    assert!(r2.stdout.contains("task-2"), "Second task should be task-2");

    cleanup_temp_dir(temp);
}

#[test]
fn list_shows_all_tasks() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    run_command(&["add", "Task one"], &temp);
    run_command(&["add", "Task two"], &temp);

    let result = run_command(&["list"], &temp);
    assert!(result.stdout.contains("task-1"), "Should show task-1");
    assert!(result.stdout.contains("task-2"), "Should show task-2");
    assert!(result.stdout.contains("Task one"), "Should show first task title");
    assert!(result.stdout.contains("Task two"), "Should show second task title");

    cleanup_temp_dir(temp);
}

#[test]
fn done_marks_task_complete() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    run_command(&["add", "Task to complete"], &temp);

    let result = run_command(&["done", "task-1"], &temp);
    assert!(result.success, "done command should succeed");

    let list = run_command(&["list"], &temp);
    assert!(
        list.stdout.contains("[x]") || list.stdout.contains("✓"),
        "Completed task should show [x] or ✓, got: {}",
        list.stdout
    );

    cleanup_temp_dir(temp);
}

#[test]
fn done_on_nonexistent_task_fails_gracefully() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    let result = run_command(&["done", "task-999"], &temp);
    assert!(!result.success, "done on nonexistent task should fail");
    assert!(
        result.stderr.contains("not found") || result.stderr.contains("doesn't exist"),
        "Should have helpful error message, got: {}",
        result.stderr
    );

    cleanup_temp_dir(temp);
}