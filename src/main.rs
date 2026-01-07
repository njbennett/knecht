use std::env;
use std::fs;

use knecht::{add_task_with_fs, mark_task_done_with_fs, read_tasks_with_fs, RealFileSystem};

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
        println!("{} task-{}  {}", checkbox, task.id, task.title);
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
            print!("Did you notice anything missing from knetch's interface during this work?

Did you notice anything the user had to correct the agent about, that could have been improved or avoided by making a change to knecht?

Did you notice anything new that was difficult about working with the codebase while you did this work? Is there anything in the work you just did that we should refactor? Make a list of the refactoring opportunities. Where you can, use named refactors from Martin Fowler's Refactoring, or Michael Feather's Working Effectively with Legacy Code. Check knecht to see if anything similar has already been filed, and if so, increase the pain count on those tasks.
");
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}