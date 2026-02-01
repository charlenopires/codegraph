# /cwa-commitpush

Commit and push with auto-generated message using local Ollama.

## Status
!git status --short

## Branch
!git branch --show-current

## Usage

```bash
# Commit and push:
cwa git commitpush

# Stage all, commit and push:
cwa git commitpush -a
```

## Notes

- Combines commit and push in one command
- Uses local Ollama for message generation
- Ensure you're on the correct branch before running
