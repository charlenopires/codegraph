# /cwa-commit

Commit with auto-generated message using local Ollama (saves Claude tokens).

## Status
!git status --short

## Diff Preview
!git diff --staged 2>/dev/null | head -30 || git diff | head -30

## Usage

```bash
# Generate message and commit:
cwa git commit

# Stage all changes and commit:
cwa git commit -a

# Edit message before committing:
cwa git commit -e
```

## Notes

- Uses local Ollama model (qwen2.5-coder:3b by default)
- Requires `cwa infra up` to be running
- Change model with: `cwa git commit --model qwen2.5-coder:7b`
