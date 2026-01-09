# Migrating from Beads to Knecht

This guide is for AI agents working in any project to migrate from `beads` (bd) to `knecht` for task management.

## Prerequisites

- The `knecht` binary must be installed and available in your PATH
- The `beads2knecht` migration tool must be built (see the "Migration from Beads" section in the knecht README)
- Your project currently uses beads with tasks in `.beads/`

## Migration Steps

### 1. Export Beads Data

```bash
cd /path/to/your/project
bd list --json > /tmp/beads_export.json
```

This exports all current tasks from beads to JSON format.

### 2. Run Migration Tool

```bash
/path/to/knecht/target/release/beads2knecht < /tmp/beads_export.json > /tmp/knecht_tasks.txt 2>/tmp/migration_report.txt
```

This converts beads format to knecht format. The migration report (stderr) will show statistics about what was preserved and what was dropped.

### 3. Review Migration Report

```bash
cat /tmp/migration_report.txt
```

Check:
- How many tasks were converted
- How many descriptions were preserved
- What priorities and issue types are being dropped

If important information is being lost, consider adding it to task titles or descriptions before proceeding.

### 4. Initialize Knecht

```bash
cd /path/to/your/project
knecht init
```

This creates the `.knecht/` directory.

### 5. Import Migrated Tasks

```bash
grep -v '^#' /tmp/knecht_tasks.txt > .knecht/tasks
```

The `grep -v '^#'` filters out the comment header lines from the migration output.

### 6. Verify Migration

```bash
knecht list
knecht show task-2  # Check that descriptions are preserved
knecht next         # Verify the command works
```

Confirm:
- All tasks appear in the list
- Descriptions are intact (for tasks that had them)
- Status mappings are correct (in_progress → open, done → done)

### 7. Update .rules File

The `.rules` file may still have references to `bd` that need updating:

```bash
# Find any remaining bd references
grep -n "bd " .rules

# Update them to use knecht
# Example: "make a note using bd" → "add a task using knecht"
```

Edit `.rules` to replace beads commands with knecht equivalents:
- `bd list` → `knecht list`
- `bd add` → `knecht add`
- `bd done` → `knecht done`

### 8. Stage and Commit

```bash
git add .knecht/tasks .rules
git status  # Review what will be committed
```

**IMPORTANT**: Before committing, review the changes:
- `.knecht/tasks` - The new task file
- `.rules` - Only bd→knecht updates

Do NOT commit unrelated changes from other work. Only commit migration-related files.

### 9. Commit the Migration

```bash
git commit -m "Migrate from beads to knecht task tracking

- Migrated [N] tasks from beads to knecht format
- Updated .rules to reference knecht instead of bd
- Preserved task titles, descriptions, and status
- Dropped beads-specific metadata (priorities, issue types, dependencies)"
```

## What Gets Migrated

### Preserved
- **Task titles** - Unchanged
- **Task descriptions** - Fully preserved in 4th pipe-delimited field
- **Task status** - Mapped: `open`→`open`, `in_progress`→`open`, `done`→`done`

### Converted
- **IDs** - Beads alphanumeric IDs (e.g., `storytime-67d`) → Sequential numbers (1, 2, 3...)
- **Status** - `in_progress` mapped to `open` (knecht only has open/done)

### Dropped (By Design)
- **Priorities** (0-4) - Can be expressed in task titles if critical
- **Issue types** (bug/task/epic/feature/chore) - Can be expressed in titles
- **Timestamps** - Git history provides this
- **Dependencies** - Not yet supported in knecht (may be added later)

## Handling Lost Information

If priorities or issue types are important, consider:

1. **High-priority tasks**: Add "Priority:" or "URGENT:" prefix to titles
2. **Issue types**: Add type prefix like "Bug:", "Feature:", "Chore:" to titles
3. **Dependencies**: Add notes in descriptions about what tasks should be done first

Example:
```
Before: "Fix authentication bug" (priority: 0, type: bug)
After:  "Bug: Fix critical authentication bug" or "URGENT: Fix authentication bug"
```

## Verification Checklist

After migration, verify:

- [ ] `knecht list` shows all expected tasks
- [ ] `knecht show task-N` displays descriptions correctly (for tasks that had them)
- [ ] `knecht next` provides a suggestion
- [ ] `.rules` references knecht, not bd
- [ ] Migration documentation exists
- [ ] Only migration files are staged for commit
- [ ] Git history is clean (no unrelated changes)

## Troubleshooting

### "knecht: command not found"
Install knecht and add it to your PATH:
```bash
cd /path/to/knecht
cargo build --release
sudo cp target/release/knecht /usr/local/bin/
```

### "beads2knecht: not found"
Build the migration tool:
```bash
cd /path/to/knecht
cargo build --release --bin beads2knecht
# Then use: /path/to/knecht/target/release/beads2knecht
```

### Multi-line descriptions appear broken
This is expected - the pipe-delimited format preserves multi-line descriptions. Use `knecht show task-N` to view them properly formatted.

### Lost important priority information
Edit `.knecht/tasks` directly and add priority info to task titles before committing.

## Post-Migration

After successful migration:

1. **Use knecht exclusively** - Stop using `bd` commands
2. **Commit `.knecht/tasks` with code changes** - Keep tasks synchronized with work
3. **Archive or remove `.beads/`** - The old data is no longer needed
4. **Update documentation** - If README or other docs reference beads, update them

## Questions?

See the knecht README for full usage documentation in the knecht repository.