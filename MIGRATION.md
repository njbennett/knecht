# Migrating from beads to knecht

This guide covers how to migrate a project from using beads task tracking to knecht.

## Prerequisites

- Rust and cargo installed (for building knecht)
- Git (to preserve task history)

## Step 1: Install knecht

```bash
cd /path/to/knecht
cargo build --release
sudo cp target/release/knecht /usr/local/bin/
sudo cp target/release/beads2knecht /usr/local/bin/
```

Verify installation:
```bash
which knecht
knecht list  # Should work in any directory with .knecht/tasks
```

## Step 2: Migrate existing beads tasks

If you have existing tasks in `.beads/`, use the migration tool:

```bash
cd /path/to/your/project
beads2knecht
```

This will:
- Read tasks from `.beads/beads`
- Convert them to knecht format
- Write to `.knecht/tasks`
- Preserve descriptions, but drop beads-specific fields (priority, issue_type)

**Review the output** to ensure tasks migrated correctly:
```bash
knecht list
```

## Step 3: Update project documentation

If your project has an `AGENTS.md` or `.rules` file mentioning beads, update the instructions:

**Before:**
```markdown
Use 'bd' for task tracking
```

**After:**
```markdown
Use 'knecht' for task tracking
```

Update any specific beads commands in `.rules`:
- `bd list` → `knecht list`
- `bd add "Task"` → `knecht add "Task"`
- `bd done N` → `knecht done task-N`
- `bd start N` → `knecht start task-N`

## Step 4: Uninstall beads

### Remove the beads binary

```bash
which bd
# Typical locations: /usr/local/bin/bd or ~/.local/bin/bd

sudo rm /usr/local/bin/bd
# Or if installed in user directory:
rm ~/.local/bin/bd
```

### Remove beads data directory (OPTIONAL)

**WARNING:** Only do this after verifying your tasks migrated correctly!

```bash
# Backup first (just in case)
cp -r .beads .beads.backup

# Review what will be deleted
ls -la .beads/

# Remove
rm -rf .beads/
```

You may want to keep `.beads.backup` in git history, or commit the removal:
```bash
git add .beads/
git commit -m "Remove beads - migrated to knecht"
```

## Step 5: Commit the migration

```bash
git add .knecht/ .rules AGENTS.md
git commit -m "Migrate from beads to knecht"
```

## Step 6: Verify the migration

Test the basic workflow:
```bash
knecht list
knecht add "Test task after migration"
knecht start task-X
knecht done task-X
```

If you have AI agents working on the project, test that they can use knecht commands successfully.

## Differences between beads and knecht

| Feature | beads | knecht |
|---------|-------|--------|
| Task IDs | Numeric (1, 2, 3) | Prefixed (task-1, task-2) |
| Descriptions | Supported | Supported (use `-d` flag) |
| Priorities | 0-4 scale | Not supported (use descriptions) |
| Issue types | bug/task/epic/feature/chore | Not supported (use descriptions) |
| Data format | Pipe-delimited | Pipe-delimited (similar) |
| Commands | `bd list`, `bd add N` | `knecht list`, `knecht add` |

## Rollback (if needed)

If you need to roll back:

```bash
# Restore beads data
cp -r .beads.backup .beads

# Reinstall beads
curl -fsSL https://raw.githubusercontent.com/steveyegge/beads/main/scripts/install.sh | bash

# Remove knecht changes
git checkout .knecht/ .rules AGENTS.md
```

## Troubleshooting

**"knecht: command not found"**
- Ensure `/usr/local/bin` is in your PATH: `echo $PATH`
- Try running directly: `/usr/local/bin/knecht list`
- Or use cargo: `cd /path/to/knecht && cargo run -- list`

**"beads2knecht: command not found"**
- Build both binaries: `cargo build --release --bins`
- Copy both: `sudo cp target/release/{knecht,beads2knecht} /usr/local/bin/`

**Tasks didn't migrate correctly**
- Check `.beads/beads` format - should be pipe-delimited
- Run migration again (it will overwrite `.knecht/tasks`)
- Manually edit `.knecht/tasks` if needed

**Want to preserve beads metadata (priority, type)**
- Add to task descriptions during migration
- Example: "Fix login bug [priority:0] [type:bug]"