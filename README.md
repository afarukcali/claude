# talus-plugins

A Claude Code plugin marketplace for working with [Talus](https://talus.network) and [Nexus](https://github.com/Talus-Network/nexus-sdk).

Plugins live under [`plugins/`](plugins/). The marketplace manifest is at [`.claude-plugin/marketplace.json`](.claude-plugin/marketplace.json).

## Plugins

### [`talus`](plugins/talus/)

Helpers for building Nexus Tools and Talus-related artifacts.

- **`/talus:tool-new [tool-name] [fqn-prefix]`** — Scaffold a new [Nexus Tool](https://github.com/Talus-Network/nexus-tools) in Rust and walk the user through implementing it. Detects whether the current directory is a nexus-tools-style workspace (root `Cargo.toml` with `members = ["tools/*"]`) and adds a workspace member at `tools/<tool-name>/`, or otherwise scaffolds a fresh standalone crate. Reference templates are read from the latest upstream `Talus-Network/nexus-tools` at invocation time, or from the local clone if you are inside one — no frozen baked-in templates.

## Install via `/plugin`

In any Claude Code session (CLI or VS Code extension):

```text
/plugin marketplace add /path/to/this/repo
/plugin install talus@talus-plugins
```

Once published to a remote, the marketplace argument becomes a git URL or `owner/repo` shorthand:

```text
/plugin marketplace add <owner>/<repo>
```

## Try a single plugin without installing (CLI only)

```sh
claude --plugin-dir /path/to/this/repo/plugins/talus
```

Then in the session:

```text
/talus:tool-new weather-current xyz.example.weather
```

Both positional arguments are optional; the skill prompts for whatever is missing.

## Status

Early. One plugin, one skill. More to come.
