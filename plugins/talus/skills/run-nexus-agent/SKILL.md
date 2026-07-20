---
name: run-nexus-agent
description: >-
  Execute a published Talus Agent skill on demand and inspect/settle the
  resulting payment and execution. Use when the user wants to actually run
  their agent, check on a running execution, or resolve/settle/abort one.
---

# Execute and monitor a Talus Agent skill

Requires an `agent-id` and `skill-id` from the `build-and-publish-agent`
skill.

## Execute

```sh
nexus tap execute \
  --agent-id <OBJECT_ID> --skill-id <INDEX> \
  --input-json '{"input": {}}'
```

Match the JSON shape to the DAG's entry vertex/port names in its
`dag.json` (see the `build-and-publish-agent` skill's Phase 0 for how to
locate it).

## Inspect and settle payments

```sh
nexus tap payments list --agent-id <OBJECT_ID>
nexus tap payments show --payment-id <OBJECT_ID>
nexus tap payments wait --payment-id <OBJECT_ID> --timeout-secs 120

nexus tap payments resolve --execution-id <OBJECT_ID> [--alias <NAME>]
nexus tap payments refill  --execution-id <OBJECT_ID> --amount <MIST> [--alias <NAME>]
```

## Settle or abort the execution itself

```sh
nexus tap execution settle --execution-id <OBJECT_ID> --walk-index 0
nexus tap execution abort  --execution-id <OBJECT_ID>
nexus tap execution resolve-expired-walk --execution-id <OBJECT_ID> --walk-index 0
```

## Check what a skill currently requires

```sh
nexus tap requirements --agent-id <OBJECT_ID> --skill-id <INDEX>
```

Fetches live payment/schedule requirements straight from the registry —
use this instead of assuming `skill.tap.json` still matches on-chain state
after any `update-skill`.

## Ground rules

- Never guess a `nexus tap` flag — run `nexus tap <cmd> --help` first.
- Never fabricate an execution/payment/object ID — copy it from the actual
  command output that created it.
- If execution fails, check the `nexus-troubleshoot` skill before
  re-trying blindly.
