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

Before proceeding, do ONE of the following:

**Option A: Propose new tasks for user approval**

If you identified friction that warrants new tasks:
1. List each proposed task clearly with:
   - Title (what the task is)
   - Acceptance criteria (how to verify it's done)
   - Brief rationale (why this friction matters)
2. Ask the user: "Which of these tasks should I file? (Reply with numbers, 'all', or 'none')"
3. WAIT for user response before running any `knecht add` commands
4. Only run `knecht add "title" -a "criteria"` for tasks the user approves

**Option B: Increment pain on existing task**

If this matches an existing task:
- Run `knecht pain -t <task-id> -d "description"` to increment pain count

**Option C: Nothing to file**

- State explicitly: "Nothing to file because: [specific reason why zero friction occurred]"

**IMPORTANT**: Never run `knecht add` without user approval. Always propose first, then wait.

### 5. Upstream Feedback (for knecht improvements)

If you're working in a project OTHER than knecht itself, and you encountered friction with:
- knecht commands or output (hard to parse, missing information, confusing)
- knecht workflow or process (unclear what to do next, wrong guidance)
- knecht documentation or CLAUDE.md guidance

Then also invoke /upstream to file it in the knecht project.

The /upstream skill will:
1. Find the knecht project on your filesystem
2. Show you existing tasks that might be related
3. Help you decide: increment pain on existing task, or create new task
4. File the feedback with source context (which project you're in, what you were working on)

This ensures knecht improvements discovered while using knecht in other projects make it back to knecht's own development.

## Commit Reminder

After reflection:
```
git add .knecht/tasks <your-changed-files>
git commit -m "task-N: Description"
```
