use anyhow::Result;
use crate::output::Output;

const SKILL_MD: &str = r##"---
name: coding-patterns
description: Search, retrieve, and create reusable code patterns from your personal pattern library
version: "0.1.1"
triggers:
  - pattern library
  - code patterns
  - reusable patterns
  - grimoire
  - best practices
---

# grimoire — Pattern Library CLI

`grimoire` manages reusable code patterns stored as markdown files with YAML
frontmatter. Requires `PATTERNS_DIR` env var pointing to your patterns directory.

## Commands

```bash
# List all patterns
grimoire list
grimoire list --json
grimoire list --json --query "[].pattern"

# Search
grimoire search --category rust
grimoire search --text "error handling" --json
grimoire search --framework axum --tag web --json

# Get full content
grimoire get "my-pattern"
grimoire get "my-pattern" --json

# Create a pattern
grimoire create --name "my-pattern" --category rust --framework axum \
  --tags "web,api" --content "Pattern content here"

# Preview without writing
grimoire create --name "test" --category rust --framework axum \
  --content "..." --dry-run

# Browse interactively (TUI)
grimoire browse
```

## Agent Notes

- Use `--json` for all structured output
- Use `--query` (JMESPath) to extract fields: `--query "[].pattern"`
- Errors are JSON on stderr when `--json` is active, exit code 1
- `--dry-run` on `create` shows what would be written, no side effects
- `PATTERNS_DIR` must be set

## Output Schemas

- `list --json` → `[{pattern, category, framework, tags}]`
- `search --json` → `[{pattern, category, framework, tags, preview}]`
- `get --json` → `{pattern, category, framework, tags, projects, content}`
- `create --json` → `{created: "<path>"}` or `{dry_run: true, path, content}`
- Error → `{error: "<message>"}` on stderr, exit 1
"##;

pub fn install(path: &str, dry_run: bool, out: &Output) -> Result<()> {
    if dry_run {
        if out.is_json() {
            out.print_json(&serde_json::json!({ "dry_run": true, "path": path, "content": SKILL_MD }))?;
        } else {
            println!("Would write: {path}\n{SKILL_MD}");
        }
        return Ok(());
    }

    let p = std::path::Path::new(path);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(p, SKILL_MD)?;

    if out.is_json() {
        out.print_json(&serde_json::json!({ "installed": path }))?;
    } else {
        println!("Installed: {path}");
    }
    Ok(())
}
