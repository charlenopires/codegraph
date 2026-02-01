# /generate-tasks

Generate tasks from a specification's acceptance criteria.

## Usage

```
/generate-tasks <spec-id>
```

## Steps

1. Get the spec details using MCP tool `cwa_get_spec` with the provided spec ID
2. Analyze each acceptance criterion in the spec
3. For each criterion, create a task using MCP tool `cwa_create_task`:
   - Title: Based on the criterion
   - Description: Include the criterion text and any context
   - Link to the spec ID
4. Report the created tasks to the user

## Example

```
/generate-tasks spec-123
```

This will create individual tasks from spec-123's acceptance criteria.
