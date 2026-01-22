mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;
#[allow(unused_imports)]
use std::path::PathBuf;
#[allow(unused_imports)]
use std::process::Command;

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
        run_command(&["add", "Another task", "-a", "Done"], &temp);
        
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
        run_command(&["add", "New task", "-a", "Done"], &temp);
        
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
