use clap::Parser;
use knecht::{
    add_task_with_fs, append_pain_entry_with_fs, FileSystem, PainEntry, PainSourceType,
    RealFileSystem,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "sentry2knecht")]
#[command(about = "Import Sentry issues as knecht tasks")]
struct Cli {
    /// Sentry organization slug
    #[arg(short, long)]
    org: String,

    /// Sentry project slug
    #[arg(short, long)]
    project: String,

    /// Sentry auth token (overrides SENTRY_AUTH_TOKEN env var)
    #[arg(long, env = "SENTRY_AUTH_TOKEN")]
    token: String,

    /// Sentry API base URL
    #[arg(long, default_value = "https://sentry.io")]
    base_url: String,

    /// Only sync issues with this status (unresolved, resolved, ignored)
    #[arg(long, default_value = "unresolved")]
    status: String,

    /// Dry run - show what would be created without creating
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Deserialize)]
struct SentryIssue {
    id: String,
    #[serde(rename = "shortId")]
    short_id: String,
    title: String,
    count: String, // Sentry returns as string
    #[allow(dead_code)]
    status: String,
    permalink: String,
    #[serde(rename = "firstSeen")]
    first_seen: String,
    #[serde(rename = "lastSeen")]
    last_seen: String,
}

#[derive(Debug, Clone)]
struct SentryMapping {
    sentry_issue_id: String,
    knecht_task_id: String,
    last_sync_timestamp: u64,
    last_event_count: u64,
}

#[derive(Debug)]
enum SyncResult {
    Created { task_id: String, pain_count: u64 },
    Updated { task_id: String, new_pain: u64 },
    Skipped { task_id: String },
}

fn main() {
    let cli = Cli::parse();

    // Verify .knecht directory exists
    if !Path::new(".knecht").exists() {
        eprintln!("Error: .knecht directory not found. Run 'knecht init' first.");
        std::process::exit(1);
    }

    let fs = RealFileSystem;

    // Fetch issues from Sentry
    eprintln!("Fetching issues from Sentry...");
    let issues = match fetch_sentry_issues(&cli) {
        Ok(issues) => issues,
        Err(e) => {
            eprintln!("Error fetching Sentry issues: {}", e);
            std::process::exit(1);
        }
    };
    eprintln!("Found {} issues", issues.len());

    // Read existing mappings
    let mappings = read_sentry_mappings(&fs).unwrap_or_default();
    eprintln!("Loaded {} existing mappings", mappings.len());

    // Sync each issue
    let mut created = 0;
    let mut updated = 0;
    let mut skipped = 0;
    let mut total_pain = 0u64;

    for issue in &issues {
        let existing = mappings.get(&issue.id);

        if cli.dry_run {
            let event_count: u64 = issue.count.parse().unwrap_or(0);
            if let Some(mapping) = existing {
                let delta = event_count.saturating_sub(mapping.last_event_count);
                if delta > 0 {
                    println!(
                        "[DRY RUN] Would update task-{}: +{} pain ({})",
                        mapping.knecht_task_id, delta, issue.title
                    );
                    updated += 1;
                    total_pain += delta;
                } else {
                    println!(
                        "[DRY RUN] Would skip task-{}: no new events",
                        mapping.knecht_task_id
                    );
                    skipped += 1;
                }
            } else {
                println!(
                    "[DRY RUN] Would create: [SENTRY-{}] {} ({} pain)",
                    issue.short_id, issue.title, event_count
                );
                created += 1;
                total_pain += event_count;
            }
        } else {
            match sync_single_issue(issue, existing, &fs) {
                Ok(SyncResult::Created { task_id, pain_count }) => {
                    println!(
                        "Created task-{}: [SENTRY-{}] {} ({} pain)",
                        task_id, issue.short_id, issue.title, pain_count
                    );
                    created += 1;
                    total_pain += pain_count;
                }
                Ok(SyncResult::Updated { task_id, new_pain }) => {
                    println!(
                        "Updated task-{}: +{} pain ({})",
                        task_id, new_pain, issue.title
                    );
                    updated += 1;
                    total_pain += new_pain;
                }
                Ok(SyncResult::Skipped { task_id }) => {
                    skipped += 1;
                    eprintln!("Skipped task-{}: no new events", task_id);
                }
                Err(e) => {
                    eprintln!("Error syncing issue {}: {}", issue.short_id, e);
                }
            }
        }
    }

    // Print summary
    println!();
    println!("=== Sync Summary ===");
    println!("Created: {} new tasks", created);
    println!("Updated: {} existing tasks", updated);
    println!("Skipped: {} tasks (no new events)", skipped);
    println!("Total pain entries: {}", total_pain);
}

fn fetch_sentry_issues(cli: &Cli) -> Result<Vec<SentryIssue>, String> {
    let url = format!(
        "{}/api/0/projects/{}/{}/issues/?query=is:{}",
        cli.base_url, cli.org, cli.project, cli.status
    );

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", cli.token))
        .send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Sentry API returned status {}: {}",
            response.status(),
            response.text().unwrap_or_default()
        ));
    }

    response
        .json::<Vec<SentryIssue>>()
        .map_err(|e| format!("Failed to parse response: {}", e))
}

fn read_sentry_mappings(fs: &dyn FileSystem) -> Result<HashMap<String, SentryMapping>, String> {
    let path = Path::new(".knecht/sentry-mapping");

    if !fs.exists(path) {
        return Ok(HashMap::new());
    }

    let reader = fs.open(path).map_err(|e| format!("Failed to open mapping file: {}", e))?;
    let mut mappings = HashMap::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            let mapping = SentryMapping {
                sentry_issue_id: parts[0].to_string(),
                knecht_task_id: parts[1].to_string(),
                last_sync_timestamp: parts[2].parse().unwrap_or(0),
                last_event_count: parts[3].parse().unwrap_or(0),
            };
            mappings.insert(mapping.sentry_issue_id.clone(), mapping);
        }
    }

    Ok(mappings)
}

fn append_sentry_mapping(mapping: &SentryMapping, fs: &dyn FileSystem) -> Result<(), String> {
    let path = Path::new(".knecht/sentry-mapping");

    let mut writer = fs
        .append(path)
        .map_err(|e| format!("Failed to open mapping file: {}", e))?;

    writeln!(
        writer,
        "{}|{}|{}|{}",
        mapping.sentry_issue_id,
        mapping.knecht_task_id,
        mapping.last_sync_timestamp,
        mapping.last_event_count
    )
    .map_err(|e| format!("Failed to write mapping: {}", e))?;

    Ok(())
}

fn sync_single_issue(
    issue: &SentryIssue,
    existing: Option<&SentryMapping>,
    fs: &dyn FileSystem,
) -> Result<SyncResult, String> {
    let event_count: u64 = issue.count.parse().unwrap_or(0);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if let Some(mapping) = existing {
        // Existing task - check for new events
        let delta = event_count.saturating_sub(mapping.last_event_count);

        if delta == 0 {
            return Ok(SyncResult::Skipped {
                task_id: mapping.knecht_task_id.clone(),
            });
        }

        // Add pain entries for new events
        add_sentry_pain_entries(&mapping.knecht_task_id, issue, delta, fs)?;

        // Update mapping with new count
        let new_mapping = SentryMapping {
            sentry_issue_id: issue.id.clone(),
            knecht_task_id: mapping.knecht_task_id.clone(),
            last_sync_timestamp: now,
            last_event_count: event_count,
        };
        append_sentry_mapping(&new_mapping, fs)?;

        Ok(SyncResult::Updated {
            task_id: mapping.knecht_task_id.clone(),
            new_pain: delta,
        })
    } else {
        // New issue - create task
        let title = format!("[SENTRY-{}] {}", issue.short_id, issue.title);
        let description = format!(
            "Sentry issue: {}\nFirst seen: {}\nLast seen: {}",
            issue.permalink, issue.first_seen, issue.last_seen
        );

        let task_id = add_task_with_fs(title, Some(description), None, fs)
            .map_err(|e| format!("Failed to create task: {}", e))?;

        // Add pain entries for all events
        add_sentry_pain_entries(&task_id, issue, event_count, fs)?;

        // Record mapping
        let mapping = SentryMapping {
            sentry_issue_id: issue.id.clone(),
            knecht_task_id: task_id.clone(),
            last_sync_timestamp: now,
            last_event_count: event_count,
        };
        append_sentry_mapping(&mapping, fs)?;

        Ok(SyncResult::Created {
            task_id,
            pain_count: event_count,
        })
    }
}

fn add_sentry_pain_entries(
    task_id: &str,
    issue: &SentryIssue,
    count: u64,
    fs: &dyn FileSystem,
) -> Result<(), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for _ in 0..count {
        let entry = PainEntry {
            task_id: task_id.to_string(),
            timestamp: now,
            source_type: PainSourceType::Sentry,
            source_id: Some(issue.short_id.clone()),
            description: format!("Sentry event: {}", issue.title),
        };
        append_pain_entry_with_fs(&entry, fs)
            .map_err(|e| format!("Failed to add pain entry: {}", e))?;
    }

    Ok(())
}
