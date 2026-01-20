# Task Reflection - REQUIRED

You just completed work. Before continuing, you MUST answer these questions.

## Anti-Dismissal Rule (task-221)

If you catch yourself thinking "this isn't really a knecht bug":
- STOP - that thought pattern is the bug task-221 documents (5+ pain instances)
- File it AS a task anyway
- Your reasoning about why it's not knecht's problem IS the task content

## Required Questions

Answer each question explicitly:

### 1. What friction did you encounter?

List ANY moments of confusion, hesitation, or uncertainty - even small ones.
If you had to re-read something, look something up, or make a judgment call, that's friction.

### 2. Did the user correct or redirect you?

If yes, this is a knecht bug. File a task describing what knecht could have told you.

### 3. What IS a knecht bug? (Review this list)

These ARE knecht bugs - file tasks for them:
- Agent confusion about workflow → knecht UX should prevent it
- User had to redirect agent → knecht guidance should have covered it
- Agent judgment call went wrong → knecht should provide better defaults
- Agent failed to read something → knecht should surface it better
- Process/methodology issues → rules or prompts need updating
- Codebase issues discovered during work → file as refactoring tasks

### 4. REQUIRED ACTION

Before proceeding, do ONE of:
- Run `knecht add "observation"` to file a new task
- Run `knecht pain -t <task-id> -d "description"` to increment pain on existing task
- State explicitly: "Nothing to file because: [specific reason why zero friction occurred]"

## Commit Reminder

After reflection:
```
git add .knecht/tasks <your-changed-files>
git commit -m "task-N: Description"
```
