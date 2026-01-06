# knecht - Test-Driven Design Specification

## Vision

A git-native task tracker so simple you can build and use it in one afternoon.

Named after Joseph Knecht from Hermann Hesse's *The Glass Bead Game* - "servant" in German.

## Core Principle: Test-First, Self-Hosting from Line 1

We will:
1. Write a failing test
2. Make it pass with the simplest code
3. Use knecht to track building knecht
4. Add the next feature only when we feel pain

## The First Test (Start Here)

Before writing any application code, write this test:

```rust
// tests/integration_test.rs
use std::fs;
use std::path::PathBuf;

#[test]
fn can_create_and_list_a_task() {
    let temp = setup_temp_dir();
    
    // Initialize
    let init_result = run_command(&["init"], &temp);
    assert!(init_result.success);
    assert!(temp.join(".knecht/tasks").exists());
    
    // Add a task
    let add_result = run_command(&["add", "Write first test"], &temp);
    assert!(add_result.success);
    assert!(add_result.stdout.contains("task-1"));
    
    // List tasks
    let list_result = run_command(&["list"], &temp);
    assert!(list_result.success);
    assert!(list_result.stdout.contains("task-1"));
    assert!(list_result.stdout.contains("Write first test"));
    assert!(list_result.stdout.contains("[ ]")); // open checkbox
    
    cleanup_temp_dir(temp);
}
```

This test defines the entire first iteration. Make it pass.

## V0.1: The Four Essential Commands

### `knecht init`
Creates `.knecht/tasks` file. That's it.

### `knecht add <title>`
Appends a line to `.knecht/tasks`:
```
1|open|Write first test
```

Returns: "Created task-1"

### `knecht list`
Reads `.knecht/tasks` and displays:
```
[ ] task-1  Write first test
[ ] task-2  Make it pass
[x] task-3  Commit the code
```

### `knecht done <id>`
Changes the status field from `open` to `done`:
```
3|done|Commit the code
```

Returns: "✓ task-3: Commit the code"

## Data Format: Simplest Possible

File: `.knecht/tasks`

```
1|open|Write first test
2|open|Make it pass
3|done|Commit the code
```

Format: `{id}|{status}|{title}`

Why pipe-delimited?
- Simple to parse (split on `|`)
- Human-readable
- Git-friendly (line-based diffs)
- No escaping needed for most titles

Why not JSONL?
- YAGNI. We have 3 fields. Pipes work fine.
- Add JSON later if we need complex nested data

Why not a database?
- YAGNI. 1000 lines parse in <1ms.
- Keep it git-native.

## TDD Development Process

### Step 1: Write the First Failing Test ✍️
Copy the test above. Run `cargo test`. Watch it fail (no binary exists).

### Step 2: Make `init` Pass
```rust
// src/main.rs
fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("init") => {
            std::fs::create_dir_all(".knecht").unwrap();
            std::fs::write(".knecht/tasks", "").unwrap();
            println!("Initialized knecht");
        }
        _ => eprintln!("Unknown command"),
    }
}
```

### Step 3: Make `add` Pass
```rust
Some("add") => {
    let title = args[2..].join(" ");
    let next_id = get_next_id();
    let line = format!("{}|open|{}\n", next_id, title);
    std::fs::OpenOptions::new()
        .append(true)
        .open(".knecht/tasks")
        .unwrap()
        .write_all(line.as_bytes())
        .unwrap();
    println!("Created task-{}", next_id);
}
```

### Step 4: Make `list` Pass
Read file, split lines, format output.

### Step 5: Make `done` Pass
Read all lines, find matching ID, change status, write back.

### Step 6: Refactor
Now that tests pass, extract functions, improve code structure.

## Dogfooding from Minute 1

As soon as `add` and `list` work, USE THEM:

```bash
# You've just made the first test pass
knecht add "Write test for 'done' command"
knecht add "Make 'done' test pass"
knecht add "Refactor: extract parse_task function"
knecht add "Write README with usage examples"

# Now work through them
knecht list
knecht done task-1
```

**Critical**: Track ALL remaining knecht development in knecht itself.

## Project Structure (Minimal)

```
knecht/
├── Cargo.toml
├── src/
│   └── main.rs          # One file until you feel pain
├── tests/
│   └── integration_test.rs
└── .knecht/
    └── tasks            # Tracking knecht's own development!
```

Start with one `main.rs` file. Split it up when it gets annoying.

## The Pain-Driven Feature Roadmap

Add features **only when you feel the pain**. Here's what pain looks like:

### Phase 2: When You Feel Pain (Maybe Day 2)

**Pain**: "I need to see only open tasks"
```bash
knecht list --status open
```

**Test first:**
```rust
#[test]
fn can_filter_by_status() {
    // ... setup ...
    let result = run_command(&["list", "--status", "open"], &temp);
    assert!(!result.stdout.contains("task-3")); // done task excluded
}
```

**Pain**: "I can't work on task-5 until task-3 is done"
```bash
knecht add "Deploy to production" --blocked-by task-3
```

**Test first:**
```rust
#[test]
fn can_add_task_with_blocker() {
    // ... verify blocked-by stored and shown ...
}
```

**Pain**: "I want to see what's ready to work on"
```bash
knecht ready
```

Shows tasks with no blockers.

### Phase 3: Agent Integration (When Needed)

**Pain**: "An AI agent needs to parse this"
```bash
knecht list --json
```

**Test first:**
```rust
#[test]
fn can_output_json() {
    let result = run_command(&["list", "--json"], &temp);
    let tasks: Vec<Task> = serde_json::from_str(&result.stdout).unwrap();
    assert_eq!(tasks.len(), 2);
}
```

### Phase 4: Discovery Features (When Needed)

**Pain**: "I discovered a bug while working on task-5"
```bash
knecht add "Fix race condition in parser" --parent task-5
```

## Rules for Adding Features

Before adding ANY feature, ask:

1. **Do I have a failing test?** 
   - No test = no feature. Period.

2. **Am I tracking this feature in knecht?**
   - Dogfood or don't build it.

3. **Have I felt actual pain?**
   - "Might need" ≠ pain. Wait for real pain.

4. **What's simpler?**
   - Can I solve this with an alias or script instead?

5. **Can I defer this?**
   - Almost always yes.

## What We're Explicitly NOT Building Yet

Defer these until post-v1.0:

- ❌ Task IDs using cryptographic hashes (sequential integers work fine)
- ❌ Graph visualization (just list blockers in text)
- ❌ Git commit integration (use git separately)
- ❌ StoryTime story references (add fields when needed)
- ❌ Subtask hierarchies (flat list is fine)
- ❌ Task notes/descriptions (use title, add field later)
- ❌ Update command (done + new task works)
- ❌ Timestamps (add when you need to sort by date)
- ❌ Multiple task files (one file scales to 1000s of tasks)
- ❌ Config files (hardcode sensible defaults)
- ❌ Pretty colors (basic formatting works)

## Success Criteria for V0.1

You can ship when:

1. ✅ All tests pass
2. ✅ You're using knecht to track knecht's development
3. ✅ You can add, list, and complete tasks
4. ✅ Basic README exists
5. ✅ Total implementation time: < 4 hours

If it takes longer than 4 hours, you're building too much.

## Development Timeline (Actual)

### Hour 1: Walking Skeleton
- Write first test
- Implement init, add, list (minimal)
- **Use knecht to track remaining work**

### Hour 2: Complete Core Loop
- Implement done command
- All tests pass
- Refactor if needed

### Hour 3: Polish & Dogfood
- Fix rough edges you hit while using it
- Write README
- Track 5-10 real tasks in knecht

### Hour 4: Ship It
- Create git repo
- Tag v0.1
- Write blog post about building it
- **Use knecht to track v0.2 features**

## Example: First Hour of Development

```bash
# Terminal 1: TDD cycle
cargo new knecht
cd knecht

# Copy first test into tests/integration_test.rs
cargo test  # FAIL - good!

# Write minimal main.rs to pass test
cargo test  # PASS - ship it!

# Terminal 2: Self-hosting begins
./target/debug/knecht init
./target/debug/knecht add "Write test for 'done' command"
./target/debug/knecht add "Implement 'done' command"
./target/debug/knecht add "Write test for blocked-by"
./target/debug/knecht add "Extract task parsing to function"
./target/debug/knecht list

# See your real work in a real tool you just built!
```

## Why This Approach Works

**Traditional Spec Problems:**
- Designs everything up front (BDUF)
- Adds features "just in case" (YAGNI violations)
- Tests after implementation (no TDD safety net)
- Can't validate assumptions early

**This Spec:**
- ✅ Test-first forces concrete thinking
- ✅ Self-hosting validates usefulness immediately
- ✅ Pain-driven features ensure everything earns its keep
- ✅ Fast to working software (hours not weeks)
- ✅ Real feedback loop from day 1

## The Kent Beck Test

If Kent Beck were pair programming with you:

**Bad**: "Let's design the dependency graph algorithm"
**Good**: "Let's write a test for adding one task"

**Bad**: "We'll need JSON output eventually"  
**Good**: "Do we need JSON output right now?"

**Bad**: "Let me think about the architecture"
**Good**: "Let me write a failing test"

**Bad**: "This should take a few weeks"
**Good**: "Let's ship something today"

## Getting Started (Right Now)

1. `cargo new knecht`
2. Copy the first test into `tests/integration_test.rs`
3. Add test helpers (setup_temp_dir, run_command, etc.)
4. Run `cargo test` - watch it fail
5. Write minimal code to pass
6. Run test - watch it pass
7. **Use knecht to track the next test**
8. Repeat

Don't read past this point. Go write that first test.

---

## Appendix: Complete V0.1 Test Suite

```rust
// tests/integration_test.rs

#[test]
fn init_creates_tasks_file() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    assert!(temp.join(".knecht/tasks").exists());
}

#[test]
fn add_creates_sequential_ids() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    
    let r1 = run_command(&["add", "First task"], &temp);
    assert!(r1.stdout.contains("task-1"));
    
    let r2 = run_command(&["add", "Second task"], &temp);
    assert!(r2.stdout.contains("task-2"));
}

#[test]
fn list_shows_all_tasks() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    run_command(&["add", "Task one"], &temp);
    run_command(&["add", "Task two"], &temp);
    
    let result = run_command(&["list"], &temp);
    assert!(result.stdout.contains("task-1"));
    assert!(result.stdout.contains("task-2"));
    assert!(result.stdout.contains("Task one"));
    assert!(result.stdout.contains("Task two"));
}

#[test]
fn done_marks_task_complete() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    run_command(&["add", "Task to complete"], &temp);
    
    let result = run_command(&["done", "task-1"], &temp);
    assert!(result.success);
    
    let list = run_command(&["list"], &temp);
    assert!(list.stdout.contains("[x]") || list.stdout.contains("✓"));
}

#[test]
fn done_on_nonexistent_task_fails_gracefully() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);
    
    let result = run_command(&["done", "task-999"], &temp);
    assert!(!result.success);
    assert!(result.stderr.contains("not found") || 
            result.stderr.contains("doesn't exist"));
}
```

These 5 tests define v0.1. Make them pass, ship it, use it.

## Appendix: Data Format Evolution

**V0.1**: Pipe-delimited
```
1|open|Write first test
```

**V0.2**: Add blocked-by (if needed)
```
1|open|Write first test|
2|open|Make it pass|1
```

**V0.3**: Add timestamps (if needed)
```
1|open|Write first test||2024-01-05T10:30:00Z
```

**V1.0**: Maybe JSON if we have 10+ fields
```json
{"id":1,"status":"open","title":"Write first test"}
```

Don't prematurely optimize. Evolve the format when you feel pain.

## Final Words

This spec is not a plan. It's a process.

The test is the spec.
The tool building itself is the validation.
The pain is the product manager.

Start with the first test. Everything else follows.

Now go write that test. 🎯