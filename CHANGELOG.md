# Changelog

## [0.1.1] - 2026-04-14

### Changed
- Skill installation now uses `coding-patterns` as the skill name and directory
  (was `grimoire`) for better agent discoverability

## [0.1.0] - 2026-04-14

### Added
- `list` — list all patterns, with `--json` and `--query` (JMESPath) support
- `search` — filter by `--text`, `--category`, `--framework`, `--tag`
- `get` — retrieve full pattern content by name
- `create` — create a new pattern with `--dry-run` support
- `browse` — interactive Ratatui TUI browser
- `setup-kiro`, `setup-oc`, `setup-codex`, `setup-claude` — install embedded `SKILL.md`
- `--json` global flag for structured output on all commands
- `--query` global flag for JMESPath filtering of JSON output
- JSON errors on stderr + exit code 1 when `--json` is active
