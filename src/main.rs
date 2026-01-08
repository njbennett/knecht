use std::env;
use std::fs;

use knecht::{add_task_with_fs, find_task_by_id_with_fs, increment_pain_count_with_fs, mark_task_done_with_fs, read_tasks_with_fs, RealFileSystem};

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
        "show" => cmd_show(&args[2..]),
        "start" => cmd_start(&args[2..]),
        "pain" => cmd_pain(&args[2..]),
        "next" => cmd_next(),
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
}

fn cmd_done(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: knecht done <task-id>");
        std::process::exit(1);
    }
    
    let task_arg = &args[0];
    let task_id = task_arg.strip_prefix("task-").unwrap_or(task_arg);
    
    match mark_task_done_with_fs(task_id, &RealFileSystem) {
        Ok(task) => {
            println!("✓ task-{}: {}", task.id, task.title);
            println!();
            print!("REFLECTION PROMPT - Create tasks immediately for anything you notice:

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
    let task_id = task_arg.strip_prefix("task-").unwrap_or(task_arg);
    
    match find_task_by_id_with_fs(task_id, &RealFileSystem) {
        Ok(task) => {
            println!("Task: task-{}", task.id);
            println!("Status: {}", task.status);
            println!("Title: {}", task.title);
            if let Some(desc) = &task.description {
                println!("Description: {}", desc);
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
    let task_id = task_arg.strip_prefix("task-").unwrap_or(task_arg);
    
    match find_task_by_id_with_fs(task_id, &RealFileSystem) {
        Ok(task) => {
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
    let task_id = task_arg.strip_prefix("task-").unwrap_or(task_arg);
    
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

fn cmd_next() {
    match read_tasks_with_fs(&RealFileSystem) {
        Ok(tasks) => {
            // Filter to open tasks only
            let open_tasks: Vec<_> = tasks.iter()
                .filter(|t| t.status == "open")
                .collect();
            
            if open_tasks.is_empty() {
                println!("No open tasks");
                return;
            }
            
            // Find task with highest pain count, preferring older tasks on tie
            let best_task = open_tasks.iter()
                .max_by_key(|t| {
                    let pain = t.pain_count.unwrap_or(0);
                    let id_num: i32 = t.id.parse().unwrap_or(0);
                    (pain, -id_num)
                })
                .unwrap();
            
            println!("Suggested next task: task-{}", best_task.id);
            println!("Title: {}", best_task.title);
            if let Some(desc) = &best_task.description {
                println!("\nDescription:\n{}", desc);
            }
            if let Some(pain) = best_task.pain_count
                && pain > 0 {
                    println!("\n(pain count: {})", pain);
                }
        }
        Err(err) => {
            eprintln!("Error reading tasks: {}", err);
            std::process::exit(1);
        }
    }
}