#!/bin/sh

commit_msg="${1-}"

if [ -z "$commit_msg" ]; then
  echo ""
  echo "ERROR: missing commit message subject line."
  echo ""
  echo "  Expected: scripts/validate-commit-msg.sh \"<subject>\""
  echo ""
  exit 1
fi

# Allow merge commits
if printf '%s\n' "$commit_msg" | grep -qE '^Merge '; then
  exit 0
fi

# Allowed types
types='feat|fix|docs|refactor|test|chore|build|ci|perf|style'

# Validate format: type(scope): description  or  type: description
if ! printf '%s\n' "$commit_msg" | grep -qE "^($types)(\\([a-z][a-z0-9-]*\\))?: .+"; then
  echo ""
  echo "ERROR: commit message does not follow conventional commits."
  echo ""
  echo "  Expected: <type>(<scope>): <description>"
  echo ""
  echo "  Types:  feat, fix, docs, refactor, test, chore, build, ci, perf, style"
  echo "  Scope is optional."
  echo ""
  echo "  Examples:"
  echo "    feat(cli): add new command"
  echo "    test: extract inline test modules"
  echo ""
  echo "  Your message:"
  echo "    $commit_msg"
  echo ""
  exit 1
fi

# Check subject line length (72 chars max)
length=$(printf '%s' "$commit_msg" | wc -c | tr -d ' ')
if [ "$length" -gt 72 ]; then
  echo ""
  echo "ERROR: commit subject line is $length characters (max 72)."
  echo ""
  echo "  Your message:"
  echo "    $commit_msg"
  echo ""
  exit 1
fi
