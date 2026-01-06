use std::env;
use std::fs;

mod task;

use task::{add_task, mark_task_done, read_tasks};

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
        eprintln!("Usage: knecht add <title>");
        std::process::exit(1);
    }
    
    let title = args.join(" ");
    let task_id = add_task(title);
    
    println!("Created task-{}", task_id);
}

fn cmd_list() {
    let tasks = read_tasks();
    
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
    
    match mark_task_done(task_id) {
        Ok(task) => {
            println!("✓ task-{}: {}", task.id, task.title);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}