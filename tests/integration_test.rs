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

#[test]
fn rules_file_stays_under_150_directives() {
    // This test enforces a hard limit on .rules file size
    // Keeps the rules concise and forces periodic condensing
    
    const MAX_LINES: usize = 250;
    const MAX_DIRECTIVES: usize = 150;
    
    let rules_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".rules");
    
    // If .rules doesn't exist, that's fine
    if !rules_path.exists() {
        return;
    }
    
    let content = fs::read_to_string(&rules_path)
        .expect("Failed to read .rules file");
    
    // Count lines
    let lines = content.lines().count();
    
    // Count directives:
    // - Lines starting with "- " (bullets)
    // - Lines starting with digits + "." (numbered lists)
    // - Lines containing bold imperatives (MUST, NEVER, ALWAYS, DON'T, DO NOT)
    let mut directives = 0;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("- ") || trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) && trimmed.contains(". ") {
            directives += 1;
        }
        if line.contains("**") && (line.contains("MUST") || line.contains("NEVER") || 
           line.contains("ALWAYS") || line.contains("DON'T") || line.contains("DO NOT")) {
            directives += 1;
        }
    }
    
    assert!(
        lines <= MAX_LINES,
        ".rules file has {} lines (max: {}). Consider condensing:\n\
         - Remove redundant sections\n\
         - Consolidate similar directives\n\
         - Ask: 'What can agents infer from core principles?'\n\
         - Keep: Philosophy, TDD, Pain-Driven Dev, Data Format",
        lines, MAX_LINES
    );
    
    assert!(
        directives <= MAX_DIRECTIVES,
        ".rules file has {} directives (max: {}). Consider condensing:\n\
         - Remove redundant directives\n\
         - Consolidate similar rules\n\
         - Focus on core principles that imply the rest",
        directives, MAX_DIRECTIVES
    );
}

#[test]
fn list_handles_malformed_task_file() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    
    // Write malformed task data
    let tasks_path = temp.join(".knecht/tasks");
    fs::write(&tasks_path, "1|open|Good task\nBAD LINE WITHOUT PIPES\n2|open|Another good task\n")
        .expect("Failed to write test file");
    
    // list should handle malformed lines gracefully
    let result = run_command(&["list"], &temp);
    assert!(result.success, "list should succeed even with malformed lines");
    assert!(result.stdout.contains("task-1"), "Should show task-1");
    assert!(result.stdout.contains("task-2"), "Should show task-2");
    assert!(result.stdout.contains("Good task"), "Should show good task");
    assert!(result.stdout.contains("Another good task"), "Should show another good task");
    
    cleanup_temp_dir(temp);
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
fn done_handles_invalid_task_id_formats() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    run_command(&["add", "Test task"], &temp);
    
    // Test various invalid formats
    let invalid_ids = vec!["not-a-number", "task-abc", "999999", "task-999999"];
    
    for invalid_id in invalid_ids {
        let result = run_command(&["done", invalid_id], &temp);
        assert!(
            !result.success,
            "done with invalid ID '{}' should fail",
            invalid_id
        );
        assert!(
            result.stderr.contains("not found") || result.stderr.contains("Error") || result.stderr.contains("Invalid"),
            "Should have error message for invalid ID '{}', got: {}",
            invalid_id,
            result.stderr
        );
    }
    
    cleanup_temp_dir(temp);
}

#[test]
fn list_works_with_empty_tasks_file() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    
    // Verify empty file exists
    let tasks_path = temp.join(".knecht/tasks");
    assert!(tasks_path.exists());
    
    // list should succeed with no tasks
    let result = run_command(&["list"], &temp);
    assert!(result.success, "list should succeed with empty file");
    assert_eq!(result.stdout.trim(), "", "Should show no tasks");
    
    cleanup_temp_dir(temp);
}

#[test]
fn add_handles_tasks_with_pipe_characters_in_title() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    
    // Add task with pipe in title - this is tricky for pipe-delimited format
    let result = run_command(&["add", "Fix bug in foo|bar function"], &temp);
    
    // Should either handle it gracefully or reject it with clear error
    if result.success {
        let list = run_command(&["list"], &temp);
        // Verify the task appears correctly
        assert!(
            list.stdout.contains("foo") && list.stdout.contains("bar"),
            "Task with pipe should be stored/displayed somehow, got: {}",
            list.stdout
        );
    } else {
        assert!(
            result.stderr.contains("pipe") || result.stderr.contains("|") || result.stderr.contains("invalid"),
            "Should explain pipe character issue, got: {}",
            result.stderr
        );
    }
    
    cleanup_temp_dir(temp);
}