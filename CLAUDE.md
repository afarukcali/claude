# CLAUDE.md

A Claude Code plugin marketplace. Currently hosts the [`talus`](plugins/talus/) plugin.

## Layout

- [`.claude-plugin/marketplace.json`](.claude-plugin/marketplace.json) — marketplace manifest
- [`plugins/<name>/.claude-plugin/plugin.json`](plugins/) — per-plugin manifest, alongside the plugin's `skills/` tree

## Adding a plugin

1. Create `plugins/<new-name>/.claude-plugin/plugin.json` and a `skills/` tree.
2. Append an entry to `marketplace.json` with `"source": "./plugins/<new-name>/"`.
3. `claude plugin validate --strict .` must pass before the change is done.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/). Use the plugin name as scope when changing a plugin (e.g. `feat(talus): ...`), `marketplace` for the manifest, or no scope for repo-wide changes.
