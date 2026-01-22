# Upstream Feedback - File Friction in Knecht Project

You're filing feedback about knecht (the task tracker tool) from a downstream project.

## Path Discovery

Find the knecht project path using this priority:
1. Check if `KNECHT_PROJECT` environment variable is set
2. Check if CLAUDE.md contains a `knecht_project_path:` configuration line
3. Try default: `~/workspace/knecht`
4. If none work, ask the user for the knecht project path

## Required Context

Before proceeding, gather this information from your /reflect context:
- **Source project**: The current working directory name (e.g., "storytime")
- **Task context**: What task you were working on when friction occurred
- **Friction description**: The specific issue with knecht

## Workflow

### Step 1: Verify knecht project access

Run: `cd $KNECHT_PATH && knecht list`

If this fails, the path is wrong. Ask user for correct path.

### Step 2: Search for related tasks

Review the task list output. Look for tasks that might already cover this friction:
- Similar wording in title or description
- Same general area (e.g., "list command", "output formatting")

### Step 3: Decision point

Present to the agent (yourself) or user:
- If related task exists: "Task-XXX looks related. Increment pain on it, or create new task?"
- If no related task: "No existing task found. Creating new task."

### Step 4: Execute action

**To create new task:**
```bash
cd $KNECHT_PATH && knecht add "title" -d "From [source_project] while working on [task_context]: [friction_description]"
```

**To increment pain on existing task:**
```bash
cd $KNECHT_PATH && knecht pain -t task-XXX -d "From [source_project] while working on [task_context]: [friction_description]"
```

### Step 5: Confirm

Report what was filed:
- Task ID created or updated
- Full description that was recorded
- Remind that this will be picked up in knecht's next session

## Description Format

Always include in the description:
```
From [project_name] while working on [task_id/description]: [friction]
```

Example:
```
From storytime while working on task-42 (sync feature): knecht list output is hard to parse programmatically - no structured format option
```

## Important Notes

- This skill is invoked BY /reflect, not directly by users
- The friction should be about knecht itself, not the downstream project
- If you're unsure whether something is knecht friction, err on the side of filing it
