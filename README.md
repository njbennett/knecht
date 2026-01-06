# knecht

A git-native task tracker for giving agents very lightweight memory.

Named after Joseph Knecht from Hermann Hesse's *The Glass Bead Game* - "knecht" means "servant" in German.

This is an experiment in agent-driven coding,
and a place to practice vibe coding.

## Philosophy

**knecht** is built following these principles:

1. **Test-Driven**: Every feature starts with a failing test
2. **Self-Hosting**: We use knecht to build knecht
3. **Pain-Driven**: Features are added only when their absence hurts
4. **Simplest Possible**: Sequential IDs, pipe-delimited files, no complexity

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

# See what needs doing
knecht list

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

### `knecht done <task-id>`

Mark a task as complete.

```bash
knecht done task-1
# or
knecht done 1
```

Output: `✓ task-1: Fix the login bug`

## Data Format

Tasks are stored in `.knecht/tasks` using a simple pipe-delimited format:

```
1|open|Fix the login bug
2|done|Write tests for authentication
3|open|Deploy to staging
4|open|Refactor auth module|Break down into smaller functions and add better error handling
```

Format: `{id}|{status}|{title}` or `{id}|{status}|{title}|{description}`

The description field is optional - tasks without descriptions use the 3-field format for backwards compatibility.

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

### Why pipe-delimited instead of JSON?

Simplicity. Three fields don't need JSON's complexity. The format is readable, editable, and git-friendly.

### Why no dependencies/blockers/subtasks in v0.1?

Pain-driven development. We'll add these features when we actually feel the pain of not having them while using knecht.

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

Features will be added based on actual pain points. Possible future additions:

- Filter tasks by status: `knecht list --status open`
- Show task descriptions: `knecht show task-1`
- Blocked-by relationships: `knecht add "Deploy" --blocked-by task-3`
- Ready work detection: `knecht ready`
- JSON output for agents: `knecht list --json`
- Story/epic references

But we won't add these until we actually need them.

## License

MIT

## Inspiration

Inspired by:
- Steve Yegge's "bd" - the build daemon
- Kent Beck's TDD philosophy
- The UNIX philosophy of simple, composable tools
