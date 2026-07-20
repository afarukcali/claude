# talus-plugins

A Claude Code plugin marketplace for working with [Talus](https://talus.network) and [Nexus](https://github.com/Talus-Network/nexus-sdk).

Plugins live under [`plugins/`](plugins/). The marketplace manifest is at [`.claude-plugin/marketplace.json`](.claude-plugin/marketplace.json).

## Plugins

### [`talus`](plugins/talus/)

Helpers for building, wiring, publishing, running, scheduling, and troubleshooting Talus Agents and Nexus Tools. Every skill auto-detects paths in the current project rather than assuming a fixed layout, so they work in any repo.

- **`/talus:tool-new [--auto] [tool-name] [fqn-prefix] [description]`** — Scaffold a new [Nexus Tool](https://github.com/Talus-Network/nexus-tools) in Rust and walk the user through implementing it. Detects whether the current directory is a nexus-tools-style workspace (root `Cargo.toml` with `members = ["tools/*"]`) and adds a workspace member at `tools/<tool-name>/`, or otherwise scaffolds a fresh standalone crate. Reference templates are read from the latest upstream `Talus-Network/nexus-tools` at invocation time, or from the local clone if you are inside one — no frozen baked-in templates. Pass `--auto` to skip all confirmation gates and infer missing arguments.
- **`/talus:setup-nexus-wallet`** — Install Rust/Sui CLI/`nexus-cli` and create a funded testnet Sui wallet, from a completely empty environment.
- **`/talus:build-and-publish-agent`** — Wire a DAG out of registered Tools, configure a Skill's payment/schedule policy, publish it, and create/bind a Talus Agent. Locates `dag.json`/`skill.tap.json` automatically.
- **`/talus:run-nexus-agent`** — Execute a published Skill on demand and inspect/settle the resulting payment and execution.
- **`/talus:schedule-nexus-agent`** — Set up or manage recurring scheduled execution of a Skill.
- **`/talus:nexus-troubleshoot`** — Diagnose a `nexus` CLI error against a table of real, verified error strings from the CLI's own test suite.

## Install

In any Claude Code session (CLI or VS Code extension):

```text
/plugin marketplace add Talus-Network/claude
/plugin install talus@talus-plugins
```

To use a local checkout instead:

```text
/plugin marketplace add /path/to/this/repo
/plugin install talus@talus-plugins
```

## Try a single plugin without installing (CLI only)

```sh
claude --plugin-dir /path/to/this/repo/plugins/talus
```

Then in the session:

```text
/talus:tool-new weather-current xyz.example.weather "Fetches current weather conditions"
```

All arguments are optional; the skill prompts for whatever is missing. Add `--auto` to skip prompts entirely:

```text
/talus:tool-new --auto "Fetches current weather conditions"
```

## Status

Early. One plugin, six skills. More to come.
