use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

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
    let next_id = get_next_id();
    let line = format!("{}|open|{}\n", next_id, title);
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(".knecht/tasks")
        .expect("Failed to open tasks file");
    
    file.write_all(line.as_bytes())
        .expect("Failed to write task");
    
    println!("Created task-{}", next_id);
}

fn cmd_list() {
    let tasks = read_tasks();
    
    for task in tasks {
        let checkbox = if task.status == "done" { "[x]" } else { "[ ]" };
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
    
    let mut tasks = read_tasks();
    let mut found = false;
    
    for task in &mut tasks {
        if task.id == task_id {
            task.status = "done".to_string();
            found = true;
            println!("✓ task-{}: {}", task.id, task.title);
            break;
        }
    }
    
    if !found {
        eprintln!("Error: task-{} not found", task_id);
        std::process::exit(1);
    }
    
    write_tasks(&tasks);
}

struct Task {
    id: String,
    status: String,
    title: String,
}

fn read_tasks() -> Vec<Task> {
    let path = Path::new(".knecht/tasks");
    
    if !path.exists() {
        return Vec::new();
    }
    
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    
    let reader = BufReader::new(file);
    let mut tasks = Vec::new();
    
    for line in reader.lines() {
        if let Ok(line) = line {
            if line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                tasks.push(Task {
                    id: parts[0].to_string(),
                    status: parts[1].to_string(),
                    title: parts[2].to_string(),
                });
            }
        }
    }
    
    tasks
}

fn write_tasks(tasks: &[Task]) {
    let mut file = fs::File::create(".knecht/tasks")
        .expect("Failed to open tasks file for writing");
    
    for task in tasks {
        let line = format!("{}|{}|{}\n", task.id, task.status, task.title);
        file.write_all(line.as_bytes())
            .expect("Failed to write task");
    }
}

fn get_next_id() -> u32 {
    let tasks = read_tasks();
    
    let max_id = tasks
        .iter()
        .filter_map(|t| t.id.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    
    max_id + 1
}