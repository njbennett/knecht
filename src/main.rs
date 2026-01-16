use std::env;
use std::fs;

use knecht::{add_task_with_fs, delete_task_with_fs, find_next_task_with_fs, find_task_by_id_with_fs, increment_pain_count_with_fs, mark_task_delivered_with_fs, mark_task_done_with_fs, read_tasks_with_fs, update_task_with_fs, RealFileSystem};

/// Parses a task ID argument, stripping the "task-" prefix if present.
/// Accepts both "task-N" and "N" formats, returning just the numeric ID part.
fn parse_task_id(task_arg: &str) -> &str {
    task_arg.strip_prefix("task-").unwrap_or(task_arg)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: knecht <command> [args]");
        std::process::exit(1);
    }
    
    match args[1].as_str() {
        "init" => cmd_init(),
        "add" => cmd_add(&args[2..]),
        "list" => cmd_list(),
        "done" => cmd_done(&args[2..]),
        "deliver" => cmd_deliver(&args[2..]),
        "delete" => cmd_delete(&args[2..]),
        "show" => cmd_show(&args[2..]),
        "start" => cmd_start(&args[2..]),
        "pain" => cmd_pain(&args[2..]),
        "next" => cmd_next(),
        "update" => cmd_update(&args[2..]),
        "block" => cmd_block(&args[2..]),
        "unblock" => cmd_unblock(&args[2..]),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn cmd_init() {
    if let Err(e) = fs::create_dir_all(".knecht") {
        eprintln!("Failed to create .knecht directory: {}", e);
        std::process::exit(1);
    }
    
    if let Err(e) = fs::write(".knecht/tasks", "") {
        eprintln!("Failed to create tasks file: {}", e);
        std::process::exit(1);
    }
    
    println!("Initialized knecht");
}

fn cmd_add(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: knecht add <title> [-d <description>]");
        std::process::exit(1);
    }
    
    // Parse args to find -d flag
    let mut title_parts = Vec::new();
    let mut description = None;
    let mut i = 0;
    
    while i < args.len() {
        if args[i] == "-d" {
            // Next args form the description
            i += 1;
            let mut desc_parts = Vec::new();
            while i < args.len() && args[i] != "-d" {
                desc_parts.push(args[i].clone());
                i += 1;
            }
            description = Some(desc_parts.join(" "));
        } else {
            title_parts.push(args[i].clone());
            i += 1;
        }
    }
    
    let title = title_parts.join(" ");
    
    if title.is_empty() {
        eprintln!("Error: Title cannot be empty");
        std::process::exit(1);
    }
    
    match add_task_with_fs(title, description, &RealFileSystem) {
        Ok(task_id) => {
            println!("Created task-{}", task_id);
            println!("To make another task blocked by this: knecht block <task> by task-{}", task_id);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_list() {
    let tasks = match read_tasks_with_fs(&RealFileSystem) {
        Ok(tasks) => tasks,
        Err(e) => {
            eprintln!("Error reading tasks: {}", e);
            std::process::exit(1);
        }
    };
    
    for task in tasks {
        let checkbox = if task.is_done() { "[x]" } else { "[ ]" };
        let pain_suffix = if let Some(count) = task.pain_count {
            format!(" (pain count: {})", count)
        } else {
            String::new()
        };
        println!("{} task-{}  {}{}", checkbox, task.id, task.title, pain_suffix);
    }
    
    // Print usage instructions for agents
    println!();
    println!("Usage instructions:");
    println!("  knecht show task-N     - View full task details including description");
    println!("  knecht start task-N    - Begin work on a task");
    println!("  knecht done task-N     - Mark a task as complete");
    println!("  knecht next            - Get suggestion for what to work on next");
}

fn cmd_deliver(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: knecht deliver <task-id>");
        std::process::exit(1);
    }

    let task_arg = &args[0];
    let task_id = parse_task_id(task_arg);

    match mark_task_delivered_with_fs(task_id, &RealFileSystem) {
        Ok(task) => {
            println!("✓ task-{} delivered: {}", task.id, task.title);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_done(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: knecht done <task-id>");
        std::process::exit(1);
    }
    
    let task_arg = &args[0];
    let task_id = parse_task_id(task_arg);
    
    match mark_task_done_with_fs(task_id, &RealFileSystem) {
        Ok(task) => {
            println!("✓ task-{}: {}", task.id, task.title);
            println!();
            print!("
================================================================================
STOP - REQUIRED REFLECTION - You MUST answer these questions before continuing
================================================================================

This is NOT optional informational text. This is REQUIRED work.
Create tasks immediately for anything you notice:

1. Did you notice anything missing from knecht's interface during this work?
   → If YOU were confused about workflow or what to do next, that's a KNECHT UX BUG.
   → Create a task describing what knecht should have told you but didn't.

2. Did the user have to correct or redirect you about anything?
   → That's a KNECHT UX BUG, not just 'you misunderstood'.
   → Create a task: How could knecht's output have prevented this confusion?

3. Did you read .knecht/tasks directly or use grep instead of knecht commands?
   → That's a KNECHT UX BUG - the interface should be better than raw file access.
   → Create a task: What's missing from knecht's output that made you bypass it?

4. Did you notice anything new that was difficult about working with the codebase while you did this work? Is there anything in the work you just did that we should refactor? Make a list of the refactoring opportunities. Where you can, use named refactors from Martin Fowler's Refactoring, or Michael Feather's Working Effectively with Legacy Code. Check knecht to see if anything similar has already been filed, and if so, increase the pain count on those tasks.

IMPORTANT: If agents are confused, knecht needs to improve. Create tasks NOW, don't just note it.

================================================================================
");
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn cmd_show(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: knecht show <task-id>");
        std::process::exit(1);
    }
    
    let task_arg = &args[0];
    let task_id = parse_task_id(task_arg);
    
    match find_task_by_id_with_fs(task_id, &RealFileSystem) {
        Ok(task) => {
            println!("Task: task-{}", task.id);
            println!("Status: {}", task.status);
            println!("Title: {}", task.title);
            if let Some(desc) = &task.description {
                println!("Description: {}", desc);
            }
            
            // Display blockers
            let blockers = get_blockers_for_task(task_id);
            if !blockers.is_empty() {
                println!("Blocked by:");
                for blocker_id in &blockers {
                    if let Ok(blocker_task) = find_task_by_id_with_fs(blocker_id, &RealFileSystem) {
                        println!("  - task-{} ({}): {}", blocker_task.id, blocker_task.status, blocker_task.title);
                    }
                }
            }
            
            // Display what this task blocks
            let blocks = get_tasks_blocked_by(task_id);
            if !blocks.is_empty() {
                println!("Blocks:");
                for blocked_id in &blocks {
                    if let Ok(blocked_task) = find_task_by_id_with_fs(blocked_id, &RealFileSystem) {
                        println!("  - task-{} ({}): {}", blocked_task.id, blocked_task.status, blocked_task.title);
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn cmd_start(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: knecht start <task-id>");
        std::process::exit(1);
    }
    
    let task_arg = &args[0];
    let task_id = parse_task_id(task_arg);
    
    match find_task_by_id_with_fs(task_id, &RealFileSystem) {
        Ok(task) => {
            // Check for open blockers
            let blockers = get_blockers_for_task(task_id);
            let mut open_blockers = Vec::new();
            
            for blocker_id in &blockers {
                if let Ok(blocker_task) = find_task_by_id_with_fs(blocker_id, &RealFileSystem)
                    && blocker_task.status != "done" {
                        open_blockers.push((blocker_id.clone(), blocker_task));
                    }
            }
            
            if !open_blockers.is_empty() {
                eprintln!("Error: Cannot start task-{}. It is blocked by the following open tasks:", task_id);
                for (blocker_id, blocker_task) in &open_blockers {
                    eprintln!("  - task-{} ({}): {}", blocker_id, blocker_task.status, blocker_task.title);
                }
                eprintln!();
                eprintln!("Complete the blocking tasks first, or use 'knecht unblock' to remove the blocker.");
                std::process::exit(1);
            }
            
            println!("Starting work on task-{}: {}", task.id, task.title);
            if let Some(desc) = &task.description {
                println!();
                println!("Description:");
                println!("{}", desc);
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn cmd_pain(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: knecht pain <task-id>");
        std::process::exit(1);
    }
    
    let task_arg = &args[0];
    let task_id = parse_task_id(task_arg);
    
    match increment_pain_count_with_fs(task_id, &RealFileSystem) {
        Ok(task) => {
            println!("Incremented pain count for task-{}: {}", task.id, task.title);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn cmd_delete(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: knecht delete <task-id>");
        std::process::exit(1);
    }
    
    let task_arg = &args[0];
    let task_id = parse_task_id(task_arg);
    
    // Validate that task_id is numeric
    if task_id.parse::<u32>().is_err() {
        eprintln!("Error: Invalid task ID format");
        std::process::exit(1);
    }
    
    match delete_task_with_fs(task_id, &RealFileSystem) {
        Ok(task) => {
            println!("Deleted task-{}: {}", task.id, task.title);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn cmd_next() {
    match find_next_task_with_fs(&RealFileSystem) {
        Ok(Some(task)) => {
            println!("Suggested next task: task-{}", task.id);
            println!("Title: {}", task.title);
            if let Some(desc) = &task.description {
                println!("\nDescription:\n{}", desc);
            }
            if let Some(pain) = task.pain_count
                && pain > 0 {
                    println!("\n(pain count: {})", pain);
                }
        }
        Ok(None) => {
            println!("No open tasks");
        }
        Err(err) => {
            eprintln!("Error reading tasks: {}", err);
            std::process::exit(1);
        }
    }
}

fn cmd_update(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: knecht update <task-id> [--title <title>] [--description <description>]");
        eprintln!("       knecht update <task-id> [-t <title>] [-d <description>]");
        std::process::exit(1);
    }
    
    let task_arg = &args[0];
    let task_id = parse_task_id(task_arg);
    
    // Parse flags
    let mut new_title: Option<String> = None;
    let mut new_description: Option<Option<String>> = None;
    let mut i = 1;
    
    while i < args.len() {
        match args[i].as_str() {
            "--title" | "-t" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --title requires a value");
                    std::process::exit(1);
                }
                new_title = Some(args[i + 1].clone());
                i += 2;
            }
            "--description" | "-d" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --description requires a value");
                    std::process::exit(1);
                }
                let desc = args[i + 1].clone();
                if desc.is_empty() {
                    new_description = Some(None); // Clear description
                } else {
                    new_description = Some(Some(desc));
                }
                i += 2;
            }
            _ => {
                eprintln!("Error: Unknown flag '{}'", args[i]);
                eprintln!("Usage: knecht update <task-id> [--title <title>] [--description <description>]");
                std::process::exit(1);
            }
        }
    }
    
    // Check that at least one flag was provided
    if new_title.is_none() && new_description.is_none() {
        eprintln!("Error: Must provide at least one of --title or --description");
        eprintln!("Usage: knecht update <task-id> [--title <title>] [--description <description>]");
        std::process::exit(1);
    }
    
    match update_task_with_fs(task_id, new_title, new_description, &RealFileSystem) {
        Ok(task) => {
            println!("Updated task-{}", task.id);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn cmd_block(args: &[String]) {
    if args.len() < 3 || args[1] != "by" {
        eprintln!("Usage: knecht block <task-id> by <blocker-task-id>");
        std::process::exit(1);
    }
    
    let blocked_task_id = parse_task_id(&args[0]);
    let blocker_task_id = parse_task_id(&args[2]);
    
    // Verify both tasks exist
    if let Err(err) = find_task_by_id_with_fs(blocked_task_id, &RealFileSystem) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
    
    if let Err(err) = find_task_by_id_with_fs(blocker_task_id, &RealFileSystem) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
    
    // Add blocker relationship
    let blockers_path = ".knecht/blockers";
    let mut content = fs::read_to_string(blockers_path).unwrap_or_default();
    
    let blocker_line = format!("task-{}|task-{}\n", blocked_task_id, blocker_task_id);
    content.push_str(&blocker_line);
    
    if let Err(e) = fs::write(blockers_path, content) {
        eprintln!("Failed to write blockers file: {}", e);
        std::process::exit(1);
    }
    
    println!("Blocker added: task-{} is blocked by task-{}", blocked_task_id, blocker_task_id);
}

fn cmd_unblock(args: &[String]) {
    if args.len() < 3 || args[1] != "from" {
        eprintln!("Usage: knecht unblock <task-id> from <blocker-task-id>");
        std::process::exit(1);
    }
    
    let blocked_task_id = parse_task_id(&args[0]);
    let blocker_task_id = parse_task_id(&args[2]);
    
    // Read blockers file
    let blockers_path = ".knecht/blockers";
    let content = match fs::read_to_string(blockers_path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Error: task-{} is not blocked by task-{}", blocked_task_id, blocker_task_id);
            std::process::exit(1);
        }
    };
    
    let blocker_line = format!("task-{}|task-{}", blocked_task_id, blocker_task_id);
    
    // Check if the relationship exists
    if !content.contains(&blocker_line) {
        eprintln!("Error: task-{} is not blocked by task-{}", blocked_task_id, blocker_task_id);
        std::process::exit(1);
    }
    
    // Remove the blocker line
    let new_content: String = content
        .lines()
        .filter(|line| *line != blocker_line)
        .collect::<Vec<_>>()
        .join("\n");
    
    let new_content = if new_content.is_empty() {
        String::new()
    } else {
        format!("{}\n", new_content)
    };
    
    if let Err(e) = fs::write(blockers_path, new_content) {
        eprintln!("Failed to write blockers file: {}", e);
        std::process::exit(1);
    }
    
    println!("Blocker removed: task-{} is no longer blocked by task-{}", blocked_task_id, blocker_task_id);
}

/// Returns a list of task IDs that block the given task
fn get_blockers_for_task(task_id: &str) -> Vec<String> {
    let blockers_path = ".knecht/blockers";
    let content = match fs::read_to_string(blockers_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    
    let mut blockers = Vec::new();
    for line in content.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() == 2 {
            let blocked = parts[0].trim_start_matches("task-");
            let blocker = parts[1].trim_start_matches("task-");
            if blocked == task_id {
                blockers.push(blocker.to_string());
            }
        }
    }
    blockers
}

/// Returns a list of task IDs that are blocked by the given task
fn get_tasks_blocked_by(task_id: &str) -> Vec<String> {
    let blockers_path = ".knecht/blockers";
    let content = match fs::read_to_string(blockers_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    
    let mut blocked_tasks = Vec::new();
    for line in content.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() == 2 {
            let blocked = parts[0].trim_start_matches("task-");
            let blocker = parts[1].trim_start_matches("task-");
            if blocker == task_id {
                blocked_tasks.push(blocked.to_string());
            }
        }
    }
    blocked_tasks
}