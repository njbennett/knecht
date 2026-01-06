use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::Write;

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
    let thread_id = std::thread::current().id();
    format!("{}-{:?}", nanos, thread_id)
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
fn add_task_with_description() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Add task with description using -d flag
    let result = run_command(&["add", "Implement feature X", "-d", "This is a longer description of the feature"], &temp);
    assert!(result.success, "add with description should succeed: {}", result.stderr);
    assert!(result.stdout.contains("task-1"), "Should create task-1");

    // Verify task file contains description in proper format: id|status|title|description
    let tasks_content = fs::read_to_string(temp.join(".knecht/tasks"))
        .expect("Failed to read tasks file");
    
    // Expected format: 1|open|Implement feature X|This is a longer description of the feature
    let lines: Vec<&str> = tasks_content.lines().collect();
    assert_eq!(lines.len(), 1, "Should have exactly one task");
    
    let parts: Vec<&str> = lines[0].split('|').collect();
    assert_eq!(parts.len(), 4, "Task should have 4 fields: id|status|title|description, got: {}", lines[0]);
    assert_eq!(parts[0], "1", "ID should be 1");
    assert_eq!(parts[1], "open", "Status should be open");
    assert_eq!(parts[2], "Implement feature X", "Title should match");
    assert_eq!(parts[3], "This is a longer description of the feature", "Description should match");

    // List should work with tasks that have descriptions
    let list_result = run_command(&["list"], &temp);
    assert!(list_result.success, "list should work with descriptions");
    assert!(list_result.stdout.contains("Implement feature X"), "Should show task title");

    cleanup_temp_dir(temp);
}

#[test]
fn add_task_without_description_still_works() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Add task without description (backwards compatibility)
    let result = run_command(&["add", "Simple task"], &temp);
    assert!(result.success, "add without description should still work");

    let list_result = run_command(&["list"], &temp);
    assert!(list_result.stdout.contains("Simple task"), "Should show task");

    cleanup_temp_dir(temp);
}

#[test]
fn read_tasks_with_and_without_descriptions() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Create mixed tasks file: some with descriptions, some without
    let tasks_path = temp.join(".knecht/tasks");
    fs::write(&tasks_path, "1|open|Old task without description\n2|open|New task|This has a description\n3|done|Another old task\n")
        .expect("Failed to write test file");

    // list should handle both formats
    let result = run_command(&["list"], &temp);
    assert!(result.success, "list should handle mixed format");
    assert!(result.stdout.contains("task-1"), "Should show task-1");
    assert!(result.stdout.contains("task-2"), "Should show task-2");
    assert!(result.stdout.contains("task-3"), "Should show task-3");
    assert!(result.stdout.contains("Old task without description"), "Should show old format task");
    assert!(result.stdout.contains("New task"), "Should show new format task");

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

#[test]
fn beads2knecht_converts_basic_tasks() {
    // Sample beads JSON with basic tasks
    let beads_json = r#"[
  {
    "id": "abc123",
    "title": "First task",
    "status": "open",
    "priority": 2,
    "issue_type": "task"
  },
  {
    "id": "def456",
    "title": "Second task",
    "status": "done",
    "priority": 1,
    "issue_type": "bug"
  },
  {
    "id": "ghi789",
    "title": "In progress task",
    "status": "in_progress",
    "priority": 3,
    "issue_type": "feature"
  }
]"#;

    // Run beads2knecht with JSON on stdin
    let mut child = Command::new(env!("CARGO_BIN_EXE_beads2knecht"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn beads2knecht");

    // Write JSON to stdin
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(beads_json.as_bytes()).expect("Failed to write to stdin");
    }

    // Read output
    let output = child.wait_with_output().expect("Failed to wait for beads2knecht");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify command succeeded
    assert!(output.status.success(), "beads2knecht should succeed, stderr: {}", stderr);

    // Parse output lines (skip comment lines starting with #)
    let task_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect();

    // Should have 3 tasks
    assert_eq!(task_lines.len(), 3, "Should convert 3 tasks, got: {:?}", task_lines);

    // Verify task 1: open task with sequential ID 1
    assert!(task_lines[0].starts_with("1|open|"), "First task should be '1|open|...', got: {}", task_lines[0]);
    assert!(task_lines[0].contains("First task"), "First task should have title 'First task'");

    // Verify task 2: done task with sequential ID 2
    assert!(task_lines[1].starts_with("2|done|"), "Second task should be '2|done|...', got: {}", task_lines[1]);
    assert!(task_lines[1].contains("Second task"), "Second task should have title 'Second task'");

    // Verify task 3: in_progress mapped to open with sequential ID 3
    assert!(task_lines[2].starts_with("3|open|"), "Third task should be '3|open|...' (in_progress maps to open), got: {}", task_lines[2]);
    assert!(task_lines[2].contains("In progress task"), "Third task should have title 'In progress task'");

    // Verify stderr contains migration stats
    assert!(stderr.contains("3"), "stderr should mention 3 tasks converted");
    assert!(stderr.contains("MIGRATION COMPLETE"), "stderr should show migration complete message");
}

#[test]
fn beads2knecht_handles_tasks_with_descriptions() {
    // Sample beads JSON with descriptions
    let beads_json = r#"[
  {
    "id": "task1",
    "title": "Task with description",
    "description": "This is a detailed description",
    "status": "open",
    "priority": 1,
    "issue_type": "task"
  },
  {
    "id": "task2",
    "title": "Task without description",
    "status": "open",
    "priority": 0,
    "issue_type": "task"
  }
]"#;

    let mut child = Command::new(env!("CARGO_BIN_EXE_beads2knecht"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn beads2knecht");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(beads_json.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for beads2knecht");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "beads2knecht should succeed");

    // Parse task lines
    let task_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect();

    assert_eq!(task_lines.len(), 2, "Should convert 2 tasks");

    // Verify tasks are in knecht format with descriptions preserved
    assert_eq!(task_lines[0], "1|open|Task with description|This is a detailed description", 
               "First task should have description: {}", task_lines[0]);
    assert_eq!(task_lines[1], "2|open|Task without description", 
               "Second task should not have description: {}", task_lines[1]);

    // Verify stderr reports descriptions as preserved (not lost)
    assert!(stderr.contains("PRESERVED INFORMATION") && stderr.contains("Descriptions: 1 tasks had descriptions (preserved)"), 
            "stderr should report 1 task with preserved description, got: {}", stderr);
}

#[test]
fn beads2knecht_reports_lost_information() {
    // Sample with various priorities and issue types
    let beads_json = r#"[
  {
    "id": "t1",
    "title": "High priority bug",
    "status": "open",
    "priority": 0,
    "issue_type": "bug"
  },
  {
    "id": "t2",
    "title": "Low priority task",
    "status": "open",
    "priority": 4,
    "issue_type": "task"
  },
  {
    "id": "t3",
    "title": "Epic work",
    "status": "open",
    "priority": 2,
    "issue_type": "epic"
  }
]"#;

    let mut child = Command::new(env!("CARGO_BIN_EXE_beads2knecht"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn beads2knecht");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(beads_json.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for beads2knecht");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "beads2knecht should succeed");

    // Verify stderr reports lost information about priorities and issue types
    assert!(stderr.contains("Priority 0:"), "Should report priority 0 tasks");
    assert!(stderr.contains("Priority 2:"), "Should report priority 2 tasks");
    assert!(stderr.contains("Priority 4:"), "Should report priority 4 tasks");
    assert!(stderr.contains("bug:"), "Should report bug issue type");
    assert!(stderr.contains("task:"), "Should report task issue type");
    assert!(stderr.contains("epic:"), "Should report epic issue type");
}