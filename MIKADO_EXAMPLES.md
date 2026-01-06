# Mikado Method with knecht

This document explores how knecht could support the Mikado Method using a **blockers-on-tasks** model.

## What is the Mikado Method?

1. **Set a Goal** - Define what you want to achieve
2. **Try It** - Attempt the change directly (timeboxed: 5-15 min)
3. **Hit Blockers** - Note the actual problems that prevented success
4. **Revert** - Go back to stable ground (don't keep broken code)
5. **Create Tasks to Address Blockers** - Turn each blocker into a task
6. **Repeat** - Try those tasks, discover their blockers
7. **Find Leaves** - Tasks with no blockers are ready to work on
8. **Complete & Commit** - Do the leaf task, commit it
9. **Work Up the Tree** - As you resolve blockers, parent tasks become ready

The result is a **dependency graph discovered through experimentation**.

## The Blocker Model

Instead of task-depends-on-task relationships, we model:

**Tasks have blockers** (problems that prevent completion)
**Tasks address blockers** (attempts to solve problems)

### Example

```
task-1: Remove Doctrine ORM
  blockers:
    - "EntityManager used in 5 repositories"
    - "Entities have @ORM annotations"

task-2: Extract repository interfaces
  addresses: "EntityManager used in 5 repositories"
  blockers:
    - "UserRepository has QueryBuilder logic"

task-3: Simplify UserRepository
  addresses: "UserRepository has QueryBuilder logic"
  blockers: []  # LEAF! Ready to work on!
```

This is closer to reality:
- You document **actual problems**, not abstract dependencies
- Multiple tasks can attempt to solve the same blocker
- You can see WHY a task exists (it's solving a specific problem)

## Data Format for v0.2

Current (v0.1):
```
1|open|Remove Doctrine ORM
```

Possible (v0.2):
```
1|open|Remove Doctrine ORM||EntityManager used in repos; ORM annotations everywhere
2|open|Extract repository interfaces|1|UserRepository has QueryBuilder
3|done|Simplify UserRepository|2|
```

Format: `{id}|{status}|{title}|{addresses-blocker-in-task-id}|{blockers}`

Where:
- `addresses-blocker-in-task-id` = which task's blocker this solves (empty if goal task)
- `blockers` = semicolon-separated list of problems (empty if leaf)

## Example 1: Remove Doctrine ORM (Manual with v0.1)

You can approximate this TODAY with v0.1 by using task titles and notes:

```bash
knecht init

# Set the goal
knecht add "GOAL: Remove Doctrine ORM"
# task-1

# Attempt: composer remove doctrine/orm
# FAILS with error: EntityManager not found in UserRepository.php
# REVERT: git reset --hard

# Document the blocker
knecht add "BLOCKER (task-1): EntityManager used in repositories"
# Just for tracking, not a real task

# Create task to address it
knecht add "Extract repository interfaces [addresses blocker in task-1]"
# task-2

# Attempt task-2
# FAILS: UserRepository uses QueryBuilder, can't extract clean interface
# REVERT

# Create task to address that blocker
knecht add "Simplify UserRepository - remove QueryBuilder [addresses blocker in task-2]"
# task-3

# Attempt task-3
# SUCCESS! This is a leaf - it has no blockers
knecht done task-3
git commit -m "task-3: Simplify UserRepository.findByEmail()"

# Now try task-2 again
knecht done task-2
git commit -m "task-2: Extract UserRepositoryInterface"

# Discover another blocker for task-1
knecht add "Remove @ORM annotations from entities [addresses blocker in task-1]"
# task-4

# This is also a leaf!
knecht done task-4
git commit -m "task-4: Replace annotations with PHP 8 attributes"

# Now task-1 is ready!
knecht done task-1
git commit -m "task-1: Remove Doctrine ORM dependency"
```

### What works (v0.1):
- ✅ Document blockers as you discover them
- ✅ Track tasks that address blockers
- ✅ Complete leaves and commit
- ✅ See progress toward goal

### What's missing:
- ❌ Can't query "what are task-1's blockers?"
- ❌ Can't see "which tasks address blocker X?"
- ❌ Can't automatically find leaf tasks (no blockers)
- ❌ Must manually track relationships in titles

## Example 2: Extract AuthService (Discovery Process)

```bash
knecht add "GOAL: Extract AuthService from User model"
# task-10

# Attempt: Create AuthService and move methods
# FAILS: password hashing is mixed throughout User class
# FAILS: session handling is tightly coupled
# FAILS: no clear boundary between auth and user domain
# REVERT

# Document blockers on task-10
# (With v0.2 would be: knecht blocker task-10 "password hashing mixed in User")
# For now, just add notes to task-10 or separate tracking tasks

knecht add "Extract password hashing to PasswordHasher [addresses auth extraction]"
# task-11

knecht add "Extract session handling [addresses auth extraction]"
# task-12

knecht add "Define Auth domain boundary [addresses auth extraction]"
# task-13

# Try task-11: Extract password hashing
# SUCCESS! This is simple, no blockers
knecht done task-11
git commit -m "task-11: Create PasswordHasher class"

# Try task-12: Extract session handling
# FAILS: Session is stored in User model directly
# REVERT

knecht add "Add Session entity [addresses session extraction]"
# task-14

# Try task-14
# SUCCESS! Leaf node
knecht done task-14
git commit -m "task-14: Create Session entity"

# Now task-12 is unblocked
knecht done task-12
git commit -m "task-12: Extract SessionManager"

# Try task-13: Define boundary
# SUCCESS! 
knecht done task-13
git commit -m "task-13: Add Auth namespace and interfaces"

# Now task-10 is unblocked!
knecht done task-10
git commit -m "task-10: Extract AuthService"
```

The graph discovered:
```
task-10 (GOAL)
  ├─ task-11 ✓ (leaf)
  ├─ task-12
  │   └─ task-14 ✓ (leaf)
  └─ task-13 ✓ (leaf)
```

## Example 3: Agent Workflow

An AI agent working with the blocker model:

```bash
# Agent receives goal
knecht add "Add Stripe payment processing"
# task-20

# Agent attempts implementation
# ERROR: No payments table exists
# REVERT

# Agent creates task to address blocker
knecht add "Create payments table [addresses: no payments table for task-20]"
# task-21

# Agent attempts task-21
# ERROR: No migration tool available
# REVERT

# Agent creates task for that blocker
knecht add "Add database migration tool [addresses: no migrations for task-21]"
# task-22

# Agent attempts task-22
# SUCCESS! This is a leaf
knecht done task-22
git commit -m "task-22: Add doctrine/migrations"

# Agent can now complete task-21
knecht done task-21
git commit -m "task-21: Create payments table migration"

# Agent discovers another blocker for task-20
knecht add "Add Stripe SDK [addresses: no Stripe SDK for task-20]"
# task-23

# Leaf! Complete it
knecht done task-23
git commit -m "task-23: composer require stripe/stripe-php"

# Now task-20 has no blockers
knecht done task-20
git commit -m "task-20: Implement Stripe payment service"
```

**Agent benefits:**
- Documents WHY each task exists (addresses specific blocker)
- Human can review the discovery process
- Each commit is stable and reviewable
- Clear trail from goal to prerequisites

## Commands for v0.2 (Add When You Feel Pain)

As you use v0.1 manually, you'll feel pain. Track it!

**Pain 1**: "I can't remember what's blocking task-5"
```bash
# Possible solution:
knecht blocker task-5 "UserRepository uses QueryBuilder"
knecht blocker task-5 "No interface defined"

knecht show task-5
# Blockers:
#   - UserRepository uses QueryBuilder
#   - No interface defined
```

**Pain 2**: "I don't know which tasks are leaves"
```bash
# Possible solution:
knecht ready
# Output:
# task-3  Simplify UserRepository
# task-7  Add Session entity
# (these have no blockers)
```

**Pain 3**: "I forgot which task addresses which blocker"
```bash
# Possible solution:
knecht add "Simplify UserRepository" --addresses task-2

knecht show task-2
# This blocker is addressed by:
#   task-5: Simplify UserRepository
```

**Pain 4**: "I can't see the whole discovery graph"
```bash
# Possible solution:
knecht graph
# Output:
# task-1: Remove Doctrine ORM
#   addresses: (goal)
#   blockers: EntityManager in repos
#   └─ task-2: Extract interfaces
#       addresses: EntityManager in repos
#       blockers: QueryBuilder usage
#       └─ task-3: Simplify UserRepository ✓
#           addresses: QueryBuilder usage
#           blockers: (none - LEAF)
```

## Possible v0.2 Commands

Only add these when manual tracking hurts:

```bash
# Add blockers to a task
knecht blocker <task-id> <description>

# Create task that addresses a blocker
knecht add <title> --addresses <task-id>

# Remove a blocker (when you realize it's not actually blocking)
knecht unblocker <task-id> <description>

# Show leaf tasks (no blockers)
knecht ready

# Show the discovery graph
knecht graph

# Show task details with blockers and what addresses them
knecht show <task-id>
```

## Key Principles

### 1. Blockers are Problems, Not Tasks

```bash
# GOOD: Describe the actual problem
"UserRepository has 200 lines of QueryBuilder logic"
"No interface exists for repository"
"Session data stored directly on User model"

# BAD: Describe a solution
"Need to refactor UserRepository"
"Should add an interface"
```

### 2. Tasks Address Blockers

```bash
# A task exists to solve a specific problem
knecht add "Simplify UserRepository" --addresses task-2

# Not just abstract dependencies
knecht add "Do step 2" --depends-on task-1  # Wrong model!
```

### 3. Multiple Tasks Can Address the Same Blocker

```bash
# Try one approach
knecht add "Use Strategy pattern for auth" --addresses task-10

# That fails, try another
knecht add "Extract AuthService with simple delegation" --addresses task-10

# Experimentation is part of the process!
```

### 4. Leaves Have No Blockers

```bash
knecht list
# [ ] task-1  Remove Doctrine (blockers: 2)
# [ ] task-2  Extract interface (blockers: 1)
# [ ] task-3  Simplify repo (blockers: 0)  ← LEAF! Do this first

# Always work on leaves
# When complete, parent tasks might become leaves
```

## Try It Today with v0.1

Manual workflow that approximates the blocker model:

```bash
# 1. Create goal
knecht add "GOAL: Your big scary change"

# 2. Try it (timebox!)
# ... attempt the change ...
# ... it fails ...
git reset --hard

# 3. Document blockers in notes or separate task
knecht add "BLOCKERS for task-1: problem A; problem B; problem C"

# 4. Create tasks to address blockers
knecht add "Solve problem A [addresses blocker in task-1]"
knecht add "Solve problem B [addresses blocker in task-1]"

# 5. Try those tasks, discover their blockers
# ... repeat process ...

# 6. Find leaves (tasks with no [addresses blocker] in title)
knecht list
# Scan for tasks that don't have child tasks

# 7. Complete leaves
knecht done task-X
git commit -m "task-X: Small safe change"

# 8. Work up the tree
# As you complete tasks, their parents become ready
```

**Track your pain:**

Every time you think "I wish I could see what's blocking task-5", add a note:
```bash
knecht add "Pain: need 'knecht show task-5' to see blockers (count: 3)"
```

## The Philosophy

**Discovery over Planning**
- Don't try to plan the whole graph upfront
- Discover blockers by attempting the work
- Revert frequently, commit small wins

**Blockers are Reality**
- They describe actual problems you hit
- Not abstract "this depends on that"
- Documentation of your learning process

**Tasks are Experiments**
- Each task is an attempt to solve a blocker
- Some will fail and reveal deeper blockers
- That's the process working correctly

**Always Keep Main Green**
- Revert failed attempts
- Only commit successful leaves
- Every commit should pass tests

## Summary

knecht v0.1 can support Mikado-style work today through manual tracking.

The blocker model is more natural than task dependencies:
- **Blockers** = Real problems you discover
- **Tasks** = Attempts to solve those problems
- **Leaves** = Tasks with no blockers (ready to work on)

Add explicit blocker support when manual tracking becomes painful.

Stay test-driven. Stay pain-driven. 🎯
