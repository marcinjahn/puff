#!/usr/bin/env bash
set -euo pipefail

# Records an asciinema demo of puff.
#
# Prerequisites: puff, git, asciinema, jq (optional: agg to render to GIF)
# Usage: ./demo/record.sh
# Output: demo/puff-demo.cast (and demo/puff-demo.gif if agg is installed)
#
# Tip: resize your terminal to ~100x25 before running for best results.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEMO_BASE="$HOME/puff-demo"
PROJECT_DIR="$DEMO_BASE/my-app"
CAST_FILE="$SCRIPT_DIR/puff-demo.cast"

# ── Setup (not recorded) ─────────────────────────────────────────────

cleanup() {
  puff project forget my-app -d -y 2>/dev/null || true
  rm -rf "$DEMO_BASE"
}

# Clean any leftover state from a previous run
cleanup

mkdir -p "$PROJECT_DIR"
cd "$PROJECT_DIR"

git init -q
git checkout -q -b main

mkdir -p src
cat >src/app.py <<'EOF'
import os
from dotenv import load_dotenv

load_dotenv()
db = os.getenv("DATABASE_URL")
EOF

cat >.gitignore <<'EOF'
__pycache__/
*.pyc
EOF

git add -A
git commit -q -m "Initial commit"

# Create .env AFTER the commit so it's never tracked by git
cat >.env <<'EOF'
DATABASE_URL=postgres://admin:s3cret@db.internal:5432/myapp
STRIPE_KEY=sk_live_4eC39HqLyjWDarjtT1zdp7dc
JWT_SECRET=kPmR9x2vQ5nL8wBcA3yD
REDIS_URL=redis://:token@cache.internal:6379/0
EOF

echo "Setup complete. Starting recording..."
echo

# ── Record ────────────────────────────────────────────────────────────

DEMO_INNER=$(mktemp)
cat >"$DEMO_INNER" <<'INNEREOF'
#!/usr/bin/env bash

DEMO_BASE="$HOME/puff-demo"

type_cmd() {
    local cmd="$1"
    local pause_after="${2:-1}"
    local dir
    dir=$(basename "$(pwd)")
    printf "\033[1;34m%s\033[0m \033[1;32m$\033[0m " "$dir"
    for ((i=0; i<${#cmd}; i++)); do
        printf "%s" "${cmd:$i:1}"
        sleep 0.04
    done
    sleep 0.4
    echo
    eval "$cmd" 2>&1 || true
    sleep "$pause_after"
}

comment() {
    local pause_before="${2:-1.5}"
    sleep "$pause_before"
    echo
    printf "\033[0;90m# %s\033[0m\n" "$1"
    sleep 0.8
}

clear
cd "$DEMO_BASE/my-app"

comment "A project with a .env file kept out of git" 0.5
type_cmd "cat .env" 2

comment "Initialize puff for this project"
type_cmd "puff init -n my-app"

comment "Let puff manage .env and add it to .gitignore"
type_cmd "puff add .env -g"

comment "The file is now a symlink to puff's central storage"
type_cmd "ls -la .env" 2.5

comment "Overview of what puff manages in this project"
type_cmd "puff status" 2

comment "Meanwhile, on another machine..." 2
comment "Puff's data directory was synced here (private git repo, rsync, etc.)" 0.3
type_cmd "git clone -q . ../my-app-clone"
type_cmd "cd ../my-app-clone" 0.5

comment "The repo is here, but .env is not. It was never committed." 1
type_cmd "cat .env" 3

comment "Puff still has it. Just link it in."
type_cmd "puff link my-app"
type_cmd "cat .env" 3
INNEREOF
chmod +x "$DEMO_INNER"

asciinema rec "$CAST_FILE" -c "$DEMO_INNER" --overwrite

rm -f "$DEMO_INNER"

# ── Cleanup ───────────────────────────────────────────────────────────

cleanup

echo
echo "Recording saved to: $CAST_FILE"

# Render to GIF if agg is available
if command -v agg &>/dev/null; then
  GIF_FILE="$SCRIPT_DIR/puff-demo.gif"
  agg "$CAST_FILE" "$GIF_FILE"
  echo "GIF rendered to: $GIF_FILE"
else
  echo "Install agg (https://github.com/asciinema/agg) to render a GIF."
fi
