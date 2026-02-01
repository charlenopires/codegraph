# /cwa-commitmsg

Preview the auto-generated commit message (without committing).

## Diff
!git diff --staged 2>/dev/null | head -50 || git diff | head -50

## Generated Message
!cwa git msg 2>/dev/null || echo "Run 'cwa infra up' to start Ollama"

## Usage

```bash
cwa git msg                           # Show generated message
cwa git msg --model qwen2.5-coder:7b  # Use different model
```

## Notes

- Useful for previewing what the commit message would be
- Doesn't make any changes to your repository
- Uses same model as cwa-commit
