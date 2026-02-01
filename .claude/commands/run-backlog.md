# /run-backlog

Plan and execute all tasks in the backlog.

## Usage

```
/run-backlog [--dry-run]
```

## Steps

1. Get the current board state using MCP tool `cwa_get_context_summary`
2. List all tasks with status "backlog" or "todo"
3. For each task in order:
   a. Move the task to "in_progress" using `cwa_update_task_status`
   b. Get task details with `cwa_get_current_task`
   c. Plan the implementation approach
   d. Execute the implementation
   e. Verify the task is complete
   f. Move to "review" then "done" as appropriate
4. Report progress after each task

## Options

- `--dry-run`: Only show which tasks would be executed without making changes

## Notes

- Respects WIP limits (only 1 task in_progress at a time)
- Will pause and ask for input if blocked or uncertain
- Uses the project's domain model and specs for context
