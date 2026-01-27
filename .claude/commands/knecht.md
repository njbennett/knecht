---
description: Knecht command reference - use CLI, never read files directly
---

# Knecht Command Reference

**RULE**: NEVER read `.knecht/tasks` or `.knecht/blockers` directly. Always use CLI commands.

## What do you want to do?

| Goal | Command |
|------|---------|
| See what to work on next | `knecht next` |
| View a specific task | `knecht show task-XXX` |
| List all tasks | `knecht list` |
| Create a new task | `knecht add "title" -a "criteria"` |
| Start working on a task | `knecht start task-XXX` |
| Mark task complete | `knecht done task-XXX` |
| Mark task delivered | `knecht deliver task-XXX` |
| Record pain on existing task | `knecht pain -t task-XXX -d "description"` |
| Update task details | `knecht update task-XXX [-t "title"] [-d "desc"]` |
| Block a task | `knecht block task-XXX --by task-YYY` |
| Remove a blocker | `knecht unblock task-XXX --from task-YYY` |
| Delete a task | `knecht delete task-XXX` |

## Don't Do This

| Bad | Good |
|-----|------|
| `Read .knecht/tasks` | `knecht list` or `knecht show` |
| `grep .knecht/tasks` | `knecht list` |
| `Write .knecht/tasks` | `knecht add`, `knecht done`, etc. |
| `cat .knecht/blockers` | `knecht show` (shows blockers) |

## Creating Tasks

Before running `knecht add`, propose the task and confirm with the user:

"I'd like to add a task: `knecht add "title" -a "criteria"`. OK?"

Don't add tasks without user approval.
