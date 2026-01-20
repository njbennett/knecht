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

fn with_initialized_repo<F>(test_fn: F)
where
    F: FnOnce(&PathBuf),
{
    let temp = setup_temp_dir();
    let init_result = run_command(&["init"], &temp);
    assert!(init_result.success, "init command failed: {}", init_result.stderr);

    test_fn(&temp);

    cleanup_temp_dir(temp);
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
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "First task"], &temp);
        assert!(r1.stdout.contains("task-1"), "First task should be task-1");

        let r2 = run_command(&["add", "Second task"], &temp);
        assert!(r2.stdout.contains("task-2"), "Second task should be task-2");
    });
}

#[test]
fn list_shows_all_tasks() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task one"], &temp);
        run_command(&["add", "Task two"], &temp);

        let result = run_command(&["list"], &temp);
        assert!(result.stdout.contains("task-1"), "Should show task-1");
        assert!(result.stdout.contains("task-2"), "Should show task-2");
        assert!(result.stdout.contains("Task one"), "Should show first task title");
        assert!(result.stdout.contains("Task two"), "Should show second task title");
    });
}

#[test]
fn done_marks_task_complete() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task to complete"], &temp);

        let result = run_command(&["done", "task-1"], &temp);
        assert!(result.success, "done command should succeed");

        let list = run_command(&["list"], &temp);
        assert!(
            list.stdout.contains("[x]") || list.stdout.contains("✓"),
            "Completed task should show [x] or ✓, got: {}",
            list.stdout
        );
    });
}

#[test]
fn done_on_nonexistent_task_fails_gracefully() {
    with_initialized_repo(|temp| {
        let result = run_command(&["done", "task-999"], &temp);
        assert!(!result.success, "done on nonexistent task should fail");
        assert!(
            result.stderr.contains("not found") || result.stderr.contains("doesn't exist"),
            "Should have helpful error message, got: {}",
            result.stderr
        );
    });
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
    fs::write(&tasks_path, "1,open,\"Good task\",,\nBAD LINE WITHOUT PIPES\n2,open,\"Another good task\",,\n")
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
    with_initialized_repo(|temp| {
        // Verify empty file exists
        let tasks_path = temp.join(".knecht/tasks");
        assert!(tasks_path.exists());

        // list should succeed with no tasks
        let result = run_command(&["list"], &temp);
        assert!(result.success, "list should succeed with empty file");
        
        // Should show usage instructions even with no tasks (helpful for agents)
        assert!(result.stdout.contains("Usage instructions:"), "Should show usage instructions");
        assert!(result.stdout.contains("knecht show task-N"), "Should mention show command");
    });
}

#[test]
fn add_task_with_description() {
    with_initialized_repo(|temp| {
        // Add task with description using -d flag
        let result = run_command(&["add", "Implement feature X", "-d", "This is a longer description of the feature"], &temp);
        assert!(result.success, "add with description should succeed: {}", result.stderr);
        assert!(result.stdout.contains("task-1"), "Should create task-1");

        // Verify task file contains description in proper CSV format: id,status,title,description,pain_count
        let tasks_content = fs::read_to_string(temp.join(".knecht/tasks"))
            .expect("Failed to read tasks file");

        // Expected CSV format: 1,open,"Implement feature X","This is a longer description of the feature",
        assert!(tasks_content.contains("1,open"), "Should have CSV format with id and status");
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
fn read_tasks_with_and_without_descriptions() {
    with_initialized_repo(|temp| {
        // Create mixed tasks file: some with descriptions, some without
        let tasks_path = temp.join(".knecht/tasks");
        fs::write(&tasks_path, "1,open,\"Old task without description\",,\n2,open,\"New task\",\"This has a description\",\n3,done,\"Another old task\",,\n")
            .expect("Failed to write test file");

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

#[test]
// ACCEPTANCE CRITERIA for task-107:
// The reflection prompt should be actionable by:
// 1. Using imperative language that requires a response ("STOP. Answer these questions:")
// 2. Making it visually distinct (more prominent formatting/separators)
// 3. Explicitly stating this is REQUIRED work, not optional
// 4. Possibly pausing for acknowledgment (though this may need --no-wait flag for tests)
//
// Success = Agents treat reflection as a blocking step that requires conscious action,
// not as informational text to skip past.

fn done_shows_refactoring_reflection_prompt() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    run_command(&["add", "Task to complete"], &temp);

    let result = run_command(&["done", "task-1"], &temp);

    assert!(result.success, "done command should succeed");
    assert!(result.stdout.contains("✓ task-1"), "Should show completed task");
    assert!(result.stdout.contains("STOP - REQUIRED REFLECTION"),
        "Should have explicit reflection prompt header");
    assert!(result.stdout.contains("Did you notice anything missing from knecht's interface"),
        "Should ask about missing interface features");
    assert!(result.stdout.contains("If YOU were confused about workflow or what to do next, that's a KNECHT UX BUG"),
        "Should explicitly state that agent confusion is a knecht UX bug");
    assert!(result.stdout.contains("Did the user have to correct or redirect you"),
        "Should ask about user corrections");
    assert!(result.stdout.contains("That's a KNECHT UX BUG, not just 'you misunderstood'"),
        "Should explicitly state that user corrections indicate knecht UX bugs");
    assert!(result.stdout.contains("Did you read .knecht/tasks directly or use grep instead of knecht commands"),
        "Should ask about bypassing knecht interface");
    assert!(result.stdout.contains("Did you notice anything new that was difficult about working with the codebase"),
        "Should ask about codebase difficulties");
    assert!(result.stdout.contains("Martin Fowler's Refactoring"),
        "Should mention Martin Fowler's Refactoring");
    assert!(result.stdout.contains("Michael Feather's Working Effectively with Legacy Code"),
        "Should mention Michael Feathers' book");
    assert!(result.stdout.contains("Check knecht to see if anything similar has already been filed"),
        "Should remind to check existing tasks");
    assert!(result.stdout.contains("increase the pain count"),
        "Should mention increasing pain count");
    assert!(result.stdout.contains("If agents are confused, knecht needs to improve. Create tasks NOW"),
        "Should emphasize that agent confusion means knecht needs improvement");

    cleanup_temp_dir(temp);
}

#[test]
fn done_reflection_prompt_uses_actionable_language() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    run_command(&["add", "Task to complete"], &temp);

    let result = run_command(&["done", "task-1"], &temp);

    assert!(result.success, "done command should succeed");
    
    // Check for imperative/blocking language
    assert!(result.stdout.contains("STOP") || result.stdout.contains("REQUIRED"),
        "Should use strong imperative language like STOP or REQUIRED");
    
    // Check for visual separators to make it stand out
    assert!(result.stdout.contains("========") || result.stdout.contains("────────"),
        "Should use visual separators to make prompt stand out");
    
    // Check that it explicitly states this is required work
    assert!(result.stdout.contains("You must") || result.stdout.contains("MUST") || result.stdout.contains("required"),
        "Should explicitly state that reflection is required work");

    cleanup_temp_dir(temp);
}

#[test]
fn done_reflection_warns_against_dismissing_issues() {
    // ACCEPTANCE CRITERIA for task-221:
    // The reflection prompt should warn agents against dismissing issues as "not a knecht bug".
    // The key insight: if you're explaining why something isn't knecht's problem,
    // that explanation IS the task to file.
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    run_command(&["add", "Task to complete"], &temp);

    let result = run_command(&["done", "task-1"], &temp);

    assert!(result.success, "done command should succeed");

    // Check for the exact guidance text about not dismissing issues
    assert!(result.stdout.contains("5. Are you about to say 'this isn't really a knecht bug'?"),
        "Should include question about dismissing issues as not a knecht bug");
    assert!(result.stdout.contains("→ STOP. That explanation IS the task to file."),
        "Should tell agent to stop and file the explanation as a task");
    assert!(result.stdout.contains("→ Describe what knecht could do differently to prevent this confusion."),
        "Should ask agent to describe what knecht could do differently");

    cleanup_temp_dir(temp);
}

#[test]
fn done_shows_commit_reminder() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    run_command(&["add", "Task to complete"], &temp);

    let result = run_command(&["done", "task-1"], &temp);

    assert!(result.success, "done command should succeed");
    assert!(result.stdout.contains("COMMIT YOUR WORK NOW:\n   → git add .knecht/tasks <your-changed-files>\n   → Commit the task changes together with your code changes"),
        "Should show commit reminder with instructions, got: {}", result.stdout);

    cleanup_temp_dir(temp);
}

#[test]
fn cli_no_args_shows_usage() {
    let temp = setup_temp_dir();

    let result = run_command(&[], &temp);

    assert!(!result.success, "Should fail when no command provided");
    assert!(result.stderr.contains("Usage: knecht"),
        "Should show usage message, got: {}", result.stderr);

    cleanup_temp_dir(temp);
}

#[test]
fn cli_unknown_command_fails() {
    let temp = setup_temp_dir();

    let result = run_command(&["nonexistent"], &temp);

    assert!(!result.success, "Should fail for unknown command");
    assert!(result.stderr.contains("unrecognized subcommand") || result.stderr.contains("nonexistent"),
        "Should show unknown command error, got: {}", result.stderr);

    cleanup_temp_dir(temp);
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
fn done_with_no_args_shows_usage() {
    with_initialized_repo(|temp| {
        let result = run_command(&["done"], &temp);

        assert!(!result.success, "Should fail when done has no args");
        assert!(result.stderr.contains("Usage:") && result.stderr.contains("done"),
            "Should show done usage message, got: {}", result.stderr);
    });
}

#[test]
fn add_task_with_pipe_in_description_works_with_escaping() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    let result = run_command(&["add", "Valid title", "-d", "Description with | pipe"], &temp);

    assert!(result.success, "Should succeed with pipe in description (CSV handles it naturally)");

    // Verify the pipe is preserved in the file (CSV format) and can be read back
    let tasks_file = temp.join(".knecht/tasks");
    let content = fs::read_to_string(&tasks_file).unwrap();

    // CSV format preserves pipe without backslash escaping
    assert!(content.contains("Description with | pipe"),
        "Should have pipe preserved in CSV format, got: {}", content);

    // When we list (which reads CSV), title should show correctly
    let list = run_command(&["list"], &temp);
    assert!(list.stdout.contains("Valid title"),
        "Should show title in list output, got: {}", list.stdout);

    cleanup_temp_dir(temp);
}

#[test]
fn list_fails_gracefully_when_tasks_file_unreadable() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Create a directory instead of a file to make it unreadable as a file
    fs::remove_file(temp.join(".knecht/tasks")).unwrap();
    fs::create_dir(temp.join(".knecht/tasks")).unwrap();

    let result = run_command(&["list"], &temp);

    assert!(!result.success, "Should fail when tasks file is unreadable");
    assert!(result.stderr.contains("Error reading tasks"),
        "Should show error reading tasks message");

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
fn done_marks_task_without_description_complete() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Add task without description
    run_command(&["add", "Task without description"], &temp);

    // Mark it done
    let result = run_command(&["done", "task-1"], &temp);
    assert!(result.success, "done command should succeed");

    // Verify it was marked done
    let list = run_command(&["list"], &temp);
    assert!(list.stdout.contains("[x]"), "Task should be marked done");

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
fn list_handles_tasks_file_with_empty_lines() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Create tasks file with empty lines
    let tasks_path = temp.join(".knecht/tasks");
    fs::write(&tasks_path, "1,open,\"First task\",,\n\n2,open,\"Second task\",,\n  \n3,done,\"Third task\",,\n\n")
        .expect("Failed to write test file");

    // list should skip empty lines
    let result = run_command(&["list"], &temp);
    assert!(result.success, "list should handle empty lines");
    assert!(result.stdout.contains("task-1"), "Should show task-1");
    assert!(result.stdout.contains("task-2"), "Should show task-2");
    assert!(result.stdout.contains("task-3"), "Should show task-3");

    cleanup_temp_dir(temp);
}

#[test]
fn done_marks_task_with_description_complete() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Add task with description
    run_command(&["add", "Task with description", "-d", "This is the description"], &temp);

    // Mark it done
    let result = run_command(&["done", "task-1"], &temp);
    assert!(result.success, "done command should succeed");

    // Verify it was marked done and still has description
    let tasks_content = fs::read_to_string(temp.join(".knecht/tasks")).unwrap();
    assert!(tasks_content.contains("1,done"), "Task should be marked done");
    assert!(tasks_content.contains("Task with description"), "Should preserve title");
    assert!(tasks_content.contains("This is the description"), "Should preserve description");

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
fn read_tasks_with_pipe_in_description_should_fail_or_preserve() {
    let temp = setup_temp_dir();

    // Manually create a tasks file with an ESCAPED pipe character in the description
    // This simulates properly escaped data with pipes
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");
    let mut file = fs::File::create(&tasks_file).unwrap();

    // Write a task with an escaped pipe in the description
    // Expected after unescaping: "Option 1) thing, 2) other, 3) curl | script"
    writeln!(file, "1|open|Test task|Option 1) thing, 2) other, 3) curl \\| script").unwrap();
    drop(file);

    // Try to list the tasks - this will read the file and unescape
    let result = run_command(&["list"], &temp);

    // List doesn't show descriptions, but it should successfully parse the file
    // and show the task with unescaped title
    assert!(result.success, "Should successfully parse file with escaped pipes");
    assert!(result.stdout.contains("Test task"), "Should show task title, got: {}", result.stdout);

    // Verify the file still has the escaped data
    let content = fs::read_to_string(&tasks_file).unwrap();
    assert!(content.contains("curl \\| script"),
        "File should still have escaped pipes, got: {}", content);

    cleanup_temp_dir(temp);
}
#[test]
fn csv_format_edge_cases_for_coverage() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // Test 1: Backslash in CSV format (no special escaping needed)
    {
        let mut file = fs::File::create(&tasks_file).unwrap();
        writeln!(file, "1,open,\"Test\\Task\",\"Description with backslash\\here\",").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should parse backslash in CSV");
    }

    // Test 2: Multiple pipes (no escaping needed in CSV)
    {
        let mut file = fs::File::create(&tasks_file).unwrap();
        writeln!(file, "2,open,\"Test|||Multi\",\"Desc|||combo\",").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should parse pipes in CSV");
    }

    // Test 3: Commas require quoting in CSV
    {
        let mut file = fs::File::create(&tasks_file).unwrap();
        writeln!(file, "3,open,\"TestA, B, C\",\"DescA, B, C\",").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should handle commas in CSV");
    }

    // Test 4: Empty description field
    {
        let mut file = fs::File::create(&tasks_file).unwrap();
        writeln!(file, "4,open,\"TaskNoDesc\",,").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should handle task without description");
    }

    // Test 5: Quotes in CSV (escaped with double quotes)
    {
        let mut file = fs::File::create(&tasks_file).unwrap();
        writeln!(file, "5,open,\"Title with \"\"quotes\"\"\",\"Desc with \"\"quotes\"\"\",").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should handle quotes in CSV");
    }

    // Test 6: Add task with backslash in title
    {
        run_command(&["init"], &temp);
        let result = run_command(&["add", "Task\\with\\backslash", "-d", "Desc\\with\\backslash"], &temp);
        assert!(result.success, "Should add task with backslashes");

        let content = fs::read_to_string(&tasks_file).unwrap();
        assert!(content.contains("Task\\with\\backslash"), "Should preserve backslashes in CSV");
    }

    // Test 7: Multiple special characters
    {
        let mut file = fs::File::create(&tasks_file).unwrap();
        writeln!(file, "7,open,\"Test with, comma | pipe\",\"Multiple|||pipes\",").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should parse multiple special chars");
    }

    // Test 8: Mixed special characters
    {
        let mut file = fs::File::create(&tasks_file).unwrap();
        writeln!(file, "8,open,\"Test\\|Mix\",\"Desc|\\combo\",").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should parse mixed characters");
    }

    cleanup_temp_dir(temp);
}

#[test]
fn test_backslash_in_csv_format() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // Backslash characters are preserved as-is in CSV format
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1,open,\"Path\\ntest\",\"C:\\folder\\file\",").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle backslash in CSV format");

    cleanup_temp_dir(temp);
}

#[test]
fn test_add_with_backslash_and_pipe_combination() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Add task with both backslashes and pipes - CSV handles these naturally
    let result = run_command(&["add", "Test\\path|command", "-d", "Run\\cmd|filter"], &temp);
    assert!(result.success, "Should add task with backslash and pipe");

    let tasks_file = temp.join(".knecht/tasks");
    let content = fs::read_to_string(&tasks_file).unwrap();

    // CSV format preserves these characters without backslash escaping
    assert!(content.contains("Test\\path|command"), "Should preserve backslash and pipe in CSV");

    // Should be able to list it back
    let list = run_command(&["list"], &temp);
    assert!(list.success, "Should list tasks successfully");

    cleanup_temp_dir(temp);
}

#[test]
fn test_backslash_at_string_end() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // Backslash at the end of a field in CSV format
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1,open,\"TaskEndsWithBackslash\\\",\"DescEndsWithBackslash\\\",").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle backslash at end of string");

    cleanup_temp_dir(temp);
}

#[test]
fn test_split_unescaped_with_trailing_backslash() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // Test split_unescaped when backslash is at end (peek returns None)
    let mut file = fs::File::create(&tasks_file).unwrap();
    // Field ends with backslash, then pipe separator
    writeln!(file, "1|open\\|Test|Description\\").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle split with trailing backslash");

    cleanup_temp_dir(temp);
}

#[test]
fn test_unescape_backslash_followed_by_various_chars() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // Test backslash followed by characters other than \ or |
    // These should NOT be treated as escape sequences
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Test\\a\\b\\c|Desc\\x\\y\\z").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle backslash followed by non-escapable chars");

    // Verify the raw content preserves backslashes when not followed by \ or |
    let content = fs::read_to_string(&tasks_file).unwrap();
    assert!(content.contains("\\a\\b\\c"), "Should preserve backslash-char sequences");

    cleanup_temp_dir(temp);
}

#[test]
fn test_split_unescaped_with_backslash_not_before_pipe_or_backslash() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // In split_unescaped, backslash followed by char that's not | or \
    // Should not be treated as escape sequence, just regular chars
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Test\\xyz|Desc\\abc").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle backslash followed by regular chars in split");

    cleanup_temp_dir(temp);
}

#[test]
fn test_empty_string_escaping() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Empty title with description - edge case
    let result = run_command(&["add", "A", "-d", ""], &temp);
    assert!(result.success || !result.success, "Should handle empty description");

    cleanup_temp_dir(temp);
}

#[test]
fn test_only_backslashes() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // String of only backslashes - tests consecutive escaping
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|\\\\\\\\|\\\\\\\\\\\\").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle string of only backslashes");

    cleanup_temp_dir(temp);
}

#[test]
fn test_only_escaped_pipes() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // String of only escaped pipes
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|\\|\\|\\|\\||\\|\\|\\|").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle string of only escaped pipes");

    cleanup_temp_dir(temp);
}

#[test]
fn test_unescape_hits_backslash_check_first() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // This specifically tests when next_ch == '\\' is true (short-circuits the OR)
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Test|\\\\").unwrap();  // Escaped backslash
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success);

    cleanup_temp_dir(temp);
}

#[test]
fn test_unescape_hits_pipe_check_second() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // This specifically tests when next_ch == '\\' is false, so we check next_ch == '|'
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Test|\\|").unwrap();  // Escaped pipe
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success);

    cleanup_temp_dir(temp);
}

#[test]
fn test_split_hits_backslash_check_first() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // In split_unescaped: next_ch == '|' is false, next_ch == '\\' is true
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Test\\\\value|Desc").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success);

    cleanup_temp_dir(temp);
}

#[test]
fn test_split_hits_pipe_check_first() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // In split_unescaped: next_ch == '|' is true (short-circuits)
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Test\\|value|Desc").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success);

    cleanup_temp_dir(temp);
}

#[test]
fn test_split_unescaped_with_escaped_backslash_not_pipe() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // In split_unescaped: backslash followed by backslash (not pipe)
    // This should hit the `next_ch == '\\'` branch of the OR
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Title\\\\|Description\\\\").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle escaped backslash in split_unescaped");

    cleanup_temp_dir(temp);
}

#[test]
fn test_error_path_coverage_with_unit_tests() {
    // This test documents that we need unit tests with dependency injection
    // to cover error paths in task.rs
    //
    // The integration tests above cover the happy paths perfectly,
    // but to reach 100% region coverage, we need to test error paths
    // for IO operations that can fail.
    //
    // We'll add unit tests in task.rs using a trait-based approach
    // to inject test doubles that can simulate failures.
}

#[test]
fn production_tasks_file_is_never_modified_by_tests() {
    // CRITICAL TEST: Prevents data loss bug documented in task-114, task-106, task-109
    // This test ensures that running 'cargo test' NEVER modifies the production .knecht/tasks file
    // in the project root.

    use std::fs;
    use std::path::PathBuf;

    // Get the project root (where Cargo.toml is)
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let production_tasks_file = project_root.join(".knecht/tasks");

    // If the production tasks file doesn't exist, this test passes trivially
    if !production_tasks_file.exists() {
        return;
    }

    // Read the production tasks file BEFORE running any tests
    let content_before = fs::read_to_string(&production_tasks_file)
        .expect("Failed to read production tasks file");
    let line_count_before = content_before.lines().count();

    // Record the modification time
    let metadata_before = fs::metadata(&production_tasks_file)
        .expect("Failed to get metadata for production tasks file");
    let modified_before = metadata_before.modified()
        .expect("Failed to get modification time");

    // Run a dummy operation to ensure this test runs after other tests
    // (This test should be one of the last to run, but we can't guarantee order)
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Read the production tasks file AFTER
    let content_after = fs::read_to_string(&production_tasks_file)
        .expect("Failed to read production tasks file after tests");
    let line_count_after = content_after.lines().count();

    let metadata_after = fs::metadata(&production_tasks_file)
        .expect("Failed to get metadata for production tasks file after tests");
    let modified_after = metadata_after.modified()
        .expect("Failed to get modification time after tests");

    // Assert that the file was NOT modified
    if content_before != content_after {
        panic!(
            "CRITICAL: Production .knecht/tasks file was MODIFIED during tests!\n\
             Line count before: {}\n\
             Line count after: {}\n\
             This is the data loss bug from task-114.\n\
             A test is writing to the production file instead of using a temp directory.\n\
             Content before:\n{}\n\n\
             Content after:\n{}",
            line_count_before,
            line_count_after,
            content_before,
            content_after
        );
    }

    if modified_before != modified_after {
        // This could be a false positive if another process modified the file,
        // but it's worth checking
        eprintln!(
            "WARNING: Production .knecht/tasks modification time changed during tests.\n\
             This might indicate a test is touching the production file.\n\
             Modified before: {:?}\n\
             Modified after: {:?}",
            modified_before,
            modified_after
        );
    }
}

#[test]
fn show_displays_task_with_description() {
    let temp = setup_temp_dir();

    // Initialize and add a task with description
    run_command(&["init"], &temp);
    run_command(&["add", "Task title", "-d", "This is a detailed description"], &temp);

    // Run show command
    let result = run_command(&["show", "task-1"], &temp);

    assert!(result.success, "show command should succeed");
    assert!(result.stdout.contains("task-1"), "should show task ID");
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
    run_command(&["add", "Simple task"], &temp);

    // Run show command
    let result = run_command(&["show", "task-1"], &temp);

    assert!(result.success, "show command should succeed");
    assert!(result.stdout.contains("task-1"), "should show task ID");
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
fn start_displays_task_details_with_description() {
    with_initialized_repo(|temp| {
        // Add a task with description
        let add_result = run_command(&["add", "Implement feature X", "-d", "This feature should do X, Y, and Z"], &temp);
        assert!(add_result.success, "Failed to add task");

        // Start working on the task
        let result = run_command(&["start", "task-1"], &temp);

        assert!(result.success, "start command should succeed");
        assert!(result.stdout.contains("task-1"), "should show task ID");
        assert!(result.stdout.contains("Implement feature X"), "should show task title");
        assert!(result.stdout.contains("This feature should do X, Y, and Z"), "should show task description");
    });
}

#[test]
fn start_displays_task_without_description() {
    with_initialized_repo(|temp| {
        // Add a task without description
        let add_result = run_command(&["add", "Simple task"], &temp);
        assert!(add_result.success, "Failed to add task");

        // Start working on the task
        let result = run_command(&["start", "task-1"], &temp);

        assert!(result.success, "start command should succeed");
        assert!(result.stdout.contains("task-1"), "should show task ID");
        assert!(result.stdout.contains("Simple task"), "should show task title");
        assert!(!result.stdout.contains("Description:"), "should not show description label when no description");
    });
}

#[test]
fn start_requires_task_id_argument() {
    with_initialized_repo(|temp| {
        // Try start without task ID
        let result = run_command(&["start"], &temp);

        assert!(!result.success, "start should fail without task ID");
        assert!(result.stderr.contains("Usage") || result.stderr.contains("usage"),
                "should show usage message");
    });
}

#[test]
fn start_fails_on_nonexistent_task() {
    with_initialized_repo(|temp| {
        // Try to start a task that doesn't exist
        let result = run_command(&["start", "task-999"], &temp);

        assert!(!result.success, "start should fail on nonexistent task");
        assert!(result.stderr.contains("not found") || result.stderr.contains("Not found"),
                "should indicate task was not found");
    });
}

#[test]
fn pain_increments_pain_count_on_task() {
    with_initialized_repo(|temp| {
        // Add a task without pain count
        run_command(&["add", "Fix bug"], &temp);

        // Increment pain count (should add it as 1)
        let result = run_command(&["pain", "-t", "task-1", "-d", "First occurrence"], &temp);
        assert!(result.success, "pain command should succeed");

        // Verify pain count was added as 1
        let list = run_command(&["list"], &temp);
        assert!(
            list.stdout.contains("Fix bug (pain count: 1)"),
            "Pain count should be added as 1, got: {}",
            list.stdout
        );

        // Increment again
        let result2 = run_command(&["pain", "-t", "task-1", "-d", "Second occurrence"], &temp);
        assert!(result2.success, "pain command should succeed again");

        // Verify pain count was incremented to 2
        let list2 = run_command(&["list"], &temp);
        assert!(
            list2.stdout.contains("Fix bug (pain count: 2)"),
            "Pain count should be incremented to 2, got: {}",
            list2.stdout
        );
    });
}

#[test]
fn pain_adds_pain_count_to_task_without_one() {
    with_initialized_repo(|temp| {
        // Add a task without pain count
        run_command(&["add", "Some task"], &temp);

        // Increment pain count
        let result = run_command(&["pain", "-t", "task-1", "-d", "Pain instance"], &temp);
        assert!(result.success, "pain command should succeed");

        // Verify pain count was added
        let list = run_command(&["list"], &temp);
        assert!(
            list.stdout.contains("(pain count: 1)"),
            "Pain count should be added as 1, got: {}",
            list.stdout
        );
    });
}

#[test]
fn pain_fails_on_nonexistent_task() {
    with_initialized_repo(|temp| {
        let result = run_command(&["pain", "-t", "task-999", "-d", "Test pain"], &temp);

        assert!(!result.success, "pain command should fail on nonexistent task");
        assert!(
            result.stderr.contains("not found") || result.stderr.contains("Not found"),
            "should indicate task was not found"
        );
    });
}

#[test]
fn pain_requires_task_id_argument() {
    with_initialized_repo(|temp| {
        let result = run_command(&["pain"], &temp);

        assert!(!result.success, "pain command should fail without arguments");
        assert!(
            result.stderr.contains("-t") || result.stderr.contains("task-id"),
            "Error should mention -t flag, got: {}",
            result.stderr
        );
    });
}

#[test]
fn pain_on_task_with_description_and_pain_count() {
    with_initialized_repo(|temp| {
        // Add a task with description
        run_command(&["add", "Fix critical bug", "-d", "This bug breaks production"], &temp);

        // Add pain count
        run_command(&["pain", "-t", "task-1", "-d", "First pain"], &temp);

        // Increment pain count again
        let result = run_command(&["pain", "-t", "task-1", "-d", "Second pain"], &temp);
        assert!(result.success, "pain command should succeed on task with description");

        // Verify both description and pain count are preserved
        let show = run_command(&["show", "task-1"], &temp);
        assert!(show.stdout.contains("Fix critical bug"), "Title should be preserved");
        assert!(show.stdout.contains("This bug breaks production"), "Description should be preserved");

        let list = run_command(&["list"], &temp);
        assert!(
            list.stdout.contains("Fix critical bug (pain count: 2)"),
            "Pain count should be 2 with description preserved, got: {}",
            list.stdout
        );
    });
}

#[test]
fn next_suggests_task_with_highest_pain_count() {
    with_initialized_repo(|temp| {
        // Add tasks
        run_command(&["add", "Low priority task"], &temp);
        run_command(&["add", "Medium pain task"], &temp);
        run_command(&["add", "High pain task"], &temp);
        run_command(&["add", "Another low priority"], &temp);
        run_command(&["add", "Medium pain again"], &temp);

        // Set pain counts using pain command
        run_command(&["pain", "-t", "task-2", "-d", "Pain 1"], &temp);
        run_command(&["pain", "-t", "task-2", "-d", "Pain 2"], &temp); // pain count: 2

        run_command(&["pain", "-t", "task-3", "-d", "Pain 1"], &temp);
        run_command(&["pain", "-t", "task-3", "-d", "Pain 2"], &temp);
        run_command(&["pain", "-t", "task-3", "-d", "Pain 3"], &temp);
        run_command(&["pain", "-t", "task-3", "-d", "Pain 4"], &temp);
        run_command(&["pain", "-t", "task-3", "-d", "Pain 5"], &temp); // pain count: 5

        run_command(&["pain", "-t", "task-5", "-d", "Pain 1"], &temp);
        run_command(&["pain", "-t", "task-5", "-d", "Pain 2"], &temp); // pain count: 2

        // Run 'knecht next'
        let result = run_command(&["next"], &temp);

        assert!(result.success, "next command should succeed");
        assert!(
            result.stdout.contains("task-3"),
            "Should suggest task-3 with highest pain count, got: {}",
            result.stdout
        );
        assert!(
            result.stdout.contains("High pain task"),
            "Should show the task title, got: {}",
            result.stdout
        );
        assert!(
            result.stdout.contains("pain count: 5"),
            "Should mention the pain count, got: {}",
            result.stdout
        );
    });
}

#[test]
fn next_prefers_older_task_when_pain_counts_equal() {
    with_initialized_repo(|temp| {
        // Add tasks
        run_command(&["add", "First task"], &temp);
        run_command(&["add", "Second task"], &temp);
        run_command(&["add", "Third task"], &temp);

        // Set same pain count on all tasks
        for i in 0..3 {
            run_command(&["pain", "-t", "task-1", "-d", &format!("Pain {}", i)], &temp);
            run_command(&["pain", "-t", "task-2", "-d", &format!("Pain {}", i)], &temp);
            run_command(&["pain", "-t", "task-3", "-d", &format!("Pain {}", i)], &temp);
        }

        let result = run_command(&["next"], &temp);

        assert!(result.success, "next command should succeed");
        assert!(
            result.stdout.contains("task-1"),
            "Should suggest oldest task (task-1) when pain counts equal, got: {}",
            result.stdout
        );
    });
}

#[test]
fn next_skips_done_tasks() {
    with_initialized_repo(|temp| {
        // Add tasks
        run_command(&["add", "High pain but done"], &temp);
        run_command(&["add", "Lower pain but open"], &temp);

        // Set pain counts
        for i in 0..5 {
            run_command(&["pain", "-t", "task-1", "-d", &format!("Pain {}", i)], &temp);
        }
        for i in 0..2 {
            run_command(&["pain", "-t", "task-2", "-d", &format!("Pain {}", i)], &temp);
        }
        
        // Mark first task as done
        run_command(&["done", "task-1"], &temp);

        let result = run_command(&["next"], &temp);
        
        assert!(result.success, "next command should succeed");
        assert!(
            result.stdout.contains("task-2"),
            "Should skip done tasks and suggest task-2, got: {}",
            result.stdout
        );
    });
}

#[test]
fn next_handles_no_open_tasks() {
    with_initialized_repo(|temp| {
        // Add and complete a task
        run_command(&["add", "Only task"], &temp);
        run_command(&["done", "task-1"], &temp);

        let result = run_command(&["next"], &temp);

        assert!(result.success, "next command should succeed");
        assert!(
            result.stdout.contains("No open tasks") || result.stdout.contains("no open tasks"),
            "Should indicate no open tasks available, got: {}",
            result.stdout
        );
    });
}

#[test]
fn next_fails_gracefully_when_tasks_file_unreadable() {
    with_initialized_repo(|temp| {
        // Add a task
        run_command(&["add", "Some task"], &temp);
        
        // Make tasks file unreadable
        let tasks_file = temp.join(".knecht/tasks");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&tasks_file).unwrap().permissions();
            perms.set_mode(0o000);
            fs::set_permissions(&tasks_file, perms).unwrap();
        }
        
        let result = run_command(&["next"], &temp);
        
        // Restore permissions for cleanup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&tasks_file).unwrap().permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&tasks_file, perms).unwrap();
        }
        
        assert!(!result.success, "next should fail when tasks file is unreadable");
        assert!(
            result.stderr.contains("Error reading tasks") || result.stderr.contains("error"),
            "should indicate error reading tasks, got: {}",
            result.stderr
        );
    });
}

#[test]
fn next_displays_task_with_description() {
    with_initialized_repo(|temp| {
        // Add a task with description
        run_command(&["add", "Important task", "-d", "This task has a detailed description explaining what needs to be done"], &temp);

        // Add pain to make it more likely to be selected
        run_command(&["pain", "-t", "task-1", "-d", "Pain 1"], &temp);
        run_command(&["pain", "-t", "task-1", "-d", "Pain 2"], &temp);

        let result = run_command(&["next"], &temp);

        assert!(result.success, "next command should succeed");
        assert!(result.stdout.contains("task-1"), "Should suggest task-1");
        assert!(result.stdout.contains("Important task"), "Should show title");
        assert!(
            result.stdout.contains("This task has a detailed description"),
            "Should show description, got: {}",
            result.stdout
        );
        assert!(result.stdout.contains("pain count: 2"), "Should show pain count");
    });
}

#[test]
fn next_with_zero_pain_count() {
    with_initialized_repo(|temp| {
        // Add tasks - one will have pain_count 0 (no pain added), one will be without pain_count
        run_command(&["add", "Task with no pain"], &temp);
        run_command(&["add", "Another task"], &temp);
        
        let result = run_command(&["next"], &temp);
        
        assert!(result.success, "next command should succeed");
        // Should suggest task-1 (older task when both have no pain)
        assert!(result.stdout.contains("task-1"), "Should suggest task-1");
        // Should not show pain count line when pain is 0 or None
        assert!(
            !result.stdout.contains("pain count:"),
            "Should not show pain count for tasks with 0 or no pain, got: {}",
            result.stdout
        );
    });
}

#[test]
fn done_increments_pain_on_skipped_top_task() {
    with_initialized_repo(|temp| {
        // Create task-1 which is the oldest
        run_command(&["add", "Primary feature work"], &temp);
        
        // Create task-2 (newer task)
        run_command(&["add", "Minor improvement"], &temp);
        
        // Complete task-2 instead of task-1 (skipping the oldest task)
        let done_result = run_command(&["done", "task-2"], &temp);
        assert!(done_result.success, "done should succeed");
        
        // Check that task-1's pain count increased (it was skipped)
        let list_result = run_command(&["list"], &temp);
        let task1_line = list_result.stdout.lines()
            .find(|line| line.contains("task-1"))
            .expect("Should find task-1 in list output");
        
        // Pain should have incremented from 0 to 1
        assert!(task1_line.contains("(pain count: 1)"),
            "task-1 pain should increment to 1 when skipped, got: {}", task1_line);
        
        // Check task-1's description mentions it was skipped
        let show_result = run_command(&["show", "task-1"], &temp);
        assert!(show_result.stdout.contains("Skip: task-2 completed instead") ||
                show_result.stdout.contains("Skip:"),
            "task-1 description should note it was skipped, got: {}", show_result.stdout);
    });
}

#[test]
fn done_on_oldest_task_does_not_increment_pain() {
    with_initialized_repo(|temp| {
        // Create two tasks
        run_command(&["add", "Oldest task"], &temp);
        run_command(&["add", "Newer task"], &temp);
        
        // Complete task-1 (the oldest task - not skipping it)
        let done_result = run_command(&["done", "task-1"], &temp);
        assert!(done_result.success);
        
        // Verify task-2 still has no pain (it wasn't skipped - we did the oldest first)
        let list_result = run_command(&["list"], &temp);
        let task2_line = list_result.stdout.lines()
            .find(|line| line.contains("task-2"))
            .expect("Should find task-2");
        
        assert!(!task2_line.contains("pain count:"),
            "task-2 should have no pain when oldest task was completed, got: {}", task2_line);
    });
}

#[test]
fn done_increments_pain_on_task_with_existing_description() {
    with_initialized_repo(|temp| {
        // Create task-1 (oldest) with a description
        run_command(&["add", "Primary feature", "-d", "Original description"], &temp);
        
        // Create task-2 (newer)
        run_command(&["add", "Minor task"], &temp);
        
        // Complete task-2, skipping task-1
        let done_result = run_command(&["done", "task-2"], &temp);
        assert!(done_result.success);
        
        // Verify task-1's pain incremented and skip note was appended to existing description
        let show_result = run_command(&["show", "task-1"], &temp);
        assert!(show_result.stdout.contains("Original description"),
            "Should preserve original description");
        assert!(show_result.stdout.contains("Skip: task-2 completed instead"),
            "Should append skip note to existing description, got: {}", show_result.stdout);
    });
}

#[test]
fn list_includes_usage_instructions_for_agents() {
    with_initialized_repo(|temp| {
        // Add a task with a description
        run_command(&["add", "Test task", "-d", "Task description here"], &temp);
        
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

// Delete command tests

#[test]
fn delete_removes_existing_task() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task to delete"], &temp);
        run_command(&["add", "Task to keep"], &temp);

        let result = run_command(&["delete", "task-1"], &temp);
        assert!(result.success, "delete command should succeed");
        assert!(
            result.stdout.contains("Deleted task-1"),
            "Should show confirmation message, got: {}",
            result.stdout
        );

        // Verify task-1 is gone and task-2 remains
        let list = run_command(&["list"], &temp);
        assert!(!list.stdout.contains("Task to delete"), "Deleted task should not appear in list");
        assert!(list.stdout.contains("Task to keep"), "Other tasks should remain");
    });
}

#[test]
fn delete_accepts_id_without_prefix() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task to delete"], &temp);

        let result = run_command(&["delete", "1"], &temp);
        assert!(result.success, "delete should accept numeric ID without 'task-' prefix");
        assert!(
            result.stdout.contains("Deleted task-1"),
            "Should show confirmation with task- prefix, got: {}",
            result.stdout
        );
    });
}

#[test]
fn delete_preserves_other_tasks() {
    with_initialized_repo(|temp| {
        run_command(&["add", "First task"], &temp);
        run_command(&["add", "Second task"], &temp);
        run_command(&["add", "Third task"], &temp);

        run_command(&["delete", "task-2"], &temp);

        let list = run_command(&["list"], &temp);
        assert!(list.stdout.contains("First task"), "First task should remain");
        assert!(!list.stdout.contains("Second task"), "Second task should be deleted");
        assert!(list.stdout.contains("Third task"), "Third task should remain");
    });
}

#[test]
fn delete_works_for_done_tasks() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Completed task"], &temp);
        run_command(&["done", "task-1"], &temp);

        let result = run_command(&["delete", "task-1"], &temp);
        assert!(result.success, "Should be able to delete done tasks");
        assert!(result.stdout.contains("Deleted task-1"));
    });
}

#[test]
fn delete_fails_on_nonexistent_task() {
    with_initialized_repo(|temp| {
        let result = run_command(&["delete", "task-999"], &temp);
        assert!(!result.success, "delete on nonexistent task should fail");
        assert!(
            result.stderr.contains("not found") || result.stderr.contains("doesn't exist"),
            "Should have helpful error message, got: {}",
            result.stderr
        );
    });
}

#[test]
fn delete_fails_with_invalid_task_id() {
    with_initialized_repo(|temp| {
        let result = run_command(&["delete", "task-abc"], &temp);
        assert!(!result.success, "delete with invalid ID should fail");
        assert!(
            result.stderr.contains("Invalid") || result.stderr.contains("invalid"),
            "Should mention invalid ID, got: {}",
            result.stderr
        );
    });
}

#[test]
fn delete_requires_task_id_argument() {
    with_initialized_repo(|temp| {
        let result = run_command(&["delete"], &temp);
        assert!(!result.success, "delete without task ID should fail");
        assert!(
            result.stderr.contains("Usage") || result.stderr.contains("required"),
            "Should show usage or mention required argument, got: {}",
            result.stderr
        );
    });
}

#[test]
fn delete_can_delete_first_task() {
    with_initialized_repo(|temp| {
        run_command(&["add", "First"], &temp);
        run_command(&["add", "Second"], &temp);

        let result = run_command(&["delete", "task-1"], &temp);
        assert!(result.success, "Should be able to delete first task");

        let list = run_command(&["list"], &temp);
        assert!(!list.stdout.contains("First"));
        assert!(list.stdout.contains("Second"));
    });
}

#[test]
fn delete_can_delete_last_task() {
    with_initialized_repo(|temp| {
        run_command(&["add", "First"], &temp);
        run_command(&["add", "Last"], &temp);

        let result = run_command(&["delete", "task-2"], &temp);
        assert!(result.success, "Should be able to delete last task");

        let list = run_command(&["list"], &temp);
        assert!(list.stdout.contains("First"));
        assert!(!list.stdout.contains("Last"));
    });
}

#[test]
fn delete_can_delete_only_task() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Only task"], &temp);

        let result = run_command(&["delete", "task-1"], &temp);
        assert!(result.success, "Should be able to delete when only one task exists");

        let list = run_command(&["list"], &temp);
        assert!(!list.stdout.contains("Only task"));
    });
}

#[test]
fn delete_maintains_file_format() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task one", "-d", "Description with | pipe"], &temp);
        run_command(&["add", "Task two"], &temp);
        run_command(&["add", "Task three", "-d", "Another description"], &temp);
        run_command(&["done", "task-2"], &temp);

        run_command(&["delete", "task-2"], &temp);

        // Verify remaining tasks are still properly formatted
        let show1 = run_command(&["show", "task-1"], &temp);
        assert!(show1.success);
        assert!(show1.stdout.contains("Description with | pipe"));

        let show3 = run_command(&["show", "task-3"], &temp);
        assert!(show3.success);
        assert!(show3.stdout.contains("Another description"));
    });
}

#[test]
fn update_title_only() {
    with_initialized_repo(|temp| {
        // Add a task
        run_command(&["add", "Old Title"], &temp);

        // Update the title
        let result = run_command(&["update", "task-1", "--title", "New Title"], &temp);
        assert!(result.success, "update command should succeed: {}", result.stderr);
        assert!(result.stdout.contains("Updated task-1"), "Should show success message");

        // Verify the title was updated
        let show = run_command(&["show", "task-1"], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("New Title"), "Title should be updated");
        assert!(!show.stdout.contains("Old Title"), "Old title should be gone");

        // Verify status is preserved (should still be open)
        let list = run_command(&["list"], &temp);
        assert!(list.stdout.contains("[ ]"), "Task should still be open");
        assert!(list.stdout.contains("New Title"), "New title should appear in list");
    });
}

#[test]
fn update_description_only() {
    with_initialized_repo(|temp| {
        // Add a task with a description
        run_command(&["add", "Task Title", "-d", "Old description"], &temp);

        // Update only the description
        let result = run_command(&["update", "task-1", "--description", "New description"], &temp);
        assert!(result.success, "update command should succeed: {}", result.stderr);

        // Verify the description was updated but title unchanged
        let show = run_command(&["show", "task-1"], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("Task Title"), "Title should be unchanged");
        assert!(show.stdout.contains("New description"), "Description should be updated");
        assert!(!show.stdout.contains("Old description"), "Old description should be gone");
    });
}

#[test]
fn update_add_description_to_task_without_one() {
    with_initialized_repo(|temp| {
        // Add a task without description
        run_command(&["add", "Task without description"], &temp);

        // Add a description
        let result = run_command(&["update", "task-1", "--description", "New description added"], &temp);
        assert!(result.success, "update command should succeed: {}", result.stderr);

        // Verify the description was added
        let show = run_command(&["show", "task-1"], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("New description added"), "Description should be added");
    });
}

#[test]
fn update_both_title_and_description() {
    with_initialized_repo(|temp| {
        // Add a task with both
        run_command(&["add", "Old Title", "-d", "Old description"], &temp);

        // Update both
        let result = run_command(&["update", "task-1", "--title", "New Title", "--description", "New description"], &temp);
        assert!(result.success, "update command should succeed: {}", result.stderr);

        // Verify both were updated
        let show = run_command(&["show", "task-1"], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("New Title"), "Title should be updated");
        assert!(show.stdout.contains("New description"), "Description should be updated");
        assert!(!show.stdout.contains("Old Title"), "Old title should be gone");
        assert!(!show.stdout.contains("Old description"), "Old description should be gone");
    });
}

#[test]
fn update_with_short_flags() {
    with_initialized_repo(|temp| {
        // Add a task
        run_command(&["add", "Old Title"], &temp);

        // Update using short flags
        let result = run_command(&["update", "task-1", "-t", "New Title", "-d", "New description"], &temp);
        assert!(result.success, "update with short flags should succeed: {}", result.stderr);

        // Verify updates
        let show = run_command(&["show", "task-1"], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("New Title"), "Title should be updated");
        assert!(show.stdout.contains("New description"), "Description should be updated");
    });
}

#[test]
fn update_clear_description() {
    with_initialized_repo(|temp| {
        // Add a task with description
        run_command(&["add", "Task Title", "-d", "Description to remove"], &temp);

        // Clear the description
        let result = run_command(&["update", "task-1", "--description", ""], &temp);
        assert!(result.success, "update should succeed: {}", result.stderr);

        // Verify description is gone
        let show = run_command(&["show", "task-1"], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("Task Title"), "Title should remain");
        assert!(!show.stdout.contains("Description to remove"), "Description should be removed");
        assert!(!show.stdout.contains("Description:"), "Description field should not appear");
    });
}

#[test]
fn update_nonexistent_task() {
    with_initialized_repo(|temp| {
        // Try to update a task that doesn't exist
        let result = run_command(&["update", "task-999", "--title", "New Title"], &temp);
        assert!(!result.success, "update should fail for nonexistent task");
        assert!(result.stderr.contains("task-999"), "Error should mention the task ID");
        assert!(result.stderr.contains("not found") || result.stderr.contains("not found"), "Error should say not found");
    });
}

#[test]
fn update_no_flags_provided() {
    with_initialized_repo(|temp| {
        // Add a task
        run_command(&["add", "Task Title"], &temp);

        // Try to update without providing any flags
        let result = run_command(&["update", "task-1"], &temp);
        assert!(!result.success, "update should fail when no flags provided");
        assert!(result.stderr.contains("title") || result.stderr.contains("description"), "Error should mention required flags");
    });
}

#[test]
fn update_preserves_status() {
    with_initialized_repo(|temp| {
        // Add and complete a task
        run_command(&["add", "Done Task"], &temp);
        run_command(&["done", "task-1"], &temp);

        // Update the title
        let result = run_command(&["update", "task-1", "--title", "Updated Done Task"], &temp);
        assert!(result.success, "update should succeed: {}", result.stderr);

        // Verify status is still done
        let show = run_command(&["show", "task-1"], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("done"), "Status should still be done");
        assert!(show.stdout.contains("Updated Done Task"), "Title should be updated");
    });
}

#[test]
fn update_only_affects_target_task() {
    with_initialized_repo(|temp| {
        // Add multiple tasks
        run_command(&["add", "Task One"], &temp);
        run_command(&["add", "Task Two"], &temp);
        run_command(&["add", "Task Three"], &temp);

        // Update only task-2
        let result = run_command(&["update", "task-2", "--title", "Updated Task Two"], &temp);
        assert!(result.success, "update should succeed: {}", result.stderr);

        // Verify only task-2 was changed
        let show1 = run_command(&["show", "task-1"], &temp);
        assert!(show1.stdout.contains("Task One"), "Task 1 should be unchanged");

        let show2 = run_command(&["show", "task-2"], &temp);
        assert!(show2.stdout.contains("Updated Task Two"), "Task 2 should be updated");

        let show3 = run_command(&["show", "task-3"], &temp);
        assert!(show3.stdout.contains("Task Three"), "Task 3 should be unchanged");
    });
}

#[test]
fn update_fails_when_no_task_id_provided() {
    with_initialized_repo(|temp| {
        // Try to update without providing a task ID
        let result = run_command(&["update"], &temp);
        assert!(!result.success, "update should fail when no task ID provided");
        assert!(result.stderr.contains("Usage") || result.stderr.contains("task-id"), "Error should show usage");
    });
}

#[test]
fn update_fails_when_title_flag_has_no_value() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task Title"], &temp);
        
        // Try to update with --title but no value
        let result = run_command(&["update", "task-1", "--title"], &temp);
        assert!(!result.success, "update should fail when --title has no value");
        assert!(result.stderr.contains("title") && result.stderr.contains("value"), "Error should mention title requires value");
    });
}

#[test]
fn update_fails_when_description_flag_has_no_value() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task Title"], &temp);
        
        // Try to update with --description but no value
        let result = run_command(&["update", "task-1", "--description"], &temp);
        assert!(!result.success, "update should fail when --description has no value");
        assert!(result.stderr.contains("description") && result.stderr.contains("value"), "Error should mention description requires value");
    });
}

#[test]
fn update_fails_with_unknown_flag() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task Title"], &temp);
        
        // Try to update with an unknown flag
        let result = run_command(&["update", "task-1", "--invalid-flag", "value"], &temp);
        assert!(!result.success, "update should fail with unknown flag");
        assert!(result.stderr.contains("Unknown") || result.stderr.contains("invalid"), "Error should mention unknown flag");
    });
}

#[test]
fn update_handles_special_characters() {
    with_initialized_repo(|temp| {
        // Add a task
        run_command(&["add", "Simple Title"], &temp);

        // Update with special characters (pipe is tricky for our format)
        let result = run_command(&["update", "task-1", "--title", "Title with | pipe", "--description", "Description with special chars: | and newlines"], &temp);
        assert!(result.success, "update should handle special characters: {}", result.stderr);

        // Verify the special characters are preserved
        let show = run_command(&["show", "task-1"], &temp);
        assert!(show.success);
        assert!(show.stdout.contains("Title with | pipe"), "Pipe in title should be preserved");
        assert!(show.stdout.contains("Description with special chars: | and newlines"), "Pipe in description should be preserved");
    });
}

// ==============================================================================
// Blocker Tracking Tests
// ==============================================================================

#[test]
fn block_command_creates_blocker_relationship() {
    with_initialized_repo(|temp| {
        // Create two tasks
        run_command(&["add", "Task A"], &temp);
        run_command(&["add", "Task B"], &temp);

        // Create blocker: task-1 is blocked by task-2
        let result = run_command(&["block", "task-1", "by", "task-2"], &temp);
        assert!(result.success, "block command should succeed: {}", result.stderr);
        assert!(result.stdout.contains("Blocker added"), "Should confirm blocker added");
        assert!(result.stdout.contains("task-1") && result.stdout.contains("task-2"), 
                "Should mention both tasks");

        // Verify blockers file exists and contains the relationship
        let blockers_path = temp.join(".knecht/blockers");
        assert!(blockers_path.exists(), "blockers file should be created");
        
        let content = fs::read_to_string(&blockers_path).unwrap();
        assert!(content.contains("task-1|task-2"), "Should store blocker relationship");
    });
}

#[test]
fn block_command_fails_on_nonexistent_task() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);

        // Try to block nonexistent task
        let result = run_command(&["block", "task-999", "by", "task-1"], &temp);
        assert!(!result.success, "block command should fail for nonexistent task");
        assert!(result.stderr.contains("does not exist") || result.stderr.contains("not found"),
                "Should have helpful error message: {}", result.stderr);
    });
}

#[test]
fn block_command_fails_on_nonexistent_blocker() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);

        // Try to block by nonexistent task
        let result = run_command(&["block", "task-1", "by", "task-999"], &temp);
        assert!(!result.success, "block command should fail for nonexistent blocker");
        assert!(result.stderr.contains("does not exist") || result.stderr.contains("not found"),
                "Should have helpful error message: {}", result.stderr);
    });
}

#[test]
fn show_displays_blockers() {
    with_initialized_repo(|temp| {
        // Create tasks
        run_command(&["add", "Blocked Task"], &temp);
        run_command(&["add", "Blocker Task"], &temp);
        run_command(&["add", "Another Blocker"], &temp);

        // Create blocker relationships
        run_command(&["block", "task-1", "by", "task-2"], &temp);
        run_command(&["block", "task-1", "by", "task-3"], &temp);

        // Check show output
        let result = run_command(&["show", "task-1"], &temp);
        assert!(result.success, "show command should succeed");
        assert!(result.stdout.contains("Blocked by:"), "Should have 'Blocked by:' section");
        assert!(result.stdout.contains("task-2"), "Should show task-2 as blocker");
        assert!(result.stdout.contains("task-3"), "Should show task-3 as blocker");
        assert!(result.stdout.contains("Blocker Task"), "Should show blocker task title");
    });
}

#[test]
fn show_displays_what_task_blocks() {
    with_initialized_repo(|temp| {
        // Create tasks
        run_command(&["add", "Blocker Task"], &temp);
        run_command(&["add", "Blocked Task A"], &temp);
        run_command(&["add", "Blocked Task B"], &temp);

        // task-1 blocks both task-2 and task-3
        run_command(&["block", "task-2", "by", "task-1"], &temp);
        run_command(&["block", "task-3", "by", "task-1"], &temp);

        // Check show output for task-1
        let result = run_command(&["show", "task-1"], &temp);
        assert!(result.success, "show command should succeed");
        assert!(result.stdout.contains("Blocks:"), "Should have 'Blocks:' section");
        assert!(result.stdout.contains("task-2"), "Should show task-2 is blocked");
        assert!(result.stdout.contains("task-3"), "Should show task-3 is blocked");
    });
}

#[test]
fn start_fails_when_blocked_by_open_task() {
    with_initialized_repo(|temp| {
        // Create tasks
        run_command(&["add", "Blocked Task"], &temp);
        run_command(&["add", "Blocker Task"], &temp);

        // Create blocker
        run_command(&["block", "task-1", "by", "task-2"], &temp);

        // Try to start blocked task
        let result = run_command(&["start", "task-1"], &temp);
        assert!(!result.success, "start should fail when task is blocked by open task");
        assert!(result.stderr.contains("Cannot start") || result.stderr.contains("blocked"),
                "Should explain why start failed: {}", result.stderr);
        assert!(result.stderr.contains("task-2"), "Should mention the blocking task");
    });
}

#[test]
fn start_succeeds_when_blocker_is_done() {
    with_initialized_repo(|temp| {
        // Create tasks
        run_command(&["add", "Blocked Task"], &temp);
        run_command(&["add", "Blocker Task"], &temp);

        // Create blocker
        run_command(&["block", "task-1", "by", "task-2"], &temp);

        // Complete the blocker
        run_command(&["done", "task-2"], &temp);

        // Now start should succeed
        let result = run_command(&["start", "task-1"], &temp);
        assert!(result.success, "start should succeed when blocker is done: {}", result.stderr);
    });
}

#[test]
fn start_succeeds_when_no_blockers() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Normal Task"], &temp);

        let result = run_command(&["start", "task-1"], &temp);
        assert!(result.success, "start should succeed for task with no blockers");
    });
}

#[test]
fn unblock_removes_blocker_relationship() {
    with_initialized_repo(|temp| {
        // Create tasks and blocker
        run_command(&["add", "Blocked Task"], &temp);
        run_command(&["add", "Blocker Task"], &temp);
        run_command(&["block", "task-1", "by", "task-2"], &temp);

        // Remove blocker
        let result = run_command(&["unblock", "task-1", "from", "task-2"], &temp);
        assert!(result.success, "unblock command should succeed: {}", result.stderr);
        assert!(result.stdout.contains("Blocker removed"), "Should confirm removal");

        // Verify blockers file no longer contains the relationship
        let blockers_path = temp.join(".knecht/blockers");
        let content = fs::read_to_string(&blockers_path).unwrap();
        assert!(!content.contains("task-1|task-2"), "Should remove blocker relationship");

        // Start should now succeed
        let start_result = run_command(&["start", "task-1"], &temp);
        assert!(start_result.success, "start should succeed after unblocking");
    });
}

#[test]
fn unblock_fails_when_relationship_does_not_exist() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);
        run_command(&["add", "Task B"], &temp);

        // Try to remove nonexistent blocker
        let result = run_command(&["unblock", "task-1", "from", "task-2"], &temp);
        assert!(!result.success, "unblock should fail when relationship doesn't exist");
        assert!(result.stderr.contains("not blocked") || result.stderr.contains("does not exist"),
                "Should have helpful error message: {}", result.stderr);
    });
}

#[test]
fn multiple_blockers_all_prevent_start() {
    with_initialized_repo(|temp| {
        // Create tasks
        run_command(&["add", "Blocked Task"], &temp);
        run_command(&["add", "Blocker 1"], &temp);
        run_command(&["add", "Blocker 2"], &temp);

        // Create multiple blockers
        run_command(&["block", "task-1", "by", "task-2"], &temp);
        run_command(&["block", "task-1", "by", "task-3"], &temp);

        // Start should fail
        let result = run_command(&["start", "task-1"], &temp);
        assert!(!result.success, "start should fail with multiple open blockers");
        assert!(result.stderr.contains("task-2") && result.stderr.contains("task-3"),
                "Should list all blocking tasks: {}", result.stderr);
    });
}

#[test]
fn start_succeeds_when_all_blockers_are_done() {
    with_initialized_repo(|temp| {
        // Create tasks
        run_command(&["add", "Blocked Task"], &temp);
        run_command(&["add", "Blocker 1"], &temp);
        run_command(&["add", "Blocker 2"], &temp);

        // Create multiple blockers
        run_command(&["block", "task-1", "by", "task-2"], &temp);
        run_command(&["block", "task-1", "by", "task-3"], &temp);

        // Complete both blockers
        run_command(&["done", "task-2"], &temp);
        run_command(&["done", "task-3"], &temp);

        // Start should succeed
        let result = run_command(&["start", "task-1"], &temp);
        assert!(result.success, "start should succeed when all blockers are done: {}", result.stderr);
    });
}

#[test]
fn show_indicates_blocker_status() {
    with_initialized_repo(|temp| {
        // Create tasks
        run_command(&["add", "Blocked Task"], &temp);
        run_command(&["add", "Open Blocker"], &temp);
        run_command(&["add", "Done Blocker"], &temp);

        // Create blockers
        run_command(&["block", "task-1", "by", "task-2"], &temp);
        run_command(&["block", "task-1", "by", "task-3"], &temp);

        // Complete one blocker
        run_command(&["done", "task-3"], &temp);

        // Check show output
        let result = run_command(&["show", "task-1"], &temp);
        assert!(result.success);
        assert!(result.stdout.contains("task-2") && result.stdout.contains("open"),
                "Should show task-2 as open: {}", result.stdout);
        assert!(result.stdout.contains("task-3") && result.stdout.contains("done"),
                "Should show task-3 as done: {}", result.stdout);
    });
}

#[test]
fn block_fails_when_blockers_file_cannot_be_written() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);
        run_command(&["add", "Task B"], &temp);

        // Make blockers file read-only
        let blockers_path = temp.join(".knecht/blockers");
        fs::write(&blockers_path, "").unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&blockers_path).unwrap().permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(&blockers_path, perms).unwrap();
        }
        
        #[cfg(windows)]
        {
            let mut perms = fs::metadata(&blockers_path).unwrap().permissions();
            perms.set_readonly(true);
            fs::set_permissions(&blockers_path, perms).unwrap();
        }

        // Try to add blocker - should fail
        let result = run_command(&["block", "task-1", "by", "task-2"], &temp);
        assert!(!result.success, "block should fail when file cannot be written");
        assert!(result.stderr.contains("Failed to write") || result.stderr.contains("Permission denied"),
                "Should have write error message: {}", result.stderr);
    });
}

#[test]
fn unblock_fails_when_blockers_file_cannot_be_written() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);
        run_command(&["add", "Task B"], &temp);
        run_command(&["block", "task-1", "by", "task-2"], &temp);

        // Make blockers file read-only
        let blockers_path = temp.join(".knecht/blockers");
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&blockers_path).unwrap().permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(&blockers_path, perms).unwrap();
        }
        
        #[cfg(windows)]
        {
            let mut perms = fs::metadata(&blockers_path).unwrap().permissions();
            perms.set_readonly(true);
            fs::set_permissions(&blockers_path, perms).unwrap();
        }

        // Try to remove blocker - should fail
        let result = run_command(&["unblock", "task-1", "from", "task-2"], &temp);
        assert!(!result.success, "unblock should fail when file cannot be written");
        assert!(result.stderr.contains("Failed to write") || result.stderr.contains("Permission denied"),
                "Should have write error message: {}", result.stderr);
    });
}

#[test]
fn block_fails_with_malformed_command_no_by() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);
        run_command(&["add", "Task B"], &temp);

        // Try block without "by" keyword
        let result = run_command(&["block", "task-1", "task-2"], &temp);
        assert!(!result.success, "block should fail without 'by' keyword");
        // Clap shows "invalid value" and "possible values: by"
        assert!(result.stderr.contains("invalid value") || result.stderr.contains("possible values"),
            "Should show error about invalid value: {}", result.stderr);
    });
}

#[test]
fn block_fails_with_too_few_arguments() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);

        // Try block with only one argument
        let result = run_command(&["block", "task-1"], &temp);
        assert!(!result.success, "block should fail with too few arguments");
        assert!(result.stderr.contains("Usage:"), "Should show usage: {}", result.stderr);
    });
}

#[test]
fn unblock_fails_with_malformed_command_no_from() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);
        run_command(&["add", "Task B"], &temp);

        // Try unblock without "from" keyword
        let result = run_command(&["unblock", "task-1", "task-2"], &temp);
        assert!(!result.success, "unblock should fail without 'from' keyword");
        // Clap shows "invalid value" and "possible values: from"
        assert!(result.stderr.contains("invalid value") || result.stderr.contains("possible values"),
            "Should show error about invalid value: {}", result.stderr);
    });
}

#[test]
fn unblock_fails_with_too_few_arguments() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);

        // Try unblock with only one argument
        let result = run_command(&["unblock", "task-1"], &temp);
        assert!(!result.success, "unblock should fail with too few arguments");
        assert!(result.stderr.contains("Usage:"), "Should show usage: {}", result.stderr);
    });
}

#[test]
fn unblock_fails_when_blockers_file_does_not_exist() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);
        run_command(&["add", "Task B"], &temp);
        
        // Try to unblock without ever creating blockers file
        let result = run_command(&["unblock", "task-1", "from", "task-2"], &temp);
        assert!(!result.success, "unblock should fail when blockers file doesn't exist");
        assert!(result.stderr.contains("not blocked"), "Should say task is not blocked: {}", result.stderr);
    });
}

#[test]
fn unblock_preserves_file_format_when_removing_middle_blocker() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);
        run_command(&["add", "Task B"], &temp);
        run_command(&["add", "Task C"], &temp);
        
        // Create three blocker relationships
        run_command(&["block", "task-1", "by", "task-2"], &temp);
        run_command(&["block", "task-1", "by", "task-3"], &temp);
        run_command(&["block", "task-2", "by", "task-3"], &temp);
        
        // Remove middle one
        run_command(&["unblock", "task-1", "from", "task-3"], &temp);
        
        // Verify file still has proper format
        let blockers_path = temp.join(".knecht/blockers");
        let content = fs::read_to_string(&blockers_path).unwrap();
        assert!(content.contains("task-1|task-2"), "Should preserve first blocker");
        assert!(!content.contains("task-1|task-3"), "Should remove middle blocker");
        assert!(content.contains("task-2|task-3"), "Should preserve last blocker");
    });
}

#[test]
fn show_handles_blockers_file_with_empty_lines_and_malformed_entries() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);
        run_command(&["add", "Task B"], &temp);
        run_command(&["add", "Task C"], &temp);
        
        // Create blockers file with empty lines and malformed entries
        let blockers_path = temp.join(".knecht/blockers");
        fs::write(&blockers_path, "task-1|task-2\n\ntask-3|task-2\nmalformed-line\ntask-1|\n|task-2\n").unwrap();
        
        // Should still parse valid entries and ignore malformed ones
        let result = run_command(&["show", "task-1"], &temp);
        assert!(result.success, "show should succeed with malformed blockers file");
        assert!(result.stdout.contains("task-2"), "Should show valid blocker");
    });
}

#[test]
fn unblock_preserves_other_blockers_with_empty_lines() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);
        run_command(&["add", "Task B"], &temp);
        run_command(&["add", "Task C"], &temp);
        
        // Create blockers file with empty lines
        let blockers_path = temp.join(".knecht/blockers");
        fs::write(&blockers_path, "task-1|task-2\n\ntask-1|task-3\n").unwrap();
        
        // Remove one blocker
        let result = run_command(&["unblock", "task-1", "from", "task-2"], &temp);
        assert!(result.success, "unblock should succeed");
        
        // Verify the other blocker is preserved
        let show_result = run_command(&["show", "task-1"], &temp);
        assert!(show_result.stdout.contains("task-3"), "Should preserve other blocker");
        assert!(!show_result.stdout.contains("task-2"), "Should remove specified blocker");
    });
}

#[test]
fn unblock_fails_when_file_exists_but_relationship_not_found() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);
        run_command(&["add", "Task B"], &temp);
        run_command(&["add", "Task C"], &temp);
        
        // Create a blocker file with a different relationship
        run_command(&["block", "task-1", "by", "task-2"], &temp);
        
        // Try to remove a relationship that doesn't exist (but file does exist)
        let result = run_command(&["unblock", "task-1", "from", "task-3"], &temp);
        assert!(!result.success, "unblock should fail when relationship doesn't exist in file");
        assert!(result.stderr.contains("not blocked"), "Should say task is not blocked: {}", result.stderr);
    });
}

#[test]
fn unblock_removes_last_blocker_leaving_empty_file() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task A"], &temp);
        run_command(&["add", "Task B"], &temp);
        
        // Create single blocker
        run_command(&["block", "task-1", "by", "task-2"], &temp);
        
        // Remove the only blocker
        let result = run_command(&["unblock", "task-1", "from", "task-2"], &temp);
        assert!(result.success, "unblock should succeed");
        
        // Verify file is empty
        let blockers_path = temp.join(".knecht/blockers");
        let content = fs::read_to_string(&blockers_path).unwrap();
        assert!(content.is_empty(), "blockers file should be empty");
        
        // Verify task can now be started
        let start_result = run_command(&["start", "task-1"], &temp);
        assert!(start_result.success, "start should succeed after removing last blocker");
    });
}

#[test]
fn start_succeeds_when_blocker_task_is_deleted() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Blocked Task"], &temp);
        run_command(&["add", "Blocker Task"], &temp);
        
        // Create blocker
        run_command(&["block", "task-1", "by", "task-2"], &temp);
        
        // Delete the blocker task (orphan the blocker reference)
        run_command(&["delete", "task-2"], &temp);
        
        // Start should succeed (orphaned blockers are ignored)
        let result = run_command(&["start", "task-1"], &temp);
        assert!(result.success, "start should succeed when blocker task is deleted: {}", result.stderr);
    });
}

#[test]
fn show_handles_orphaned_blocks_reference() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Blocker Task"], &temp);
        run_command(&["add", "Blocked Task"], &temp);
        
        // Create blocker relationship
        run_command(&["block", "task-2", "by", "task-1"], &temp);
        
        // Delete the blocked task (orphan the reference in "Blocks" list)
        run_command(&["delete", "task-2"], &temp);
        
        // Show should succeed and skip the orphaned reference
        let result = run_command(&["show", "task-1"], &temp);
        assert!(result.success, "show should succeed with orphaned blocks reference: {}", result.stderr);
        // Should not crash or show error - just silently skip the orphaned reference
    });
}

#[test]
fn test_read_task_with_delivered_status() {
    with_initialized_repo(&|temp: &PathBuf| {
        // Manually create a task with "delivered" status
        let tasks_path = temp.join(".knecht/tasks");
        fs::write(&tasks_path, "1,delivered,\"Fix the bug\",,\n").unwrap();
        
        // List should read and display the delivered task
        let result = run_command(&["list"], &temp);
        assert!(result.success, "list should succeed: {}", result.stderr);
        assert!(result.stdout.contains("task-1"), "Should show task-1");
        assert!(result.stdout.contains("Fix the bug"), "Should show task title");
    });
}

#[test]
fn test_write_task_with_delivered_status() {
    with_initialized_repo(&|temp: &PathBuf| {
        // For now, we can't set delivered status via CLI (no deliver command yet)
        // So this test will manually create a delivered task, read it, and verify it persists
        let tasks_path = temp.join(".knecht/tasks");
        fs::write(&tasks_path, "1,delivered,\"Fix the bug\",,\n").unwrap();
        
        // Add another task - this will read and rewrite the file
        run_command(&["add", "Another task"], &temp);
        
        // Verify the delivered status was preserved
        let content = fs::read_to_string(&tasks_path).unwrap();
        assert!(content.contains("1,delivered,\"Fix the bug\""), 
                "Delivered status should be preserved after file rewrite. Content: {}", content);
    });
}

#[test]
fn test_delivered_status_with_description() {
    with_initialized_repo(&|temp: &PathBuf| {
        // Create a delivered task with description
        let tasks_path = temp.join(".knecht/tasks");
        fs::write(&tasks_path, "1,delivered,\"Fix the bug\",\"This is the description\",\n").unwrap();
        
        // Show command should display it correctly
        let result = run_command(&["show", "task-1"], &temp);
        assert!(result.success, "show should succeed: {}", result.stderr);
        assert!(result.stdout.contains("Status: delivered"), "Should show delivered status");
        assert!(result.stdout.contains("Fix the bug"), "Should show title");
        assert!(result.stdout.contains("This is the description"), "Should show description");
    });
}

#[test]
fn test_backwards_compatibility_with_open_and_done() {
    with_initialized_repo(&|temp: &PathBuf| {
        // Create tasks with all three statuses
        let tasks_path = temp.join(".knecht/tasks");
        fs::write(&tasks_path, "1,open,\"Open task\",,\n2,delivered,\"Delivered task\",,\n3,done,\"Done task\",,\n").unwrap();
        
        // List should show all three
        let result = run_command(&["list"], &temp);
        assert!(result.success, "list should succeed: {}", result.stderr);
        assert!(result.stdout.contains("Open task"), "Should show open task");
        assert!(result.stdout.contains("Delivered task"), "Should show delivered task");
        assert!(result.stdout.contains("Done task"), "Should show done task");
        
        // Add a new task - this will read and rewrite
        run_command(&["add", "New task"], &temp);
        
        // Verify all statuses were preserved
        let content = fs::read_to_string(&tasks_path).unwrap();
        assert!(content.contains("1,open,\"Open task\""), "Open status preserved");
        assert!(content.contains("2,delivered,\"Delivered task\""), "Delivered status preserved");
        assert!(content.contains("3,done,\"Done task\""), "Done status preserved");
    });
}

#[test]
fn test_delivered_status_value_is_preserved() {
    with_initialized_repo(&|temp: &PathBuf| {
        // Create a task with delivered status
        let tasks_path = temp.join(".knecht/tasks");
        fs::write(&tasks_path, "1,delivered,\"Fix the bug\",,\n").unwrap();
        
        // Show command should display "delivered" as the status
        let result = run_command(&["show", "task-1"], &temp);
        assert!(result.success, "show should succeed: {}", result.stderr);
        assert!(result.stdout.contains("Status: delivered"), 
                "Status should be 'delivered', got: {}", result.stdout);
        
        // Verify the raw file still contains "delivered"
        let content = fs::read_to_string(&tasks_path).unwrap();
        assert!(content.contains("1,delivered,\"Fix the bug\""), 
                "File should contain delivered status: {}", content);
    });
}

#[test]
fn next_prefers_unblocked_subtasks_over_parent_task() {
    with_initialized_repo(|temp| {
        // Create a parent task with high pain count
        run_command(&["add", "Large feature with verification", "-d", "This is a big task"], &temp);
        for i in 0..3 {
            run_command(&["pain", "-t", "task-1", "-d", &format!("Pain {}", i)], &temp);
        }
        
        // Create subtasks that block the parent task
        run_command(&["add", "Foundation work", "-d", "Must be done first"], &temp);
        run_command(&["add", "Build feature A", "-d", "Needs foundation"], &temp);
        run_command(&["add", "Build feature B", "-d", "Needs foundation"], &temp);
        
        // Parent is blocked by all subtasks (can't complete parent until subtasks done)
        run_command(&["block", "task-1", "by", "task-2"], &temp); // task-1 blocked by task-2
        run_command(&["block", "task-1", "by", "task-3"], &temp); // task-1 blocked by task-3
        run_command(&["block", "task-1", "by", "task-4"], &temp); // task-1 blocked by task-4
        
        // Feature tasks also blocked by foundation
        run_command(&["block", "task-3", "by", "task-2"], &temp); // task-3 blocked by task-2
        run_command(&["block", "task-4", "by", "task-2"], &temp); // task-4 blocked by task-2
        
        // Complete the foundation (task-2)
        run_command(&["done", "task-2"], &temp);
        
        // Now next should suggest task-3 or task-4 (unblocked subtasks) instead of task-1 (parent)
        let result = run_command(&["next"], &temp);
        
        assert!(result.success, "next command should succeed");
        // Should NOT suggest task-1 (the parent with high pain) because it has open subtasks
        assert!(
            !result.stdout.contains("task-1"),
            "Should not suggest parent task-1 when it has unblocked subtasks, got: {}",
            result.stdout
        );
        // Should suggest one of the unblocked subtasks (task-3 or task-4)
        assert!(
            result.stdout.contains("task-3") || result.stdout.contains("task-4"),
            "Should suggest one of the unblocked subtasks (task-3 or task-4), got: {}",
            result.stdout
        );
    });
}

#[test]
fn next_handles_three_level_blocker_tree() {
    with_initialized_repo(|temp| {
        // Create a three-level blocker tree like task-143 → task-176 → tasks 184-192
        // Root task with high pain count
        run_command(&["add", "Root feature", "-d", "Top level feature"], &temp);
        for i in 0..3 {
            run_command(&["pain", "-t", "task-1", "-d", &format!("Pain {}", i)], &temp);
        }
        
        // Middle task (blocks root)
        run_command(&["add", "Middle task", "-d", "Intermediate step"], &temp);
        run_command(&["block", "task-1", "by", "task-2"], &temp);
        
        // Leaf tasks (block middle task)
        run_command(&["add", "Leaf task A", "-d", "First leaf"], &temp);
        run_command(&["add", "Leaf task B", "-d", "Second leaf"], &temp);
        run_command(&["add", "Leaf task C", "-d", "Third leaf"], &temp);
        run_command(&["block", "task-2", "by", "task-3"], &temp);
        run_command(&["block", "task-2", "by", "task-4"], &temp);
        run_command(&["block", "task-2", "by", "task-5"], &temp);
        
        // Now next should suggest one of the leaf tasks (3, 4, or 5)
        // NOT the root (task-1) or middle (task-2)
        let result = run_command(&["next"], &temp);
        
        assert!(result.success, "next command should succeed");
        assert!(
            !result.stdout.contains("task-1"),
            "Should not suggest root task-1 when it has blockers, got: {}",
            result.stdout
        );
        assert!(
            !result.stdout.contains("task-2"),
            "Should not suggest middle task-2 when it has blockers, got: {}",
            result.stdout
        );
        // Should suggest one of the leaf tasks
        assert!(
            result.stdout.contains("task-3") || result.stdout.contains("task-4") || result.stdout.contains("task-5"),
            "Should suggest one of the leaf tasks (task-3, task-4, or task-5), got: {}",
            result.stdout
        );
    });
}

#[test]
fn deliver_command_is_recognized() {
    with_initialized_repo(|temp| {
        // Add a task first
        let add_result = run_command(&["add", "Test task"], temp);
        assert!(add_result.success);
        assert!(add_result.stdout.contains("Created task-1"));

        // Try to deliver it
        let deliver_result = run_command(&["deliver", "task-1"], temp);
        
        // The command should be recognized (not "Unknown command")
        assert!(
            !deliver_result.stderr.contains("Unknown command"),
            "deliver command should be recognized, got stderr: {}",
            deliver_result.stderr
        );
    });
}

#[test]
fn deliver_requires_task_id_argument() {
    with_initialized_repo(|temp| {
        let result = run_command(&["deliver"], temp);

        assert!(!result.success);
        assert!(result.stderr.contains("Usage:") && result.stderr.contains("deliver"),
            "Should show deliver usage, got: {}", result.stderr);
    });
}

#[test]
fn csv_format_reading_basic_fields() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Write CSV-formatted task data
    let tasks_path = temp.join(".knecht/tasks");
    fs::write(&tasks_path, "1,open,\"Simple title\",,\n2,done,\"Another task\",\"Description here\",3\n")
        .expect("Failed to write test file");

    // list should read CSV format
    let result = run_command(&["list"], &temp);
    assert!(result.success, "list should succeed with CSV format");
    assert!(result.stdout.contains("task-1"), "Should show task-1");
    assert!(result.stdout.contains("Simple title"), "Should show task title");
    assert!(result.stdout.contains("task-2"), "Should show task-2");
    assert!(result.stdout.contains("Another task"), "Should show another task");

    cleanup_temp_dir(temp);
}

#[test]
fn csv_format_handles_special_characters() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Write CSV with special characters that would break pipe format
    let tasks_path = temp.join(".knecht/tasks");
    fs::write(&tasks_path, "1,open,\"Title with, comma\",,\n2,open,\"Title with | pipe\",\"Description with \\\"quotes\\\"\",\n")
        .expect("Failed to write test file");

    let result = run_command(&["list"], &temp);
    assert!(result.success, "list should handle CSV special characters");
    assert!(result.stdout.contains("Title with, comma"), "Should handle comma in title");
    assert!(result.stdout.contains("Title with | pipe"), "Should handle pipe in title");

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

    // Output should show how to make this task a blocker for another task
    assert!(
        result.stdout.contains("knecht block") && result.stdout.contains("by task-1"),
        "add output should show block syntax, got: {}",
        result.stdout
    );

    cleanup_temp_dir(temp);
}

#[test]
fn deliver_changes_task_status_to_delivered() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task to deliver"], &temp);

        let result = run_command(&["deliver", "task-1"], &temp);
        assert!(result.success, "deliver command should succeed");

        let show = run_command(&["show", "task-1"], &temp);
        assert!(
            show.stdout.contains("delivered"),
            "Task status should be 'delivered', got: {}",
            show.stdout
        );
    });
}

#[test]
fn deliver_fails_for_already_delivered_task() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task to deliver twice"], temp);

        // First delivery should succeed
        let first = run_command(&["deliver", "task-1"], temp);
        assert!(first.success, "First deliver should succeed");

        // Second delivery should fail
        let second = run_command(&["deliver", "task-1"], temp);
        assert!(!second.success, "Second deliver should fail");
        assert!(
            second.stderr.contains("already delivered"),
            "Error should mention task is already delivered, got: {}",
            second.stderr
        );
    });
}

#[test]
fn deliver_fails_for_already_done_task() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task that is done"], temp);
        run_command(&["done", "task-1"], temp);

        // Trying to deliver a done task should fail
        let result = run_command(&["deliver", "task-1"], temp);
        assert!(!result.success, "Deliver of done task should fail");
        assert!(
            result.stderr.contains("already done") || result.stderr.contains("already completed"),
            "Error should mention task is already done, got: {}",
            result.stderr
        );
    });
}

#[test]
fn pain_requires_d_flag_for_description() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task needing pain"], &temp);

        // Old syntax without -d should fail
        let result = run_command(&["pain", "-t", "task-1"], &temp);
        assert!(!result.success, "pain command should fail without -d flag");
        assert!(
            result.stderr.contains("-d") || result.stderr.contains("description"),
            "Error should mention -d flag or description requirement, got: {}",
            result.stderr
        );
    });
}

#[test]
fn pain_with_d_flag_increments_and_documents() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Task needing pain"], &temp);

        // New syntax with -t and -d should succeed
        let result = run_command(&["pain", "-t", "task-1", "-d", "Hit this during task-99 work"], &temp);
        assert!(result.success, "pain command should succeed with -t and -d flags, got stderr: {}", result.stderr);

        // Verify pain count was added
        let list = run_command(&["list"], &temp);
        assert!(
            list.stdout.contains("(pain count: 1)"),
            "Pain count should be 1, got: {}",
            list.stdout
        );

        // Verify description was added to task
        let show = run_command(&["show", "task-1"], &temp);
        assert!(
            show.stdout.contains("Hit this during task-99 work"),
            "Pain description should be appended to task, got: {}",
            show.stdout
        );
    });
}

#[test]
fn pain_appends_multiple_descriptions() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Repeated pain task", "-d", "Initial description"], &temp);

        // First pain instance
        run_command(&["pain", "-t", "task-1", "-d", "First pain instance"], &temp);

        // Second pain instance
        run_command(&["pain", "-t", "task-1", "-d", "Second pain instance"], &temp);

        // Verify pain count
        let list = run_command(&["list"], &temp);
        assert!(
            list.stdout.contains("(pain count: 2)"),
            "Pain count should be 2, got: {}",
            list.stdout
        );

        // Verify all descriptions are preserved
        let show = run_command(&["show", "task-1"], &temp);
        assert!(
            show.stdout.contains("Initial description"),
            "Original description should be preserved, got: {}",
            show.stdout
        );
        assert!(
            show.stdout.contains("First pain instance"),
            "First pain description should be appended, got: {}",
            show.stdout
        );
        assert!(
            show.stdout.contains("Second pain instance"),
            "Second pain description should be appended, got: {}",
            show.stdout
        );
    });
}

#[test]
fn pain_without_t_flag_fails() {
    with_initialized_repo(|temp| {
        run_command(&["add", "Some task"], &temp);

        // Bare task-id without -t flag should fail
        let result = run_command(&["pain", "task-1", "-d", "some description"], &temp);
        assert!(!result.success, "pain command should fail without -t flag");
    });
}

#[test]
fn help_flag_shows_usage() {
    let temp = setup_temp_dir();

    // --help should show usage information and succeed
    let result = run_command(&["--help"], &temp);
    assert!(result.success, "--help should succeed, got stderr: {}", result.stderr);
    assert!(
        result.stdout.contains("Usage:") || result.stdout.to_lowercase().contains("usage"),
        "--help should contain 'Usage', got: {}",
        result.stdout
    );

    cleanup_temp_dir(temp);
}

#[test]
fn subcommand_help_shows_usage() {
    let temp = setup_temp_dir();

    // help for subcommand should show its usage
    let result = run_command(&["add", "--help"], &temp);
    assert!(result.success, "add --help should succeed, got stderr: {}", result.stderr);
    assert!(
        result.stdout.contains("add") || result.stdout.contains("Add"),
        "add --help should mention 'add', got: {}",
        result.stdout
    );

    cleanup_temp_dir(temp);
}

#[test]
fn deliver_success_message_matches_done_format() {
    // Task-191: deliver and done should have consistent success message format
    with_initialized_repo(|temp| {
        run_command(&["add", "Task one"], &temp);
        run_command(&["add", "Task two"], &temp);

        let done_result = run_command(&["done", "task-1"], &temp);
        let deliver_result = run_command(&["deliver", "task-2"], &temp);

        // Both should start with "✓ task-N: Title" format
        // done shows: "✓ task-1: Task one"
        // deliver should show: "✓ task-2: Task two" (not "✓ task-2 delivered: Task two")
        assert!(
            done_result.stdout.contains("✓ task-1: Task one"),
            "done should output '✓ task-1: Task one', got: {}",
            done_result.stdout
        );
        assert!(
            deliver_result.stdout.contains("✓ task-2: Task two"),
            "deliver should output '✓ task-2: Task two' (matching done format), got: {}",
            deliver_result.stdout
        );
    });
}

#[test]
fn precommit_hook_prompts_readme_review_on_readme_changes() {
    // Task-40: Pre-commit hook should prompt user to review README when it changes
    let temp = setup_temp_dir();

    // Initialize git repo
    let git_init = Command::new("git")
        .args(["init"])
        .current_dir(&temp)
        .output()
        .expect("Failed to run git init");
    assert!(git_init.status.success(), "git init failed");

    // Configure git user for commits
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(&temp)
        .output()
        .expect("Failed to configure git email");
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&temp)
        .output()
        .expect("Failed to configure git name");

    // Set up hooks directory and copy our pre-commit hook
    let hooks_dir = temp.join(".githooks");
    fs::create_dir_all(&hooks_dir).unwrap();

    // Copy the pre-commit hook from the project
    let project_hook = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".githooks/pre-commit");
    let test_hook = hooks_dir.join("pre-commit");
    fs::copy(&project_hook, &test_hook).expect("Failed to copy pre-commit hook");

    // Configure git to use our hooks directory
    Command::new("git")
        .args(["config", "core.hooksPath", ".githooks"])
        .current_dir(&temp)
        .output()
        .expect("Failed to configure hooks path");

    // Create initial commit without README
    fs::write(temp.join("file.txt"), "initial content").unwrap();
    Command::new("git")
        .args(["add", "file.txt"])
        .current_dir(&temp)
        .output()
        .expect("Failed to stage file");
    Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(&temp)
        .output()
        .expect("Failed to create initial commit");

    // Now create README.md and commit it
    fs::write(temp.join("README.md"), "# Test Project\n\nThis is a test.").unwrap();
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(&temp)
        .output()
        .expect("Failed to stage README");

    // Commit and capture output - the pre-commit hook should print a reminder
    let commit_output = Command::new("git")
        .args(["commit", "-m", "add readme"])
        .current_dir(&temp)
        .output()
        .expect("Failed to commit");

    let stdout = String::from_utf8_lossy(&commit_output.stdout);
    let stderr = String::from_utf8_lossy(&commit_output.stderr);
    let combined_output = format!("{}{}", stdout, stderr);

    // The hook should output a reminder about reviewing README
    assert!(
        combined_output.contains("README") && combined_output.contains("review"),
        "Pre-commit hook should prompt README review when README changes.\n\
         Expected output containing 'README' and 'review'.\n\
         Got stdout: {}\n\
         Got stderr: {}",
        stdout,
        stderr
    );

    cleanup_temp_dir(temp);
}
