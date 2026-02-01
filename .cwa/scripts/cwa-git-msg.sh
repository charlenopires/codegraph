#!/bin/bash
# Generate commit message using Ollama local LLM
# Usage: cwa-git-msg.sh [--model MODEL]

set -e

OLLAMA_URL="${OLLAMA_URL:-http://localhost:11434}"
MODEL="${OLLAMA_GEN_MODEL:-qwen2.5-coder:3b}"

# Parse args
while [[ $# -gt 0 ]]; do
  case $1 in
    --model) MODEL="$2"; shift 2 ;;
    *) shift ;;
  esac
done

# Get diff (prefer staged, fallback to unstaged)
DIFF=$(git diff --staged 2>/dev/null)
if [ -z "$DIFF" ]; then
  DIFF=$(git diff 2>/dev/null)
fi

if [ -z "$DIFF" ]; then
  echo "Error: No changes to commit" >&2
  exit 1
fi

# Truncate if too large (max ~4000 chars for context)
DIFF=$(echo "$DIFF" | head -c 4000)

# Build prompt
PROMPT="Generate a concise git commit message for the following changes.

Rules:
1. Use conventional commits format: type(scope): description
2. Types: feat, fix, docs, style, refactor, test, chore, perf, ci, build
3. Keep the first line under 72 characters
4. Be specific but concise
5. Return ONLY the commit message, nothing else

Changes:
$DIFF

Commit message:"

# Escape for JSON
PROMPT_JSON=$(echo "$PROMPT" | jq -Rs .)

# Call Ollama
RESPONSE=$(curl -s "$OLLAMA_URL/api/generate" \
  -H "Content-Type: application/json" \
  -d "{\"model\":\"$MODEL\",\"prompt\":$PROMPT_JSON,\"stream\":false}" 2>/dev/null)

if [ -z "$RESPONSE" ]; then
  echo "Error: Failed to connect to Ollama. Is it running? Try: cwa infra up" >&2
  exit 1
fi

# Extract response
MESSAGE=$(echo "$RESPONSE" | jq -r '.response // empty' | head -1 | tr -d '\n' | sed 's/^[`"]*//;s/[`"]*$//')

if [ -z "$MESSAGE" ]; then
  echo "Error: Failed to generate message. Check if model is pulled: cwa infra models" >&2
  exit 1
fi

echo "$MESSAGE"
