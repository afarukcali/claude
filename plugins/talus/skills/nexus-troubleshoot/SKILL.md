---
name: nexus-troubleshoot
description: >-
  Diagnose a `nexus` CLI error encountered while building or running a
  Talus Agent. Use whenever a `nexus tool`/`nexus tap`/`nexus scheduler`
  command fails and the fix isn't obvious from the error text alone.
---

# Troubleshoot a `nexus` CLI error

The table below is sourced directly from the CLI's own test suite (real,
verified error strings) — match the failing command's error text against
it before guessing a fix.

| Message contains | Cause | Fix |
| --- | --- | --- |
| `Sui RPC URL is not configured` | `nexus conf set --sui.rpc-url ...` was never run | Run the `setup-nexus-wallet` skill |
| `No Talus agent alias` | `--alias <NAME>` doesn't exist locally | `nexus tap agent list`, or use `--agent-id` directly |
| `invalid payment-source hex` | `--payment-source-hex` isn't valid hex | Pass real hex, or omit the flag |
| `The DAG has no entry vertices or ports.` | `dag.json`'s `vertices` array is empty/malformed | Every DAG needs ≥1 vertex with `entry_ports` |
| `has no source file declaring \`module <pkg>::...;\`` | `tap/Move.toml`'s `[package].name` doesn't match any Move source module | Keep the package name and `module` declaration in sync |
| `[addresses]` | `tap/Move.toml` still has an old-style `[addresses]` table | Delete it — new-style packages resolve deps via `[environments]` only |
| `[package].version` / `missing [package].name` | Incomplete `Move.toml` `[package]` table | Needs `name`, `version`, `edition = "2024"` |
| `edition = "2024.beta"` | Old beta edition | Change to `edition = "2024"` |
| `[environments]` (missing) | No `[environments]` table | Add ≥1 network → chain-id row |
| `--refund-recipient is only valid with --payment-source user-funded` | `--refund-recipient` passed with `--payment-source agent-funded` | Drop the flag, or switch payment source |
| `does not match pinned DAG` | `--dag-id` on a pinned Skill doesn't match its real DAG | Omit `--dag-id`, or pass the correct one |
| `active TAP skill ... is runtime-DAG selected; provide --dag-id` | Scheduling/executing a runtime-selected Skill without `--dag-id` | Pass `--dag-id` explicitly |

## If the error isn't in the table

1. Run the failing command with `--help` to confirm every flag is spelled
   and typed correctly for the installed CLI version.
2. Re-run `nexus tap validate-skill --config <agent_dir>/skill.tap.json`
   — most `dag.json`/`skill.tap.json` authoring mistakes surface here
   before they ever reach the network.
3. Run `nexus conf get` and confirm `sui.pk`, `sui.rpc_url`, and `nexus`
   (objects) are all populated and point at the intended network.
4. If it's a genuine on-chain/object-not-found error, double check every
   object ID (agent, DAG, execution, payment) was copied from real command
   output, not guessed or reused from a different network.

## Ground rules

- Don't paper over an error by inventing a plausible-sounding object ID,
  key, or `objects.toml` path — ask the user for the real value instead.
