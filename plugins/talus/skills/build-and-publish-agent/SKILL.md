---
name: build-and-publish-agent
description: >-
  Wire a DAG out of registered Nexus Tools, configure a Skill's payment and
  schedule policy, publish it, and create/bind a Talus Agent to it. Use
  when the user has one or more registered Tools and wants to turn them
  into a working, on-chain agent.
allowed-tools: Bash(find *) Bash(pwd)
---

# Wire the DAG, configure the Skill, publish, and bind the Agent

Requires: Tools already registered on-chain (see the `tool-new` skill) and
`nexus conf get` fully configured (see the `setup-nexus-wallet` skill).

## Phase 0 — locate the agent directory

This skill edits `<agent_dir>/dag.json` and `<agent_dir>/skill.tap.json`.
Locate `<agent_dir>` before doing anything else:

```sh
find . -maxdepth 4 -name dag.json
find . -maxdepth 4 -name skill.tap.json
```

- **Exactly one directory contains both files** → that's `<agent_dir>`,
  proceed.
- **Multiple candidates** → ask the user which agent they mean.
- **None found** → ask the user whether to scaffold one with
  `nexus tap scaffold --name <agent-name>` (creates an `agent/<agent-name>/`
  directory in the current project), or point you at the correct path.

## 1. Wire `<agent_dir>/dag.json`

One vertex per Tool call. `entry_ports` mark the vertices that accept the
DAG's initial input. `edges` connect a producing vertex's output port to a
consuming vertex's input port. Every `tool_fqn` must already be registered
on-chain. An empty/malformed `vertices` array fails validation with
"The DAG has no entry vertices or ports."

## 2. Configure `<agent_dir>/skill.tap.json`

- `name` — display name.
- `requirements.payment_policy` — `"UserFunded"` (caller pays per run) or
  an agent-funded policy with a max budget (the agent's vault pays — fund
  it in step 4).
- `requirements.schedule_policy.recurrence` — `"Once"` unless this Skill
  should also support recurring runs (see the `schedule-nexus-agent`
  skill).
- `requirements.fixed_tools` — leave empty unless a vertex must be pinned
  to one specific registered Tool object instead of being resolved by FQN
  at execution time.

Validate after **every** edit to either file:

```sh
nexus tap validate-skill --config <agent_dir>/skill.tap.json
```

Optionally sanity-check example inputs before publishing anything
on-chain:

```sh
nexus tap dry-run --config <agent_dir>/skill.tap.json
```

## 3. Publish

```sh
nexus tap publish-skill --config <agent_dir>/skill.tap.json --out <agent_dir>/artifact.json
```

## 4. Create/bind the Agent

New agent + first skill in one atomic step:

```sh
nexus tap bind --artifact <agent_dir>/artifact.json
```

Or, against an agent you already have:

```sh
nexus tap create-agent
nexus tap agent save --name <alias> --agent-id <OBJECT_ID>   # optional local alias
nexus tap register-skill --artifact <agent_dir>/artifact.json --agent-id <OBJECT_ID>
```

Changed the DAG/skill config after publishing once? Re-publish and push
the update to the same slot:

```sh
nexus tap publish-skill --config <agent_dir>/skill.tap.json --out <agent_dir>/artifact.json
nexus tap update-skill --artifact <agent_dir>/artifact.json --agent-id <OBJECT_ID> --skill-id <INDEX>
```

If the payment policy is agent-funded, fund the vault before executing:

```sh
nexus tap vault deposit --agent-id <OBJECT_ID> --amount <MIST>
nexus tap vault balance --agent-id <OBJECT_ID>
```

## When done

You have an `agent-id` and a `skill-id` (index, usually `0` for the first
skill on a fresh agent). Hand off to `run-nexus-agent` to execute it, or
`schedule-nexus-agent` for recurring runs.

## Ground rules

- Never guess a `nexus tap` flag — run `nexus tap <cmd> --help` first.
- Re-run `nexus tap validate-skill` after every `dag.json`/`skill.tap.json`
  edit — treat both as data, not code.
- Never fabricate an object ID (agent ID, DAG ID, ...) — get it from the
  actual command output.
