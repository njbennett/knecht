mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn pain_increments_pain_count_on_task() {
    with_initialized_repo(|temp| {
        // Add a task without pain count
        let add_result = run_command(&["add", "Fix bug", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Increment pain count (should add it as 1)
        let result = run_command(&["pain", "-t", &format!("task-{}", task_id), "-d", "First occurrence"], &temp);
        assert!(result.success, "pain command should succeed");

        // Verify pain count was added as 1
        let list = run_command(&["list"], &temp);
        assert!(
            list.stdout.contains("Fix bug (pain count: 1)"),
            "Pain count should be added as 1, got: {}",
            list.stdout
        );

        // Increment again
        let result2 = run_command(&["pain", "-t", &format!("task-{}", task_id), "-d", "Second occurrence"], &temp);
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
        let add_result = run_command(&["add", "Some task", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Increment pain count
        let result = run_command(&["pain", "-t", &format!("task-{}", task_id), "-d", "Pain instance"], &temp);
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
        let add_result = run_command(&["add", "Fix critical bug", "-d", "This bug breaks production", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Add pain count
        run_command(&["pain", "-t", &format!("task-{}", task_id), "-d", "First pain"], &temp);

        // Increment pain count again
        let result = run_command(&["pain", "-t", &format!("task-{}", task_id), "-d", "Second pain"], &temp);
        assert!(result.success, "pain command should succeed on task with description");

        // Verify both description and pain count are preserved
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
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
fn pain_requires_d_flag_for_description() {
    with_initialized_repo(|temp| {
        let add_result = run_command(&["add", "Task needing pain", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Old syntax without -d should fail
        let result = run_command(&["pain", "-t", &format!("task-{}", task_id)], &temp);
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
        let add_result = run_command(&["add", "Task needing pain", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // New syntax with -t and -d should succeed
        let result = run_command(&["pain", "-t", &format!("task-{}", task_id), "-d", "Hit this during task-99 work"], &temp);
        assert!(result.success, "pain command should succeed with -t and -d flags, got stderr: {}", result.stderr);

        // Verify pain count was added
        let list = run_command(&["list"], &temp);
        assert!(
            list.stdout.contains("(pain count: 1)"),
            "Pain count should be 1, got: {}",
            list.stdout
        );

        // Verify description was added to task
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
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
        let add_result = run_command(&["add", "Repeated pain task", "-d", "Initial description", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // First pain instance
        run_command(&["pain", "-t", &format!("task-{}", task_id), "-d", "First pain instance"], &temp);

        // Second pain instance
        run_command(&["pain", "-t", &format!("task-{}", task_id), "-d", "Second pain instance"], &temp);

        // Verify pain count
        let list = run_command(&["list"], &temp);
        assert!(
            list.stdout.contains("(pain count: 2)"),
            "Pain count should be 2, got: {}",
            list.stdout
        );

        // Verify all descriptions are preserved
        let show = run_command(&["show", &format!("task-{}", task_id)], &temp);
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
        let add_result = run_command(&["add", "Some task", "-a", "Done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Bare task-id without -t flag should fail
        let result = run_command(&["pain", &format!("task-{}", task_id), "-d", "some description"], &temp);
        assert!(!result.success, "pain command should fail without -t flag");
    });
}
