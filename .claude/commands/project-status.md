# /project-status

Show current project status including specs, tasks, and domain model.

## Usage

```
/project-status
```

## Steps

1. Call MCP tool `cwa_get_context_summary` to get overall status
2. Display:
   - Active specs with acceptance criteria progress
   - Task board summary (counts per column)
   - Current in-progress work
   - Recent observations/decisions
