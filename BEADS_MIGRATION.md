# Beads to Knecht Migration - Mikado Method Discovery

## Goal (task-8)
Switch storytime project from beads to knecht

## Status
**Phase 1 Complete**: Basic migration tool built and decisions made.
**Phase 2 In Progress**: Adding description support to knecht (task-17).

## What We Built
A working `beads2knecht` tool that:
- Reads beads JSON format from stdin
- Converts to knecht pipe-delimited format
- Reports on information loss
- Can be used by agents in other projects

## Try It (Successful!)
```bash
# In storytime project:
bd list --json | path/to/knecht/target/debug/beads2knecht > .knecht/tasks

# Then use knecht normally:
knecht list
```

## Blockers Discovered & Decisions Made

### 1. Descriptions (task-12) ✅ DECISION: KEEP
**Beads has:** Multi-line markdown descriptions (some tasks have 500+ line descriptions!)
**Knecht v0.1 has:** Only titles (single line)
**Pain:** High - storytime has rich task descriptions with formatting

**DECISION:** Add description support to knecht (task-17)
- 4 out of 5 storytime tasks have descriptions
- Some descriptions are 500+ lines with detailed context
- This is pain count #2 for descriptions (task-7 was pain #1)
- **This is a BLOCKER for migration** - must have descriptions

**Implementation:**
- task-17: Add description field to knecht format
- task-18: Update beads2knecht to preserve descriptions

### 2. Priorities (task-13) ✅ DECISION: DROP
**Beads has:** Numeric priorities 0-4 (0=highest, 4=lowest)
**Knecht v0.1 has:** Nothing

**DECISION:** Drop priorities - not a blocker
- Can be expressed in task titles if needed ("High priority: Fix bug")
- Knecht philosophy: order tasks in the file manually
- If pain emerges later, track it and reconsider

### 3. Issue Types (task-14) ✅ DECISION: DROP
**Beads has:** bug, task, epic, feature, chore
**Knecht v0.1 has:** Nothing

**DECISION:** Drop issue types - not a blocker
- Can be expressed naturally in titles ("Bug: Fix marker", "Epic: Refactor auth")
- Not essential for task tracking
- If pain emerges later, track it and reconsider

### 4. Status Mapping ✅ (RESOLVED)
**Beads has:** open, in_progress, done
**Knecht has:** open, done
**Solution:** Map in_progress → open (working on it = still open)

### 5. ID Mapping ✅ (RESOLVED)
**Beads has:** Alphanumeric IDs (storytime-46, storytime-67d)
**Knecht has:** Sequential numbers (1, 2, 3...)
**Solution:** Map to sequential IDs. Beads IDs are lost but not needed.

### 6. Timestamps ✅ (RESOLVED - DROP)
**Beads has:** created_at, updated_at
**Knecht has:** Nothing
**Decision:** Drop them. Git commits provide history. Not needed for task tracking.

### 7. Dependencies (DEFERRED)
**Beads has:** dependency_count, dependent_count (but not full graph?)
**Knecht v0.1 has:** Nothing
**Knecht v0.2:** Planned blocker model (see MIKADO_EXAMPLES.md)

**Decision:** Drop for now. Knecht's blocker model (v0.2) is different anyway.
If storytime needs dependencies, that's pain to track for v0.2 design.

## Current Migration Tool Output

Example with 2 tasks:
```
# Beads to Knecht Migration
# 2 tasks found
#
# BLOCKERS DISCOVERED:
# 1. Beads has descriptions - knecht v0.1 doesn't
# 2. Beads has priorities (0-4) - knecht doesn't
# 3. Beads has issue_types (bug/task/epic/etc) - knecht doesn't
# 4. Beads has 'in_progress' status - knecht only has open/done
# 5. Beads has alphanumeric IDs - knecht uses sequential numbers
#
1|open|Test suite exceeds 2 second performance requirement
2|open|Extract marker updates into asynchronous function

=== MIGRATION COMPLETE ===
Tasks converted: 2

LOST INFORMATION:
- Descriptions: 1 tasks had descriptions
- Priorities: Distribution:
  Priority 0: 1 tasks
  Priority 1: 1 tasks
- Issue types:
  bug: 1 tasks
  task: 1 tasks
```

## Current Task Tree (Mikado Graph)

```
task-8: Switch storytime from beads to knecht
  ├─ task-12: Decide on descriptions ✅ (KEEP THEM)
  │   └─ task-17: Add description field to knecht ⭐ (BLOCKER - must do first)
  │       └─ task-18: Update beads2knecht to preserve descriptions
  ├─ task-13: Decide on priorities ✅ (DROP)
  ├─ task-14: Decide on issue types ✅ (DROP)
  ├─ task-15: Write test for beads2knecht tool (after task-18)
  ├─ task-16: Add README documentation for beads2knecht tool (after task-18)
  ├─ task-19: Replace bd instructions with knecht in storytime .rules ⭐ (LEAF)
  └─ task-20: Document how to uninstall/remove beads from storytime ⭐ (LEAF)
```

## Leaf Tasks (Ready to Work On Now)

- **task-17**: Add description field to knecht format ⭐⭐⭐ (CRITICAL PATH)
- **task-19**: Replace bd instructions with knecht in storytime .rules
- **task-20**: Document how to uninstall/remove beads from storytime

## Blocked Tasks (Need task-17 first)

- **task-18**: Update beads2knecht to preserve descriptions
- **task-15**: Write test for beads2knecht tool
- **task-16**: Add README documentation for beads2knecht tool

## Next Steps - Clear Path Forward

### Phase 2: Add Description Support (IN PROGRESS)
1. **task-17**: Add description field to knecht format (CRITICAL PATH)
2. **task-18**: Update beads2knecht to preserve descriptions
3. **task-15**: Write tests for beads2knecht tool
4. **task-16**: Add README documentation for beads2knecht tool

### Phase 3: Prepare Storytime for Migration
5. **task-19**: Replace bd instructions with knecht in storytime .rules
6. **task-20**: Document how to uninstall/remove beads from storytime

### Phase 4: Execute Migration
7. Run the migration tool on storytime
8. Verify all tasks migrated correctly
9. Uninstall beads from storytime
10. Complete task-8!

## Decisions Made ✅

- ✅ **Descriptions**: KEEP (blocker for migration)
- ✅ **Priorities**: DROP (not a blocker)
- ✅ **Issue Types**: DROP (not a blocker)
- ✅ **Timestamps**: DROP (git provides history)
- ✅ **Dependencies**: DROP (different model in v0.2)
- ✅ **Alphanumeric IDs**: Map to sequential numbers

## Implementation Notes

### New knecht Format with Descriptions

The format will need to support optional multi-line descriptions.

**Proposed format (task-17 will define exactly):**
```
1|open|Test suite exceeds 2 second performance requirement|
2|open|Improve test coverage to 90%|## Goal
Increase test coverage from 77.43% to 90%...

## Strategy
Focus on high-value modules...
3|done|Another task|
```

Challenges to solve in task-17:
- How to handle pipe characters in descriptions?
- How to preserve newlines in descriptions?
- How to make it backwards compatible with v0.1 tasks?
- How to keep it git-friendly (good diffs)?

## Files Created
- `src/bin/beads2knecht.rs` - The migration tool
- `Cargo.toml` - Added serde dependencies
- This document

## How to Use (for agents in other projects)

```bash
# In a project using beads:
bd list --json | /path/to/knecht/target/release/beads2knecht > migration_preview.txt

# Review the output and lost information

# If acceptable, initialize knecht and migrate:
knecht init
bd list --json | /path/to/knecht/target/release/beads2knecht | grep -v '^#' > .knecht/tasks

# Verify:
knecht list

# Start using knecht:
knecht add "New task"
knecht done task-1
```

## Philosophy Check ✅

- ✅ **TDD**: Need to add tests (task-15)
- ✅ **Pain-Driven**: Discovered real pain (descriptions)
- ✅ **YAGNI**: Built minimal tool first
- ✅ **Mikado Method**: Tried → hit blockers → documented → created sub-tasks
- ✅ **Self-Hosting**: Tracked work in knecht itself

## Summary

**Phase 1 Complete!** 
- ✅ Built working beads2knecht migration tool
- ✅ Discovered and documented all blockers
- ✅ Made decisions: keep descriptions, drop priority/types
- ✅ Identified clear path forward

**Phase 2 Starting:**
- 🎯 task-17: Add description field to knecht (CRITICAL PATH)
- Then update migration tool to preserve descriptions
- Then document and test the complete tool
- Then prepare storytime for migration

**Ready to work on task-17!** This is the blocker for the entire migration.