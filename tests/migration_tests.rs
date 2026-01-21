mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;
#[allow(unused_imports)]
use std::io::Write;
#[allow(unused_imports)]
use std::process::{Command, Stdio};

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

#[test]
fn beads2knecht_handles_empty_task_list() {
    let temp = setup_temp_dir();

    // Create empty JSON array
    let empty_json = "[]";

    let mut child = Command::new(env!("CARGO_BIN_EXE_beads2knecht"))
        .current_dir(&temp)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn beads2knecht");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(empty_json.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for child");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "beads2knecht should succeed with empty list");
    assert!(stdout.contains("# 0 tasks found"), "Should report 0 tasks");
    assert!(stderr.contains("Tasks converted: 0"), "Should report 0 tasks converted");

    cleanup_temp_dir(temp);
}

#[test]
fn beads2knecht_handles_task_without_description() {
    let temp = setup_temp_dir();

    // Create JSON with a task that has no description
    let json_input = r#"[
        {
            "id": "beads-1",
            "title": "Task without description",
            "status": "open",
            "priority": 1,
            "issue_type": "feature"
        }
    ]"#;

    let mut child = Command::new(env!("CARGO_BIN_EXE_beads2knecht"))
        .current_dir(&temp)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn beads2knecht");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(json_input.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for child");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "beads2knecht should succeed");

    // Find the task line (skip comment lines)
    let task_lines: Vec<&str> = stdout.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .collect();

    assert_eq!(task_lines.len(), 1, "Should have exactly one task");
    // Task should have 3 fields (no description field)
    assert_eq!(task_lines[0].matches('|').count(), 2, "Task without description should have only 2 pipes");
    assert!(task_lines[0].starts_with("1|open|"), "Should be task 1 with open status");
    assert!(task_lines[0].contains("Task without description"), "Should have correct title");

    cleanup_temp_dir(temp);
}

#[test]
fn beads2knecht_handles_unknown_status() {
    let temp = setup_temp_dir();

    // Create JSON with a task that has an unknown status
    let json_input = r#"[
        {
            "id": "beads-1",
            "title": "Task with unknown status",
            "status": "blocked",
            "priority": 1,
            "issue_type": "feature"
        }
    ]"#;

    let mut child = Command::new(env!("CARGO_BIN_EXE_beads2knecht"))
        .current_dir(&temp)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn beads2knecht");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(json_input.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for child");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "beads2knecht should succeed");

    // Find the task line (skip comment lines)
    let task_lines: Vec<&str> = stdout.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .collect();

    assert_eq!(task_lines.len(), 1, "Should have exactly one task");
    // Unknown status should default to "open"
    assert!(task_lines[0].starts_with("1|open|"), "Unknown status should default to open, got: {}", task_lines[0]);

    cleanup_temp_dir(temp);
}
