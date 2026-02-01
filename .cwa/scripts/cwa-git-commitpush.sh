#!/bin/bash
# Commit and push with auto-generated message
# Usage: cwa-git-commitpush.sh [--model MODEL] [-a]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODEL_ARG=""
STAGE_ALL=false

# Parse args
while [[ $# -gt 0 ]]; do
  case $1 in
    --model) MODEL_ARG="--model $2"; shift 2 ;;
    -a|--all) STAGE_ALL=true; shift ;;
    *) shift ;;
  esac
done

# Stage all if requested
if [ "$STAGE_ALL" = true ]; then
  git add -A
fi

# Generate message
MSG=$("$SCRIPT_DIR/cwa-git-msg.sh" $MODEL_ARG)
if [ $? -ne 0 ]; then
  exit 1
fi

echo "Commit message: $MSG"
git commit -m "$MSG"
echo "Committed successfully!"

echo "Pushing..."
git push
echo "Pushed successfully!"
