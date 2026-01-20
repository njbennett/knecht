# knecht

A git-native task tracker designed primarily for AI agents to work in highly structured, incremental workflows.

Named after Joseph Knecht from Hermann Hesse's *The Glass Bead Game* - "knecht" means "servant" in German.

## Design Principles

1. **Agent-First**: Optimize for AI agents working autonomously - programmatic interfaces, structured data, git-native storage
2. **Test-Driven**: Every feature starts with a failing test
3. **Self-Hosting**: We use knecht to build knecht
4. **Pain-Driven**: Features are added only when their absence hurts (track pain counts at ~3-5 before implementing)
5. **Simplest Possible**: Sequential IDs, CSV files, no complexity

## Installation

```bash
cargo build --release
cp target/release/knecht /usr/local/bin/  # or anywhere in your PATH
```

## Quick Start

```bash
# Initialize knecht in your project
knecht init

# Add some tasks
knecht add "Fix the login bug"
knecht add "Write tests for authentication"
knecht add "Deploy to staging"

# Get suggestion for what to work on
knecht next

# Mark a task as done
knecht done task-1

# See the updated list
knecht list
```

## Commands

### `knecht init`

Initialize knecht in the current directory. Creates `.knecht/tasks`.

```bash
knecht init
```

### `knecht add <title> [-d <description>]`

Create a new task with the given title and optional description.

```bash
knecht add "Implement payment processing"
knecht add "Add tests for edge cases"
knecht add "Refactor auth module" -d "Break down into smaller functions and add better error handling"
```

Output: `Created task-1`

### `knecht list`

Show all tasks with their status.

```bash
knecht list
```

Output:
```
[ ] task-1  Fix the login bug
[x] task-2  Write tests for authentication
[ ] task-3  Deploy to staging
```

### `knecht next`

Get a suggestion for what to work on next. Analyzes open tasks and suggests the highest priority task based on pain count (tasks causing the most friction) and task age (older tasks first when pain is equal).

```bash
knecht next
```

Output:
```
Suggested next task: task-3
Title: Fix critical authentication bug

Description:
Users are unable to login after password reset. This is blocking production deployment.

(pain count: 5)
```

This is especially useful for agents and when managing many tasks - instead of scanning through all tasks manually, `knecht next` provides an objective recommendation.

### `knecht done <task-id>`

Mark a task as complete.

```bash
knecht done task-1
# or
knecht done 1
```

Output: `✓ task-1: Fix the login bug`

### `knecht show <task-id>`

Display full details for a specific task, including its description if present.

```bash
knecht show task-1
# or
knecht show 1
```

Output:
```
Task: task-1
Status: open
Title: Fix the login bug
Description: User sessions are expiring too early. Need to investigate token timeout settings.
```

For tasks without descriptions, only the ID, status, and title are shown.

### `knecht pain -t <task-id> -d <description>`

Increment the pain count for a task with a description of why. Use this when you encounter friction or difficulty - tracking pain helps prioritize work. Tasks with higher pain counts are suggested first by `knecht next`.

```bash
knecht pain -t task-1 -d "Had to work around this manually again"
# or
knecht pain -t 1 -d "Third time hitting this limitation"
```

Output: `Incremented pain count for task-1: Fix the login bug`

The pain count appears in `knecht list` output and is used by `knecht next` to prioritize which tasks to work on. This implements pain-driven development: track what hurts, and fix the things that hurt most.

### `knecht deliver <task-id>`

Mark a task as delivered (ready for verification). This is an intermediate state between `open` and `done` - useful when work is complete but needs review or testing.

```bash
knecht deliver task-1
```

### `knecht delete <task-id>`

Remove a task entirely.

```bash
knecht delete task-1
```

### `knecht start <task-id>`

Begin work on a task (prints the task details).

```bash
knecht start task-1
```

### `knecht update <task-id> [-t <title>] [-d <description>]`

Update a task's title and/or description.

```bash
knecht update task-1 -t "New title"
knecht update task-1 -d "New description"
knecht update task-1 -t "New title" -d "And new description"
```

### `knecht block <task-id> <blocker-id>`

Mark a task as blocked by another task. Blocked tasks won't be suggested by `knecht next` until their blocker is resolved.

```bash
knecht block task-2 task-1  # task-2 is blocked by task-1
```

### `knecht unblock <task-id> <blocker-id>`

Remove a blocker from a task.

```bash
knecht unblock task-2 task-1
```

## Data Format

Tasks are stored in `.knecht/tasks` using standard CSV format:

```
1,open,"Fix the login bug",,
2,done,"Write tests for authentication",,
3,open,"Deploy to staging",,
4,open,"Refactor auth module","Break down into smaller functions and add better error handling",
```

Format: `{id},{status},"{title}","{description}",{pain_count}`

All fields are always present. Empty fields (description, pain_count) are included as empty values for consistency.

This format is:
- **Git-friendly**: Line-based diffs work perfectly
- **Human-readable**: You can edit it with any text editor
- **Simple**: No parsing complexity, no dependencies
- **Fast**: Parsing 1000 tasks takes <1ms

## Git Integration

Commit your `.knecht/tasks` file along with your code:

```bash
git add .knecht/tasks
git commit -m "Update task status"
```

## Parallel Sessions with `cw` (Claude Worktree)

The `cw` script enables parallel Claude agent sessions using git worktrees. Each session works in an isolated branch that gets merged back to main when the session ends.

### Installation

```bash
# Option 1: Run directly from the scripts directory
./scripts/cw

# Option 2: Symlink to your PATH
ln -s $(pwd)/scripts/cw /usr/local/bin/cw

# Option 3: Copy to your dotfiles
cp scripts/cw ~/.local/bin/cw
```

### Usage

```bash
cw [branch-name]   # If no branch-name, generates work-YYYYMMDD-HHMMSS
```

### What it does

**On start:**
1. Detects main branch (main/master)
2. Verifies main worktree is clean
3. Creates worktree: `../reponame-branchname`
4. Runs `claude` (or `$CW_COMMAND`) in the worktree

**On claude exit:**
1. Checks for uncommitted changes (warns and preserves worktree if dirty)
2. If commits exist: fetches, rebases onto main, fast-forward merges
3. Removes worktree and branch

### Edge cases

| Scenario | Behavior |
|----------|----------|
| Uncommitted changes | Warns, leaves worktree intact, shows manual cleanup commands |
| Rebase conflicts | Aborts rebase, warns, leaves worktree for manual resolution |
| No commits made | Cleans up worktree without merge |
| Ctrl+C during session | Traps signal, runs cleanup logic |

### Configuration

- `CW_COMMAND` - Override the command to run (default: `claude`)

```bash
CW_COMMAND="claude --model opus" cw my-feature
```

## Examples

### Track a feature from start to finish

```bash
knecht init
knecht add "Research payment APIs"
knecht add "Add Stripe integration"
knecht add "Add payment UI"
knecht add "Write tests"

knecht list
# Work on tasks...

knecht done task-1
knecht list  # See remaining work
```

### Using knecht to build knecht

This project uses itself! Check `.knecht/tasks` to see what's being worked on.

```bash
knecht list
```

## Design Decisions

### Why sequential IDs instead of hashes?

YAGNI (You Ain't Gonna Need It). Sequential integers work fine for a personal task tracker. We can add content-based hashing later if merge conflicts become painful.

### Why CSV instead of JSON?

Simplicity and standards. CSV is a well-understood format with robust parsing libraries. It's readable, editable, git-friendly, and handles special characters (commas, pipes, quotes) correctly out of the box.

### How do blockers work?

Blockers were added when we felt the pain of needing them. Use `knecht block` to mark dependencies between tasks. `knecht next` won't suggest blocked tasks until their blockers are resolved. Subtasks aren't a separate feature - just use blockers to express dependencies.

### Why Rust?

Fast, reliable, single binary, cross-platform. But the design is language-agnostic - you could implement this in Python or Go in an afternoon.

## Development

### Running tests

```bash
cargo test
```

### Building

```bash
cargo build --release
```

### Contributing

1. Create or pick up a knecht task
2. Write a failing test
3. Make it pass with the simplest code
4. Submit a PR

## Roadmap

Features are added based on actual pain points. Possible future additions:

- Filter tasks by status: `knecht list --status open`
- JSON output for agents: `knecht list --json`
- Story/epic references

We won't add these until we actually need them.

## License

MIT

## Migration from Beads

If you're migrating from the `beads` task tracker, knecht includes a migration tool that converts beads JSON format to knecht format.

### beads2knecht Tool

The `beads2knecht` binary reads beads JSON from stdin and outputs knecht format to stdout.

#### Building the Tool

```bash
cargo build --release
# The binary is at target/release/beads2knecht
```

#### Usage

```bash
# Preview the migration (see what will be converted)
bd list --json | /path/to/knecht/target/release/beads2knecht > migration_preview.txt
cat migration_preview.txt

# If acceptable, migrate to knecht
knecht init
bd list --json | /path/to/knecht/target/release/beads2knecht | grep -v '^#' > .knecht/tasks

# Verify the migration
knecht list
```

#### What Gets Migrated

**Preserved:**
- Task titles
- Task status (open/in_progress/done → open/done)
- Task descriptions (if present)

**Converted:**
- Beads IDs (alphanumeric) → Sequential numbers (1, 2, 3...)
- Status `in_progress` → `open` (knecht has open/delivered/done)

**Dropped (Intentionally):**
- Priorities (0-4) - Can be expressed in task titles if needed
- Issue types (bug/task/epic/feature/chore) - Can be expressed in titles
- Timestamps - Git provides history
- Dependencies - Different model planned for future knecht versions

The tool outputs migration statistics to stderr, including:
- Number of tasks converted
- Number of descriptions preserved
- Distribution of priorities (for reference)
- Distribution of issue types (for reference)

#### Example Output

```bash
$ bd list --json | beads2knecht
# Beads to Knecht Migration
# 3 tasks found
#
# MIGRATION STRATEGY:
# - Map beads IDs to sequential numbers (1, 2, 3...)
# - Map 'in_progress' -> 'open'
# - PRESERVE: descriptions (in CSV description field)
# - DROP: priorities, issue_types, timestamps, dependencies
# - Keep: id, status, title, description
#
1,open,Fix authentication bug,,
2,done,Add user registration,"Implement user registration with email verification",
3,open,Refactor database layer,,

=== MIGRATION COMPLETE ===
Tasks converted: 3

PRESERVED INFORMATION:
- Descriptions: 1 tasks had descriptions (preserved)

LOST INFORMATION:
- Priorities: Distribution:
  Priority 0: 1 tasks
  Priority 1: 2 tasks
- Issue types:
  bug: 1 tasks
  task: 2 tasks
```

Note: The migration tool may need to be rebuilt to output current CSV format.

#### Tips for Migration

1. **Preview first**: Always run a preview to see what information will be lost
2. **Review priorities**: If high-priority tasks exist, consider adding "High priority:" prefix to titles
3. **Review issue types**: If issue types matter, add them to titles (e.g., "Bug: Fix auth")
4. **Backup**: Keep your beads data until you're confident in the migration
5. **Commit**: Commit the new `.knecht/tasks` file to git immediately after migration

## Inspiration

Inspired by:
- Steve Yegge's "bd" - the build daemon
- Kent Beck's TDD philosophy
- The UNIX philosophy of simple, composable tools
