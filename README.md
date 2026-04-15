# grimoire

Agent-ready CLI for your pattern library. Manages reusable code patterns stored as markdown files with YAML frontmatter.


![](/img/screen.png)

## Install

```bash
cargo install --path .
```

## Setup

```bash
export PATTERNS_DIR="/path/to/your/patterns"
```

## Usage

```bash
# List patterns
grimoire list
grimoire list --json
grimoire list --json --query "[].pattern"

# Search
grimoire search --category rust
grimoire search --text "error handling" --framework axum --json

# Get a pattern
grimoire get "my-pattern"
grimoire get "my-pattern" --json

# Create a pattern
grimoire create --name "my-pattern" --category rust --framework axum \
  --tags "web,api" --content "Pattern content here"

# Preview without writing
grimoire create ... --dry-run

# Browse interactively
grimoire browse

# Install agent skill file
grimoire setup-kiro
grimoire setup-claude
grimoire setup-oc
grimoire setup-codex
```

## Agent Notes

- `--json` on any command → structured JSON output
- `--query` → JMESPath filter (like AWS CLI)
- Errors → `{"error": "..."}` on stderr + exit 1 when `--json` is active
- `--dry-run` on `create` → preview without writing

## Pattern Format

```markdown
---
pattern: my-pattern
category: rust
framework: axum
tags: [web, api]
projects: [my-project]
---

Pattern content here.
```
