---
name: schedule-nexus-agent
description: >-
  Set up or manage recurring/scheduled execution of a Talus Agent skill
  (as opposed to one-off on-demand runs). Use when the user wants their
  agent to run automatically on a schedule, or to pause/resume/cancel an
  existing scheduled task.
---

# Schedule recurring runs of a Talus Agent skill

Requires an `agent-id` and `skill-id` from the `build-and-publish-agent`
skill.

This wraps the lower-level `nexus scheduler` primitive (`nexus scheduler
--help` for the raw, non-TAP form) with TAP payment handling built in.

## Create a scheduled task

```sh
nexus tap schedule-task \
  --agent-id <OBJECT_ID> --skill-id <INDEX> \
  --input-json '{"input": {}}' \
  --generator queue \
  --payment-source user-funded \
  --prepay-amount <MIST> \
  --occurrence-budget <MIST_PER_RUN>
```

Key decisions:

- **`--payment-source`**: `user-funded` (you fund it, and may set
  `--refund-recipient`) or `agent-funded` (draws from the agent's vault —
  `--refund-recipient` is invalid here and is rejected locally, before any
  transaction is built).
- **`--prepay-amount` / `--occurrence-budget`**: both are **required and
  default to `0`** if omitted — a `0` prepay creates the task but funds no
  occurrence, so don't forget to set real values once the user actually
  wants it to run.
- **`--generator`**: `queue` (default) or `periodic`. Periodic tasks
  cannot enqueue an initial occurrence via this command — configure their
  schedule with `nexus scheduler periodic set` instead once the task
  exists.
- **`--dag-id`**: required if the skill is runtime-DAG-selected (as
  opposed to pinned to one published DAG) — the CLI rejects the command
  locally with "provide --dag-id" otherwise. For a pinned skill it's
  optional, and if given must match the pinned DAG.

## Manage an existing scheduled task

```sh
nexus tap scheduled-task pause  --task-id <OBJECT_ID> --agent-id <OBJECT_ID>
nexus tap scheduled-task resume --task-id <OBJECT_ID> --agent-id <OBJECT_ID>
nexus tap scheduled-task cancel --task-id <OBJECT_ID> --agent-id <OBJECT_ID>
```

## Ground rules

- Never guess a `nexus tap schedule-task` / `nexus scheduler` flag — run
  `--help` first.
- Always ask what `--prepay-amount` / `--occurrence-budget` should
  actually be rather than silently leaving them at `0` — a task that
  never funds an occurrence looks "created successfully" but never runs.
