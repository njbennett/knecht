use std::fs;

use clap::{Parser, Subcommand};
use knecht::{add_task_with_fs, delete_task_with_fs, find_next_task_with_fs, find_task_by_id_with_fs, get_all_pain_counts, get_pain_count_for_task, get_pain_entries_for_task, increment_pain_count_with_fs, mark_task_claimed_with_fs, mark_task_delivered_with_fs, mark_task_done_with_fs, read_tasks_with_fs, update_task_with_fs, RealFileSystem};

#[derive(Parser)]
#[command(name = "knecht")]
#[command(about = "A git-native task tracker for AI agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new knecht repository
    Init,
    /// Add a new task
    Add {
        /// Task title (can be multiple words)
        #[arg(required = true, num_args = 1..)]
        title: Vec<String>,
        /// Task description
        #[arg(short, long = "description")]
        d: Option<String>,
        /// Acceptance criteria
        #[arg(short, long = "acceptance-criteria")]
        a: Option<String>,
    },
    /// List tasks (open tasks by default)
    List {
        /// Show all tasks including done/delivered
        #[arg(long)]
        all: bool,
    },
    /// Mark a task as done
    Done {
        /// Task ID (e.g., task-1 or 1)
        task_id: String,
    },
    /// Mark a task as delivered
    Deliver {
        /// Task ID (e.g., task-1 or 1)
        task_id: String,
    },
    /// Delete a task
    Delete {
        /// Task ID (e.g., task-1 or 1)
        task_id: String,
    },
    /// Show details of a task
    Show {
        /// Task ID (e.g., task-1 or 1)
        task_id: String,
    },
    /// Start working on a task
    Start {
        /// Task ID (e.g., task-1 or 1)
        task_id: String,
    },
    /// Increment pain count for a task
    Pain {
        /// Task ID (e.g., task-1 or 1)
        #[arg(short = 't', required = true)]
        task_id: String,
        /// Description of the pain instance
        #[arg(short, required = true)]
        d: String,
    },
    /// Get the next suggested task to work on
    Next,
    /// Update a task's title or description
    Update {
        /// Task ID (e.g., task-1 or 1)
        task_id: String,
        /// New title
        #[arg(short = 't', long = "title")]
        title: Option<String>,
        /// New description
        #[arg(short, long = "description")]
        d: Option<String>,
        /// Acceptance criteria
        #[arg(short, long = "acceptance-criteria")]
        a: Option<String>,
    },
    /// Mark a task as blocked by another task
    Block {
        /// Task ID to block (e.g., task-1 or 1)
        task_id: String,
        /// Must be "by"
        #[arg(value_parser = clap::builder::PossibleValuesParser::new(["by"]))]
        by: String,
        /// Blocker task ID (e.g., task-2 or 2)
        blocker_id: String,
    },
    /// Remove a blocker from a task
    Unblock {
        /// Task ID to unblock (e.g., task-1 or 1)
        task_id: String,
        /// Must be "from"
        #[arg(value_parser = clap::builder::PossibleValuesParser::new(["from"]))]
        from: String,
        /// Blocker task ID to remove (e.g., task-2 or 2)
        blocker_id: String,
    },
}

/// Parses a task ID argument, stripping the "task-" prefix if present.
/// Accepts both "task-N" and "N" formats, returning just the numeric ID part.
fn parse_task_id(task_arg: &str) -> &str {
    task_arg.strip_prefix("task-").unwrap_or(task_arg)
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cmd_init(),
        Commands::Add { title, d, a } => cmd_add(&title.join(" "), d, a),
        Commands::List { all } => cmd_list(all),
        Commands::Done { task_id } => cmd_done(&task_id),
        Commands::Deliver { task_id } => cmd_deliver(&task_id),
        Commands::Delete { task_id } => cmd_delete(&task_id),
        Commands::Show { task_id } => cmd_show(&task_id),
        Commands::Start { task_id } => cmd_start(&task_id),
        Commands::Pain { task_id, d } => cmd_pain(&task_id, &d),
        Commands::Next => cmd_next(),
        Commands::Update { task_id, title, d, a } => cmd_update(&task_id, title, d, a),
        Commands::Block { task_id, by: _, blocker_id } => cmd_block(&task_id, &blocker_id),
        Commands::Unblock { task_id, from: _, blocker_id } => cmd_unblock(&task_id, &blocker_id),
    }
}

fn cmd_init() {
    if let Err(e) = fs::create_dir_all(".knecht/tasks") {
        eprintln!("Failed to create .knecht/tasks directory: {}", e);
        std::process::exit(1);
    }

    println!("Initialized knecht");
}

fn cmd_add(title: &str, description: Option<String>, acceptance_criteria: Option<String>) {
    if title.is_empty() {
        eprintln!("Error: Title cannot be empty");
        std::process::exit(1);
    }

    if acceptance_criteria.is_none() {
        eprintln!("Error: Acceptance criteria is required. Use -a to specify criteria.");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  knecht add \"Task title\" -a \"Criteria that defines done\"");
        std::process::exit(1);
    }

    match add_task_with_fs(title.to_string(), description, acceptance_criteria, &RealFileSystem) {
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

fn cmd_list(show_all: bool) {
    let tasks = match read_tasks_with_fs(&RealFileSystem) {
        Ok(tasks) => tasks,
        Err(e) => {
            eprintln!("Error reading tasks: {}", e);
            std::process::exit(1);
        }
    };

    // Filter to open tasks unless --all flag is provided
    let filtered_tasks: Vec<_> = if show_all {
        tasks
    } else {
        tasks.into_iter().filter(|t| !t.is_done() && t.status != "delivered").collect()
    };

    // Get all pain counts from the pain log (efficient bulk read)
    let pain_counts = get_all_pain_counts(&RealFileSystem).unwrap_or_default();

    for task in &filtered_tasks {
        let checkbox = if task.is_done() {
            "[x]"
        } else if task.status == "delivered" {
            "[>]"
        } else if task.status == "claimed" {
            "[~]"
        } else {
            "[ ]"
        };
        let pain_count = pain_counts.get(&task.id).copied().unwrap_or(0);
        let pain_suffix = if pain_count > 0 {
            format!(" (pain count: {})", pain_count)
        } else {
            String::new()
        };
        println!("{} task-{}  {}{}", checkbox, task.id, task.title, pain_suffix);
    }

    // Print usage instructions for agents
    println!();
    if !show_all {
        println!("Showing open tasks only. Use --all to see all tasks.");
        println!();
    }
    println!("Usage instructions:");
    println!("  knecht show task-N     - View full task details including description");
    println!("  knecht start task-N    - Begin work on a task");
    println!("  knecht done task-N     - Mark a task as complete");
    println!("  knecht next            - Get suggestion for what to work on next");
}

fn cmd_deliver(task_arg: &str) {
    let task_id = parse_task_id(task_arg);

    match mark_task_delivered_with_fs(task_id, &RealFileSystem) {
        Ok(task) => {
            println!("✓ task-{}: {}", task.id, task.title);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_done(task_arg: &str) {
    let task_id = parse_task_id(task_arg);

    match mark_task_done_with_fs(task_id, &RealFileSystem) {
        Ok(task) => {
            println!("✓ task-{}: {}", task.id, task.title);
            print!("
================================================================================
REFLECTION REQUIRED
================================================================================

Run: /reflect

This loads the reflection skill which will guide you through required questions
about this work session. You MUST complete reflection before continuing.

================================================================================
");
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn cmd_show(task_arg: &str) {
    let task_id = parse_task_id(task_arg);

    match find_task_by_id_with_fs(task_id, &RealFileSystem) {
        Ok(task) => {
            println!("Task: task-{}", task.id);
            println!("Status: {}", task.status);
            println!("Title: {}", task.title);
            if let Some(desc) = &task.description {
                println!("Description: {}", desc);
            }
            if let Some(criteria) = &task.acceptance_criteria {
                println!("Acceptance Criteria:\n{}", criteria);
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

            // Display pain history from pain log
            if let Ok(pain_entries) = get_pain_entries_for_task(task_id, &RealFileSystem) {
                if !pain_entries.is_empty() {
                    println!("Pain ({} instance{}):", pain_entries.len(), if pain_entries.len() == 1 { "" } else { "s" });
                    for entry in &pain_entries {
                        println!("  {}", entry.description);
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

fn cmd_start(task_arg: &str) {
    let task_id = parse_task_id(task_arg);

    match find_task_by_id_with_fs(task_id, &RealFileSystem) {
        Ok(_task) => {
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

            // Claim the task by changing status to "claimed"
            match mark_task_claimed_with_fs(task_id, &RealFileSystem) {
                Ok(claimed_task) => {
                    println!("Starting work on task-{}: {}", claimed_task.id, claimed_task.title);
                    if let Some(desc) = &claimed_task.description {
                        println!();
                        println!("Description:");
                        println!("{}", desc);
                    }
                }
                Err(err) => {
                    eprintln!("Error claiming task: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn cmd_pain(task_arg: &str, description: &str) {
    let task_id = parse_task_id(task_arg);

    match increment_pain_count_with_fs(task_id, Some(description), &RealFileSystem) {
        Ok(task) => {
            println!("Incremented pain count for task-{}: {}", task.id, task.title);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn cmd_delete(task_arg: &str) {
    let task_id = parse_task_id(task_arg);

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
            let pain_count = get_pain_count_for_task(&task.id, &RealFileSystem).unwrap_or(0);
            if pain_count > 0 {
                println!("\n(pain count: {})", pain_count);
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

fn cmd_update(task_arg: &str, new_title: Option<String>, new_description: Option<String>, new_acceptance_criteria: Option<String>) {
    let task_id = parse_task_id(task_arg);

    // Check that at least one flag was provided
    if new_title.is_none() && new_description.is_none() && new_acceptance_criteria.is_none() {
        eprintln!("Error: Must provide at least one of --title, --description, or --acceptance-criteria");
        eprintln!("Usage: knecht update <task-id> [--title <title>] [--description <description>] [--acceptance-criteria <criteria>]");
        std::process::exit(1);
    }

    // Convert Option<String> to Option<Option<String>> for description
    let desc_update = new_description.map(|d| {
        if d.is_empty() {
            None // Clear description
        } else {
            Some(d)
        }
    });

    // Convert Option<String> to Option<Option<String>> for acceptance_criteria
    let criteria_update = new_acceptance_criteria.map(|c| {
        if c.is_empty() {
            None // Clear acceptance criteria
        } else {
            Some(c)
        }
    });

    match update_task_with_fs(task_id, new_title, desc_update, criteria_update, &RealFileSystem) {
        Ok(task) => {
            println!("Updated task-{}", task.id);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn cmd_block(blocked_task_arg: &str, blocker_task_arg: &str) {
    let blocked_task_id = parse_task_id(blocked_task_arg);
    let blocker_task_id = parse_task_id(blocker_task_arg);

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

fn cmd_unblock(blocked_task_arg: &str, blocker_task_arg: &str) {
    let blocked_task_id = parse_task_id(blocked_task_arg);
    let blocker_task_id = parse_task_id(blocker_task_arg);

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
