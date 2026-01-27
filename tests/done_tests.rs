mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

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
    // task-221: Reflection content moved to /reflect skill file
    // This test now verifies the skill file contains the expected guidance
    let skill_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".claude/commands/reflect.md");

    let skill_content = fs::read_to_string(&skill_path)
        .expect("Reflect skill file should exist at .claude/commands/reflect.md");

    // Verify the skill file contains key reflection questions
    assert!(skill_content.contains("What friction did you encounter"),
        "Should ask about friction");
    assert!(skill_content.contains("Did the user correct or redirect you"),
        "Should ask about user corrections");
    assert!(skill_content.contains("What IS a knecht bug"),
        "Should explain what qualifies as a knecht bug");
    assert!(skill_content.contains("REQUIRED ACTION"),
        "Should have required action section");
    assert!(skill_content.contains("knecht add") && skill_content.contains("knecht pain"),
        "Should mention knecht commands to file tasks");
}

#[test]
fn done_marks_task_complete() {
    with_initialized_repo(|temp| {
        let add_result = run_command(&["add", "Task to complete", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        let result = run_command(&["done", &format!("task-{}", task_id)], &temp);
        assert!(result.success, "done command should succeed");

        // Use --all since done tasks are hidden by default
        let list = run_command(&["list", "--all"], &temp);
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
fn done_handles_invalid_task_id_formats() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    run_command(&["add", "Test task", "-a", "Done"], &temp);

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
fn done_reflection_prompt_uses_actionable_language() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    let add_result = run_command(&["add", "Task to complete", "-a", "Done"], &temp);
    let task_id = extract_task_id(&add_result.stdout);

    let result = run_command(&["done", &format!("task-{}", task_id)], &temp);

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
    // The reflection skill should warn agents against dismissing issues as "not a knecht bug".
    // The key insight: if you're explaining why something isn't knecht's problem,
    // that explanation IS the task to file.
    let skill_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".claude/commands/reflect.md");

    let skill_content = fs::read_to_string(&skill_path)
        .expect("Reflect skill file should exist at .claude/commands/reflect.md");

    // Check for the anti-dismissal guidance (task-221 core feature)
    assert!(skill_content.contains("Anti-Dismissal Rule"),
        "Should have anti-dismissal rule section");
    assert!(skill_content.contains("this isn't really a knecht bug"),
        "Should mention the problematic thought pattern");
    assert!(skill_content.contains("STOP"),
        "Should tell agent to stop when having dismissive thoughts");
    assert!(skill_content.contains("File it AS a task anyway"),
        "Should tell agent to file it as a task regardless");
    assert!(skill_content.contains("Your reasoning about why it's not knecht's problem IS the task content"),
        "Should explain that the reasoning itself is the task content");
}

#[test]
fn done_shows_commit_reminder() {
    // Commit reminder is now in the /reflect skill file
    let skill_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".claude/commands/reflect.md");

    let skill_content = fs::read_to_string(&skill_path)
        .expect("Reflect skill file should exist at .claude/commands/reflect.md");

    assert!(skill_content.contains("Commit Reminder"),
        "Should have commit reminder section");
    assert!(skill_content.contains("git add .knecht/tasks"),
        "Should show git add command");
    assert!(skill_content.contains("git commit"),
        "Should show git commit command");
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
fn done_marks_task_without_description_complete() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Add task without description
    let add_result = run_command(&["add", "Task without description", "-a", "Done"], &temp);
    let task_id = extract_task_id(&add_result.stdout);

    // Mark it done
    let result = run_command(&["done", &format!("task-{}", task_id)], &temp);
    assert!(result.success, "done command should succeed");

    // Verify it was marked done (use --all since done tasks are hidden by default)
    let list = run_command(&["list", "--all"], &temp);
    assert!(list.stdout.contains("[x]"), "Task should be marked done");

    cleanup_temp_dir(temp);
}

#[test]
fn done_marks_task_with_description_complete() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Add task with description
    let add_result = run_command(&["add", "Task with description", "-d", "This is the description", "-a", "Done"], &temp);
    let task_id = extract_task_id(&add_result.stdout);

    // Mark it done
    let result = run_command(&["done", &format!("task-{}", task_id)], &temp);
    assert!(result.success, "done command should succeed");

    // Verify it was marked done and still has description (now reading individual task file)
    let tasks_content = fs::read_to_string(temp.join(format!(".knecht/tasks/{}", task_id))).unwrap();
    assert!(tasks_content.contains(&format!("{},done", task_id)), "Task should be marked done");
    assert!(tasks_content.contains("Task with description"), "Should preserve title");
    assert!(tasks_content.contains("This is the description"), "Should preserve description");

    cleanup_temp_dir(temp);
}

#[test]
fn done_increments_pain_on_skipped_top_task() {
    with_initialized_repo(|temp| {
        // Create two tasks
        let r1 = run_command(&["add", "Primary feature work", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Minor improvement", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Determine which ID is lexicographically smaller (that's the "top" task for tiebreaker)
        let (top_id, other_id) = if id1 < id2 { (&id1, &id2) } else { (&id2, &id1) };

        // Complete the non-top task (skipping the top task)
        let done_result = run_command(&["done", &format!("task-{}", other_id)], &temp);
        assert!(done_result.success, "done should succeed");

        // Check that top task's pain count increased (it was skipped)
        let list_result = run_command(&["list"], &temp);
        let top_task_line = list_result.stdout.lines()
            .find(|line| line.contains(&format!("task-{}", top_id)))
            .expect("Should find top task in list output");

        // Pain should have incremented from 0 to 1
        assert!(top_task_line.contains("(pain count: 1)"),
            "top task pain should increment to 1 when skipped, got: {}", top_task_line);

        // Check top task's description mentions it was skipped
        let show_result = run_command(&["show", &format!("task-{}", top_id)], &temp);
        assert!(show_result.stdout.contains(&format!("Skip: task-{} completed instead", other_id)) ||
                show_result.stdout.contains("Skip:"),
            "top task description should note it was skipped, got: {}", show_result.stdout);
    });
}

#[test]
fn done_on_oldest_task_does_not_increment_pain() {
    with_initialized_repo(|temp| {
        // Create two tasks
        let r1 = run_command(&["add", "First task", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Second task", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Determine which ID is lexicographically smaller (that's the "top" task)
        let (top_id, other_id) = if id1 < id2 { (&id1, &id2) } else { (&id2, &id1) };

        // Complete the top task (not skipping it)
        let done_result = run_command(&["done", &format!("task-{}", top_id)], &temp);
        assert!(done_result.success);

        // Verify other task still has no pain (it wasn't skipped - we did the top first)
        let list_result = run_command(&["list"], &temp);
        let other_task_line = list_result.stdout.lines()
            .find(|line| line.contains(&format!("task-{}", other_id)))
            .expect("Should find other task");

        assert!(!other_task_line.contains("pain count:"),
            "other task should have no pain when top task was completed, got: {}", other_task_line);
    });
}

#[test]
fn done_increments_pain_on_task_with_existing_description() {
    with_initialized_repo(|temp| {
        // Create first task with a description
        let r1 = run_command(&["add", "Primary feature", "-d", "Original description", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);

        // Create second task
        let r2 = run_command(&["add", "Minor task", "-a", "Done"], &temp);
        let id2 = extract_task_id(&r2.stdout);

        // Determine which ID is lexicographically smaller (that's the "top" task)
        let (top_id, other_id) = if id1 < id2 { (&id1, &id2) } else { (&id2, &id1) };

        // Complete the non-top task, skipping the top task
        let done_result = run_command(&["done", &format!("task-{}", other_id)], &temp);
        assert!(done_result.success);

        // Verify top task's pain incremented and skip note was appended to existing description
        let show_result = run_command(&["show", &format!("task-{}", top_id)], &temp);
        // Only the task with description (id1) will have "Original description"
        if top_id == &id1 {
            assert!(show_result.stdout.contains("Original description"),
                "Should preserve original description");
        }
        assert!(show_result.stdout.contains(&format!("Skip: task-{} completed instead", other_id)),
            "Should append skip note to description, got: {}", show_result.stdout);
    });
}

#[test]
fn done_instructs_agent_to_run_reflect_skill() {
    // task-221: Instead of inline reflection prompts, instruct agent to run /reflect skill
    with_initialized_repo(|temp| {
        let add_result = run_command(&["add", "Task to complete", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        let result = run_command(&["done", &format!("task-{}", task_id)], &temp);

        assert!(result.success, "done should succeed: {}", result.stderr);
        assert!(result.stdout.contains("/reflect"),
            "Should instruct agent to run /reflect skill, got: {}", result.stdout);
    });
}

#[test]
fn done_fails_for_already_done_task() {
    // task-228: 'done' on already-done task should fail like 'deliver' on already-delivered
    with_initialized_repo(|temp| {
        let add_result = run_command(&["add", "Task to mark done twice", "-a", "Done"], temp);
        let task_id = extract_task_id(&add_result.stdout);

        // First done should succeed
        let first = run_command(&["done", &format!("task-{}", task_id)], temp);
        assert!(first.success, "First done should succeed");

        // Second done should fail
        let second = run_command(&["done", &format!("task-{}", task_id)], temp);
        assert!(!second.success, "Second done should fail");
        assert!(
            second.stderr.contains("already done"),
            "Error should mention task is already done, got: {}",
            second.stderr
        );
    });
}
