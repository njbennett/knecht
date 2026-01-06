use serde::Deserialize;
use std::io::{self, Read};

#[derive(Debug, Deserialize)]
struct BeadsTask {
    #[allow(dead_code)]
    id: String,
    title: String,
    #[serde(default)]
    description: Option<String>,
    status: String,
    priority: u8,
    issue_type: String,
}

fn main() {
    // Read JSON from stdin
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Failed to read from stdin");

    // Parse JSON
    let beads_tasks: Vec<BeadsTask> = serde_json::from_str(&buffer)
        .expect("Failed to parse JSON");

    // Convert to knecht format
    println!("# Beads to Knecht Migration");
    println!("# {} tasks found", beads_tasks.len());
    println!("#");
    println!("# MIGRATION STRATEGY:");
    println!("# - Map beads IDs to sequential numbers (1, 2, 3...)");
    println!("# - Map 'in_progress' -> 'open'");
    println!("# - PRESERVE: descriptions (in 4th pipe-delimited field)");
    println!("# - DROP: priorities, issue_types, timestamps, dependencies");
    println!("# - Keep: id, status, title, description");
    println!("#");
    
    // Generate knecht tasks file content
    for (index, task) in beads_tasks.iter().enumerate() {
        let knecht_id = index + 1;
        let knecht_status = match task.status.as_str() {
            "done" => "done",
            "in_progress" => "open",
            "open" => "open",
            _ => "open",
        };
        
        // knecht format: {id}|{status}|{title}|{description} (description optional)
        if let Some(ref desc) = task.description {
            println!("{}|{}|{}|{}", knecht_id, knecht_status, task.title, desc);
        } else {
            println!("{}|{}|{}", knecht_id, knecht_status, task.title);
        }
    }

    eprintln!("\n=== MIGRATION COMPLETE ===");
    eprintln!("Tasks converted: {}", beads_tasks.len());
    eprintln!("\nPRESERVED INFORMATION:");
    eprintln!("- Descriptions: {} tasks had descriptions (preserved)", 
        beads_tasks.iter().filter(|t| t.description.is_some()).count());
    eprintln!("\nLOST INFORMATION:");
    eprintln!("- Priorities: Distribution:");
    for p in 0..=4 {
        let count = beads_tasks.iter().filter(|t| t.priority == p).count();
        if count > 0 {
            eprintln!("  Priority {}: {} tasks", p, count);
        }
    }
    eprintln!("- Issue types:");
    let mut types: Vec<String> = beads_tasks.iter()
        .map(|t| t.issue_type.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    types.sort();
    for issue_type in types {
        let count = beads_tasks.iter().filter(|t| t.issue_type == issue_type).count();
        eprintln!("  {}: {} tasks", issue_type, count);
    }
}