# /next-task

Get and start working on the next available task.

## Usage

```
/next-task
```

## Steps

1. Call MCP tool `cwa_get_next_steps` to identify next available work
2. If a task is available:
   a. Move it to "in_progress" using `cwa_update_task_status`
   b. Display task details and context
   c. Begin implementation planning
3. If no tasks available, suggest creating new tasks or specs
