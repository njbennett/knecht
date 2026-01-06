# Handoff: Implementing Description Support for Knecht

## Context

We're migrating the storytime project from beads to knecht. We built a working `beads2knecht` migration tool, but discovered that **descriptions are a blocker** - storytime has rich, multi-line markdown descriptions (some 500+ lines!) that we need to preserve.

## Current State

**What's Done:**
- ✅ Built `beads2knecht` tool in `src/bin/beads2knecht.rs`
- ✅ Tool converts beads JSON to knecht format
- ✅ Made decisions: keep descriptions, drop priorities/types
- ✅ Committed: See git log for "task-8 (phase 1)"

**What's Blocking:**
- ❌ knecht v0.1 only has: `{id}|{status}|{title}` (no descriptions)
- ❌ Need to add description field to preserve storytime's task data

## Your Mission: task-17

**Add description field to knecht format**

This is the CRITICAL PATH blocker for the entire migration.

### Requirements

1. **Support multi-line descriptions** (some are 500+ lines of markdown)
2. **Keep it git-friendly** (good diffs when descriptions change)
3. **Handle edge cases:**
   - Pipe characters (`|`) in descriptions
   - Newlines in descriptions
   - Empty/missing descriptions (backwards compatible with v0.1)
4. **Follow TDD** (write failing test first!)
5. **Stay simple** (knecht philosophy: simplest possible solution)

### Current Format (v0.1)

```
1|open|Write first test
2|done|Make it pass
3|open|Commit the code
```

Format: `{id}|{status}|{title}`

### Proposed Format (v0.2) - You Decide!

**Option A: Fourth pipe-delimited field**
```
1|open|Write first test|
2|done|Make it pass|This is a description
with multiple lines
3|open|Commit|
```

**Option B: Separate description block**
```
1|open|Write first test
2|open|Make it pass
---description-2
This is a description
with multiple lines
---end
3|open|Commit
```

**Option C: Something else entirely**

You decide! Consider:
- What makes the best git diffs?
- What's simplest to parse?
- What handles edge cases cleanly?

### Files to Modify

1. **`src/task.rs`** - Add description field to Task struct
2. **`tests/integration_test.rs`** - Write tests FIRST
3. **`src/main.rs`** - Update commands if needed (e.g., `knecht add --description "..."`)
4. **`.knecht/tasks`** - The actual tasks file (will need migration)

### TDD Workflow (MANDATORY)

```bash
# 1. Write failing test
cargo test  # Should FAIL

# 2. Implement feature
cargo test  # Should PASS

# 3. Update knecht's own tasks
knecht list  # Should work with old format
# Migrate to new format if needed

# 4. Track completion
knecht done task-17
git commit -m "task-17: Add description field to knecht format"
```

### Test Cases to Consider

```rust
#[test]
fn test_task_with_description() {
    // Task with description
}

#[test]
fn test_task_without_description_backwards_compatible() {
    // Old format still works
}

#[test]
fn test_description_with_pipes() {
    // Description containing | characters
}

#[test]
fn test_description_with_newlines() {
    // Multi-line description
}

#[test]
fn test_empty_description() {
    // Empty string vs None
}
```

## After task-17 is Complete

The next agent will work on:

**task-18**: Update beads2knecht to preserve descriptions
- Modify `src/bin/beads2knecht.rs`
- Map beads `description` field to knecht description field
- Test with real storytime data in `test_beads_data.json`

**task-15**: Write test for beads2knecht tool
**task-16**: Add README documentation for beads2knecht tool

Then:
**task-19**: Replace bd instructions with knecht in storytime .rules
**task-20**: Document how to uninstall/remove beads from storytime

Finally:
**task-8**: Execute the actual migration!

## Important Files

- `BEADS_MIGRATION.md` - Full Mikado Method discovery document
- `MIKADO_EXAMPLES.md` - Examples of Mikado Method workflow
- `.rules` - Project philosophy (TDD, pain-driven, YAGNI)
- `test_beads_data.json` - Real storytime tasks (in .gitignore)

## Design Considerations

### Git-Friendly Diffs

Good:
```diff
 1|open|Task title
+---description-1
+This is a new description
+---end
 2|done|Another task
```

Bad:
```diff
-1|open|Task title|
+1|open|Task title|This is a new description with everything on one line making diffs hard to read
```

### Pipe Character Escaping

If using pipe-delimited format, consider:
- URL encoding? (`%7C` for pipe)
- Escape sequences? (`\|` for literal pipe)
- Different delimiter? (but stays consistent with v0.1)
- Separate format entirely?

### Backwards Compatibility

**Critical**: Existing v0.1 tasks must still work!

```
# This should still parse correctly:
1|open|Task without description
2|done|Another old task

# And new format:
3|open|New task|With description
```

### Performance

knecht reads the entire `.knecht/tasks` file into memory. With large descriptions:
- Still fast for 100s of tasks
- File I/O is simple and predictable
- Don't optimize prematurely

## Philosophy Reminder

From `.rules`:

1. **TDD**: Write failing test first
2. **Pain-Driven**: We have pain count #2 for descriptions (real pain!)
3. **YAGNI**: Build simplest thing that works
4. **Zero Warnings**: Fix all cargo warnings immediately
5. **Self-Hosting**: Track work in knecht itself

## Questions to Answer

As you implement, consider documenting:

1. Why did you choose the format you chose?
2. What edge cases did you discover?
3. Are there any breaking changes from v0.1?
4. How should users migrate their existing tasks?

## Success Criteria

- [ ] Tests pass for tasks with descriptions
- [ ] Tests pass for tasks without descriptions (backwards compatible)
- [ ] Edge cases handled (pipes, newlines, empty)
- [ ] No cargo warnings
- [ ] knecht's own `.knecht/tasks` file works with new format
- [ ] task-17 marked done and committed

## Good Luck!

You're working on the critical path for the beads migration. Once descriptions work, everything else falls into place.

Remember: **Write the test first!** 🎯