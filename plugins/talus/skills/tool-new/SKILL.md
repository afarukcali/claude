---
name: tool-new
description: Scaffold a new Nexus Tool in Rust and implement it end-to-end based on the user's description. Detects context: inside the Talus-Network/nexus-tools repo (or a fork) adds a member at offchain/tools/<name>/ and generates the extra files the CI pipeline requires (tools.json, build.rs, [[bin]], version-threaded FQN); in any other Cargo workspace with members = ["tools/*"] adds a member at tools/<name>/; otherwise scaffolds a standalone crate. Fetches the latest reference patterns from upstream Talus-Network/nexus-tools at invocation time (or reads them locally when inside a clone or fork) — no baked-in templates; the NexusTool trait signatures are taken from upstream math/src/i64/add.rs at every invocation. After the scaffold compiles, replaces the placeholder Input/Output and invoke() logic with the real implementation — for well-known APIs (OpenAI, Anthropic, etc.) it applies the actual request/response shapes directly rather than asking the user to design the schema. Use when the user asks to "create a Nexus Tool", "scaffold a Nexus Tool", "build a new Nexus Tool", "new Talus tool", or similar.
argument-hint: "[--auto] [tool-name] [fqn-prefix] [description]"
allowed-tools: Bash(pwd) Bash(command -v *) Bash(head *) Bash(find *) Bash(chmod +x *) Bash(bash -n *) Bash(grep *) Bash(sed -n *) Bash(gh api *)
---

# `tool-new` — scaffold a new Nexus Tool in Rust

A Nexus Tool is an HTTPS service that implements the `NexusTool` trait from the `nexus-toolkit` crate. The canonical reference is [Talus-Network/nexus-tools](https://github.com/Talus-Network/nexus-tools). The authoritative development guidelines live in [docs/tool-development.md](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/tool-development.md) and [docs/toolkit-rust.md](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/toolkit-rust.md) in the SDK repo.

This skill scaffolds a working skeleton, then implements the real Input/Output and invoke() logic based on the user's description. The `NexusTool` trait signatures used in the generated code come from upstream `tools/math/src/i64/add.rs` (fetched at every invocation) — never from a baked-in copy of the trait.

## Arguments

Supported invocation forms:

```
/talus:tool-new my-tool xyz.acme.weather "Fetches current weather"
/talus:tool-new --auto "Fetches current weather"
/talus:tool-new --auto Fetches current weather
/talus:tool-new "Fetches current weather" --auto
/talus:tool-new Fetches current weather --auto
```

**Parsing rules.** Strip `--auto` or `--yes` wherever it appears — its presence sets auto mode (see below). From the remaining tokens:

- A token matching the reverse-domain pattern (contains dots, e.g. `xyz.acme.weather`) → `fqn_prefix`
- **Without `--auto`:** the first remaining token matching `^[a-z][a-z0-9-]*$` → `tool_name` (single-word names like `calculator` are valid here).
- **With `--auto`:** only a token matching `^[a-z][a-z0-9]*(-[a-z0-9]+)+$` (hyphenated, e.g. `current-weather`) → `tool_name`. Single lowercase words are treated as description to avoid misidentifying prose words (e.g. "fetches" in "Fetches weather data") as the tool name.
- Everything else, joined → `description`

**Named arguments:**

- `tool_name` — kebab-case crate/dir name (e.g. `weather-current`). In auto mode, if absent: derive from the significant words of the description (drop leading verbs like "fetches"/"gets", convert nouns to kebab-case — e.g. "Fetches current weather" → `current-weather`). Print the derived name; do not wait for confirmation. Without auto mode, ask.
- `fqn_prefix` — reverse-domain namespace prefix **without trailing dot** (e.g. `xyz.taluslabs.weather`). In auto mode, if absent: scan existing tools in the workspace for `fqn!(...)` calls and extract the common prefix; if found, use it; if ambiguous or not found, use `com.example` and warn the user to update it before publishing. **Without auto mode, never invent or default this — always ask.** The final FQN is `<fqn_prefix>.<tool_name_snake>@1`.
- `description` — one-line description of what the tool does. Required in all modes; ask if missing even in auto mode.

**Auto mode** (`--auto` / `--yes`) **or all three arguments provided**: skip every confirmation gate throughout all phases — placement, FQN preview, overwrite check. Infer anything not explicitly provided using the rules above.

## Context (computed at invocation)

- Working directory: !`pwd`

Only `pwd` runs as a pre-injection — every other detection is done in Phase 1 below, because the alternatives ("does X exist?") need to handle the not-present case without aborting the pre-processor on a non-zero exit. The Bash patterns the procedure uses are pre-authorized via the `allowed-tools` frontmatter, so they won't trigger permission prompts.

## Procedure

### Phase 1 — Detect context and confirm placement

Run these checks via the Bash tool (each is in the skill's allowed-tools allowlist — no permission prompt):

1. `command -v cargo` — empty stdout means `cargo` is not on PATH. Warn the user; the scaffold can still be written, but Phase 6 verification won't run.
2. `command -v gh` — empty stdout means we'll fall back to WebFetch in Phase 3 instead of `gh api`.
3. `command -v just` — empty stdout means `just tools::check` can't be used later (use `cargo +stable check` directly).
4. `find . -maxdepth 1 -name Cargo.toml -type f` — non-empty: a `Cargo.toml` is present at the current directory root.
5. `find . -maxdepth 2 -path ./offchain/Cargo.toml -type f` — non-empty: a `Cargo.toml` exists one level down at `offchain/`. This is the Rust workspace root when working from the nexus-tools repo root.
6. If step 4 was non-empty, `head -n 50 Cargo.toml` — look for `members = ["tools/*"]`.
7. If step 5 was non-empty, `head -n 50 offchain/Cargo.toml` — look for `members = ["tools/*"]`.
8. If step 5 was non-empty, `find . -maxdepth 2 -path ./offchain/tools -type d` — non-empty: the `offchain/tools/` directory exists. Combined with step 7 containing `members = ["tools/*"]`, this confirms a nexus-tools-style workspace at the repo root.
9. If step 4 was non-empty and step 6 contains `members = ["tools/*"]`, `find . -maxdepth 2 -path ./tools -type d` — confirms the `tools/` directory exists (working from inside `offchain/`).

Decide placement and mode from the results:

- **Step 7 contains `members = ["tools/*"]` AND step 8 non-empty (nexus-tools, working from repo root):** The Rust workspace is at `offchain/`; place the new tool at `offchain/tools/$tool_name/`. All `cargo` commands in later phases must be run from `offchain/`. The `tools/.just` file is at `offchain/tools/.just`. This is **nexus-tools mode** — apply the additional CI requirements in Phase 4.5. Prefer reading templates locally (Phase 3, step 1 — Local read) over fetching from upstream.
- **Step 6 contains `members = ["tools/*"]` AND step 9 non-empty (nexus-tools, working from inside `offchain/`):** Place the new tool at `tools/$tool_name/`. Run `cargo` commands from the current directory. The `tools/.just` file is at `tools/.just`. This is **nexus-tools mode** — apply Phase 4.5. Prefer reading templates locally.
- **Step 4 non-empty AND step 6 contains `members = ["tools/*"]` AND steps 7–9 empty:** Generic workspace (not nexus-tools, but same layout). Place at `tools/$tool_name/`. Standard scaffold, no Phase 4.5 extras.
- **Step 4 non-empty AND step 6 does NOT contain `members = ["tools/*"]`:** Unrelated Cargo project. In auto mode, default to standalone. Otherwise ask the user: (a) treat as standalone (tool goes in a subdirectory), or (b) abort.
- **Step 5 non-empty AND step 7 does NOT contain `members = ["tools/*"]`:** `offchain/Cargo.toml` exists but is not a tools workspace. Fall through: check step 4; if that also fails, use standalone mode. In auto mode, proceed with standalone. Otherwise ask the user to confirm before proceeding.
- **Steps 4, 5 both empty:** Standalone mode. Default to `$tool_name/` under the current directory.

In auto mode, skip the placement confirmation — proceed with the detected mode without asking. Never overwrite an existing target directory without explicit user confirmation (this check is not bypassed by auto mode).

### Phase 2 — Collect inputs

> **AUTO MODE (`--auto` / `--yes`) or all three arguments provided:** Do NOT ask any questions in this phase. Apply the rules below silently, print a one-line summary of what was resolved, and proceed immediately to Phase 3. The user said they do not want to be asked.

Resolve each value:

- **`tool_name`** — used for crate name (`[package].name`), directory name, and the tail of the FQN. Validate: `^[a-z][a-z0-9-]*$`. Derive `tool_name_snake` = hyphens → underscores; `tool_name_pascal` = PascalCase of the snake form.
  - *Interactive:* ask if not provided.
  - *Auto mode:* if not provided, derive from the description — drop leading verbs ("fetches", "gets", "creates", "calculates", "sends"), convert the remaining significant words to kebab-case (e.g. "Fetches current weather" → `current-weather`, "OpenAI completion API" → `openai-completion`). Print the derived name. Do not ask for confirmation.

- **`fqn_prefix`** — reverse-domain prefix, no trailing dot. Validate: `^[a-z][a-z0-9.-]*[a-z0-9]$`.
  - *Interactive:* ask if not provided. **Never invent or default this without asking.**
  - *Auto mode:* if not provided, scan existing tool source files for `fqn!(` calls (`grep -r 'fqn!(' . --include='*.rs'`) and extract the common prefix (everything before the last `.`-separated segment and `@`). If a single consistent prefix is found, use it. If ambiguous or not found, use `com.example` and print a warning that the user must update it before publishing. Print the resolved prefix. Do not ask for confirmation.

- **`description`** — one-line description; used in both `Cargo.toml` `[package].description` and `impl NexusTool::description`. Ask if missing in both interactive and auto mode — there is no reasonable default. **Validate: must not contain `"` (double quote) or `\` (backslash).** The description is substituted unescaped into a Rust string literal (`fn description() -> &'static str { "..." }`) and a TOML string (`[package].description = "..."`) — either character breaks the generated file. If the user-provided description contains one, ask them to rephrase (e.g. replace `Says "hello"` with `Says hello (with quotes)`); in auto mode print the rejection reason and stop rather than silently mangling.

- **FQN preview** — show the computed final FQN (`<fqn_prefix>.<tool_name_snake>@1`).
  - *Interactive:* ask the user to confirm before proceeding. In nexus-tools mode note that the literal `@1` is the local-development default; CI threads the real version via `build.rs`.
  - *Auto mode:* print it and proceed immediately.

### Phase 3 — Fetch reference templates

The canonical reference files live under `offchain/` in the nexus-tools repo. Adjust local read paths depending on working depth (repo root vs. inside `offchain/`).

Files to read (two groups — note the difference):

- **Repo-root files** (path is the same regardless of working depth):
  - `README.md` — contains the "Adding a new tool" section; read in full
- **`offchain/`-rooted files** (use `offchain/<path>` from repo root; strip `offchain/` when already inside `offchain/`):
  - `offchain/Cargo.toml` (workspace root)
  - `offchain/rust-toolchain.toml`
  - `offchain/tools/.just`
  - `offchain/tools/math/Cargo.toml`
  - `offchain/tools/math/build.rs`
  - `offchain/tools/math/tools.json`
  - `offchain/tools/math/src/main.rs`
  - `offchain/tools/math/src/i64/add.rs`
  - `offchain/tools/math/README.md`

In order of preference:

1. **Local read** if nexus-tools mode was detected. Read using the paths above (adjusted for working depth).

2. **`gh` fetch** if `gh` is on PATH. Use each path as listed above (repo-root files without prefix, `offchain/`-rooted files with `offchain/` prefix):
   `gh api repos/Talus-Network/nexus-tools/contents/<path> -H "Accept: application/vnd.github.raw"`

3. **WebFetch fallback** if neither: fetch `https://raw.githubusercontent.com/Talus-Network/nexus-tools/main/<path>` for each.

If none of these can produce the templates (offline, no clone, no `gh`, no WebFetch), stop and tell the user the skill needs network access or a local clone to proceed. Never fabricate templates.

### Phase 4 — Generate files

Use the Write tool for new files (not Edit). The `Cargo.toml` is generated inline based on upstream patterns read in Phase 3; the three source files (`main.rs`, `<tool_name_snake>.rs`, `README.md`) come from templates in this skill's own repo, fetched and substituted as described below.

#### `<target-dir>/Cargo.toml`

- **Workspace member mode (nexus-tools or generic):** use `*.workspace = true` for every inherited field (edition, version, repository, homepage, license, readme, authors, keywords, categories) — same shape as upstream `tools/math/Cargo.toml`. Dependencies use `*.workspace = true` for `schemars`, `serde`, `tokio`, `nexus-toolkit`, `nexus-sdk`, **`log`** (the template uses `log::*` macros directly, so `log` must be a direct dep — `nexus_toolkit::*` does not re-export the `log` macros).
  - **nexus-tools mode only:** also add `[build-dependencies]` with `serde_json.workspace = true` and `toml = "0.8"` — required by `build.rs`, which parses both `tools.json` and `Cargo.toml` at compile time. For `[[bin]]` and the FQN version threading, see Phase 4.5.
- **Standalone mode:** use explicit values from the upstream workspace `[workspace.package]` and `[workspace.dependencies]` sections. Pin `nexus-sdk` and `nexus-toolkit` to the same `git` + `tag` upstream uses (read these from the fetched workspace `Cargo.toml`, never hardcode). Include `log = "0.4"` (or the version pinned by the upstream workspace) as a direct dep — required by the template's `log::*` macros.

#### `<target-dir>/src/main.rs`, `<target-dir>/src/<tool_name_snake>.rs`, `<target-dir>/README.md`

These three files come from templates in this skill's own repo. Fetch each from `Talus-Network/claude` using the same preference order as Phase 3 (`gh api` → WebFetch):

| Template | Target path |
|---|---|
| `plugins/talus/skills/tool-new/templates/main.rs` | `<target-dir>/src/main.rs` |
| `plugins/talus/skills/tool-new/templates/tool.rs` | `<target-dir>/src/<tool_name_snake>.rs` |
| `plugins/talus/skills/tool-new/templates/README.md` | `<target-dir>/README.md` |

Each template contains `__PLACEHOLDER__` markers. Substitute every marker before writing:

| Placeholder | Value |
|---|---|
| `__TOOL_NAME_SNAKE__` | `tool_name` with hyphens → underscores |
| `__TOOL_NAME_PASCAL__` | PascalCase of `__TOOL_NAME_SNAKE__` |
| `__FQN_PREFIX__` | `fqn_prefix` from Phase 2 |
| `__DESCRIPTION__` | `description` from Phase 2 |
| `__COMPUTED_FQN__` | `<fqn_prefix>.<tool_name_snake>@1` |

After writing each file, confirm no markers remain. If any are present, the substitution missed something — fix before continuing:

```
grep -l '__[A-Z_]*__' <target-dir>/src/main.rs <target-dir>/src/<tool_name_snake>.rs <target-dir>/README.md
```

Empty output = success.

**Verify trait signatures against upstream (mandatory).**

The template carries a working `impl NexusTool for ...` block, but the `NexusTool` trait can evolve in upstream. After substitution, compare every method signature in the generated `src/<tool_name_snake>.rs`'s `impl NexusTool` block against the same methods in the math reference tool fetched in Phase 3 (`offchain/tools/math/src/i64/add.rs`). For each method (`new`, `fqn`, `path`, `description`, `health`, `invoke`):

- The argument list, receiver (`self`, `&self`, none), return type, and `async`-ness must match math/add.rs exactly.
- If our template's signature differs from upstream, **upstream wins** — update the generated file to match. The template may lag.

This means math/add.rs is the source of truth for trait method shapes at every invocation, not anything baked into this skill.

**Mode-specific edits after substitution:**

- **nexus-tools mode:** in `src/<tool_name_snake>.rs`, switch the `fqn!()` form. The template's `fn fqn()` body has four lines: a `// Generic workspace / standalone:` comment, the active `fqn!("...@1")` call, a `// nexus-tools mode — uncomment...` comment, and a commented-out `// fqn!(concat!(...))` line. Delete the first three lines entirely and uncomment the last one, leaving only the `fqn!(concat!(...))` call. The version is then threaded from `build.rs` (see Phase 4.5).
- **No secrets needed:** if analysis of the tool description finds no plausible required env var, empty the body of `validate_config` and delete the `EXAMPLE_API_KEY` static, the `example_api_key()` accessor, and `load_required` (if nothing else calls it). Do not delete `validate_config` itself — `main.rs` calls it unconditionally.
- **Additional secrets:** to add more env vars, declare another `static <NAME>: OnceLock<String>` and accessor, and add another `.set(load_required("<NAME>"))` line to `validate_config`. Follow the EXAMPLE_API_KEY pattern exactly.

### Phase 4.5 — nexus-tools additional files (nexus-tools mode only)

Skip this phase entirely in generic workspace mode and standalone mode.

**This phase is driven by the live README fetched in Phase 3, not by instructions baked into this skill.** The upstream repo may evolve — new files, changed structure, additional steps. The README is the authoritative source; this skill's description below is context to help interpret it, not a substitute for reading it.

1. In the README fetched in Phase 3, locate the section whose heading is "Adding a new tool" (or the closest equivalent if the heading has changed).
2. **If the section is not found:** stop and tell the user that the upstream README no longer contains an "Adding a new tool" section. Ask them to check [https://github.com/Talus-Network/nexus-tools](https://github.com/Talus-Network/nexus-tools) and confirm how to proceed before continuing.
3. **If the section is found:** read it in full and follow every step it prescribes, using the reference tool (`offchain/tools/math/`) as the concrete implementation to copy from and adapt. Do not skip any step listed there.

For context, at the time this skill was last updated the section prescribed four things — but treat this as background knowledge for interpretation only, not as the current source of truth:

- A `tools.json` at the crate root. The file must contain at minimum a `"command"` field equal to the binary/crate name (this is what `build.rs` reads and validates). It also carries `"tool_name"` and an `"environment"` map. Copy from the reference tool and substitute the name — do not invent the structure. If the tool reads runtime secrets via `std::env::var`, add their keys to `"environment"` as entries; actual values are injected by the deployment pipeline, never hardcoded.
- A `build.rs` copied verbatim from an existing tool; do not invent it. It reads `tools.json["command"]`, asserts it matches the `[[bin]]` name in `Cargo.toml`, and emits `TOOL_FQN_VERSION` as a Cargo env var from the Docker build arg (defaulting to `"1"` locally). It requires `[build-dependencies]` with `serde_json` and `toml` — these are already added to `Cargo.toml` in Phase 4.
- A `[[bin]]` section in `Cargo.toml` with `name` equal to the crate name, enforced by `build.rs` at compile time.
- `fqn!()` using `concat!("...", env!("TOOL_FQN_VERSION"))` so the content version flows from the CI build arg.

If the live README section contains steps beyond or different from this list, follow the README — it is newer than this skill. Flag any discrepancy to the user so they are aware.

### Phase 5 — Wire into workspace `tools/.just` (workspace mode only)

If the target is a workspace member, edit `tools/.just` (at `offchain/tools/.just` when working from the repo root, or `tools/.just` when working from inside `offchain/`) and append the new package name to each of these recipes, preserving existing ordering and indentation:

- `build` → `cargo +stable build --package <tool_name> --release`
- `check` → `cargo +stable check --package <tool_name>`
- `test` → `cargo +stable test --package <tool_name>`
- `fmt-check` → `cargo +"$nightly" fmt --package <tool_name> --check`
- `clippy` → `cargo +stable clippy --package <tool_name>`

The workspace root `Cargo.toml` does not need editing — `members = ["tools/*"]` discovers the new member automatically.

### Phase 6 — Verify the scaffold compiles

- **nexus-tools mode, working from repo root:** `cd offchain && cargo +stable check --package <tool_name>` (or `just tools::check` from `offchain/` if `just` is on PATH)
- **nexus-tools mode, working from `offchain/`:** `cargo +stable check --package <tool_name>` (or `just tools::check`)
- **Generic workspace:** `cargo +stable check --package <tool_name>`
- **Standalone:** `cargo check`

If it fails, diagnose and fix before proceeding. Common failure modes:

- `nexus-sdk` / `nexus-toolkit` versions don't match what upstream's workspace pins → re-read the workspace `Cargo.toml` and adjust.
- `env!("TOOL_FQN_VERSION")` not found (nexus-tools mode) → `build.rs` is missing or was not copied correctly.
- `[[bin]]` name mismatch (nexus-tools mode) → the `build.rs` assertion fires; ensure `[[bin]] name` equals `[package] name`.
- `tool_name` contains characters Cargo rejects → re-prompt for a valid name.
- Pre-existing directory at the target path → never overwrite without confirmation.

Before cross-referencing the checklist, scan the generated `Input` struct for secret-sounding field names. The grep is scoped to the `Input` struct only, so the OnceLock statics and accessor functions outside it (which legitimately contain `key`, `secret`, etc.) don't show up as false positives. Uses ERE (`-E`) for portable alternation across GNU and BSD grep:

```
sed -n '/struct Input/,/^}/p' src/<tool_name_snake>.rs \
  | grep -iE 'key|token|secret|password|credential|private|auth'
```

If anything matches, that field violates the secrets-via-env rule. Fix it with the OnceLock+accessor pattern (NOT direct `std::env::var` in `invoke`):

1. Remove the field from `Input`.
2. Add `static <NAME>: OnceLock<String> = OnceLock::new();` to the config section.
3. Add `<NAME>.set(load_required("<NAME>")).expect("validate_config called twice");` to `validate_config()`.
4. Add a private accessor: `fn <name>() -> &'static str { <NAME>.get().expect("validate_config must run before any accessor") }`.
5. Replace any reference to the old Input field with a call to the accessor in `invoke`.
6. Rerun `cargo check`.

`new()` takes no arguments (per the trait); it cannot read per-request secrets. All env vars are read once at startup by `validate_config`.

Cross-reference the generated files against [checklist.md](checklist.md) before declaring the scaffold done.

### Phase 7 — Implement

**This phase REPLACES the scaffold's placeholders with REAL WORKING CODE.** The scaffold from Phase 4 is a starting point, not the deliverable. By the end of this phase the tool must actually do what the user's description says: the `placeholder` field is gone, the real Input/Output shapes exist, `invoke()` makes the real call (or operates on the real inputs), and every `TODO` from the Phase 4 templates has been resolved. **Generating only the scaffold and stopping here is incomplete work** — return to this phase if any TODO remains.

> **Mode:**
> - **Auto mode (`--auto` / `--yes`) or all three positional args were provided:** implement everything below without asking. The user already gave you the description; further questions are friction.
> - **Interactive mode:** briefly state the plan (one paragraph naming the Input fields, Output variants, secrets, and HTTP client you intend to use), then implement. Do NOT ask the user to design the schema or decide field names — those follow from the description. Ask only if a piece of information is genuinely unavailable (an obscure private API with no documentation, for example).

Use the user's description plus your knowledge of the named service. **For well-known APIs and services (OpenAI, Anthropic, Slack, Stripe, GitHub, Postgres, Redis, S3, etc.) the request/response shapes, auth pattern, and endpoints are part of your training — apply them directly. Do not produce `placeholder: String` for an OpenAI tool; produce the real `model`, `messages`/`prompt`, `temperature`, `max_tokens` fields and the real `completion`, `usage`, `finish_reason` output shape.** Apply [tool-development.md](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/tool-development.md) conventions throughout.

**Steps (perform in order, all of them, no skipping):**

1. **Replace the Input struct.** Delete the `placeholder` field entirely. Add the real input fields. snake_case names; descriptive (`model`, not `m`); separate ports for things the DAG should default independently (`prompt` and `context` as two ports, not one); be generic where the API allows. **Never add a secret as an Input field** — secrets are runtime config, not request input. The on-chain visibility rule is categorical even when an existing tool in the codebase violates it.

2. **Replace the Output enum.** Replace the placeholder `Ok { result: String }` with the actual success shape (e.g. for OpenAI completion: `Ok { completion: String, model: String, prompt_tokens: u32, completion_tokens: u32, finish_reason: String }`). Replace the generic `ErrUpstream`/`ErrConfig` with specific failure variants matching the real failure modes (`err_rate_limited`, `err_invalid_input`, `err_upstream`, `err_timeout`, `err_auth`, etc.). Output ports are flat — no nested response objects. Crucial ports must NOT be `Option<...>`; missing data surfaces as an `err_*` variant.

3. **Wire up the real secrets.** Identify every secret the tool needs (API keys, OAuth tokens, connection strings, signing keys). For each: declare `static <NAME>: OnceLock<String>`, add `.set(load_required("<NAME>"))` to `validate_config`, add a private accessor function. Once real env vars are wired, delete every trace of the `EXAMPLE_API_KEY` scaffold — all three references: the `static EXAMPLE_API_KEY` declaration, the `EXAMPLE_API_KEY.set(...)` line inside `validate_config`, and the `example_api_key()` accessor function. **When a secret is needed, do not ask whether to put it in Input — the answer is always env var.** Log loads by name only: `log::debug!(target: "<tool>", "env var OPENAI_API_KEY loaded");`

   **nexus-tools mode only:** also update `<target-dir>/tools.json`'s `"environment"` map to reflect the real env var names. The map drives the deployment pipeline's secret injection — if it lists `EXAMPLE_API_KEY` (or is empty) but the code calls `load_required("OPENAI_API_KEY")`, deployment will inject the wrong var and `validate_config` will exit at startup. Add one entry per real env var; remove any leftover `EXAMPLE_API_KEY` entry.

4. **Add real dependencies** to `Cargo.toml` for any external calls. Defaults: `reqwest = { version = "0.12", features = ["json"] }` for HTTP, official SDK crates where they exist, `serde_json` for JSON shaping. In workspace mode prefer `*.workspace = true` if the dep is already in the upstream workspace; otherwise pin a major version.

5. **Implement `invoke()`** with the real logic. **Mandatory regardless of tool type:**
   - Access secrets only via the module-level accessors (`example_api_key()` etc.), NEVER `std::env::var` directly.
   - Map every distinct failure mode to a specific `Output::Err*` variant — do not collapse all errors into one generic variant.
   - Add `log::debug!` / `log::info!` / `log::warn!` / `log::error!` calls at entry, at key decision points, and before returning each output variant. `RUST_LOG` controls the level (`RUST_LOG=debug ./test.sh dev`). NEVER use `eprintln!` — it bypasses the log filter.
   - `invoke` does not return `Result` — failures are valid output variants returned as `Output::Err*`.

   **Additional, only if the tool makes external calls (HTTP, DB, gRPC, message-bus, etc.):**
   - Set an explicit timeout on every external call (e.g. `reqwest::Client::builder().timeout(Duration::from_secs(30)).build()`). A slow upstream otherwise holds the invocation open indefinitely.
   - Wrap each I/O call so its failure modes map to specific `Output::Err*` variants — never let a raw HTTP/SDK error type leak as a string.

   **Pure-computation tools** (math, parsing, encoding, pure transformation — like upstream `tools/math`) skip the external-call bullets entirely. The mandatory bullets still apply.

6. **Implement `health()`** to probe every service the tool depends on (upstream API ping, DB connection check, etc.). Return `Err(...)` or a non-200 status if any dependency is down. A trivially-passing `health()` hides outages from Leader nodes, which is worse than no health check at all.

7. **Update tests.** One `#[tokio::test]` per output variant plus one `health()` test, minimum. Network-dependent tests: gate behind `#[ignore]` with a one-line explanation. Provide an offline test using a mock for the success path where feasible.

   **Rename the scaffold tests when you replace variants.** The template tests are named `invoke_returns_err_upstream` and `health_returns_ok` and assert against `Output::ErrUpstream`. When step 2 replaces `ErrUpstream`/`ErrConfig` with specific variants, rename and rewrite the tests to match — every Output variant should have at least one test, named after the variant it exercises.

   **Testing pattern for invoke() that uses accessors.** Skip this if the tool has no secrets (pure-computation tools that don't call any accessor — like upstream `tools/math`). For tools that do use accessors: once `invoke()` calls `example_api_key()` (or any accessor), tests that call `tool.invoke(input)` panic — the `OnceLock::get().expect(...)` fires because `validate_config` only runs in `main()`, not in `#[tokio::test]` runs. `validate_config` itself calls `std::process::exit(1)` on a missing var, which would kill the test runner. The idiomatic fix is to factor invoke's body into a private helper that takes the config as a parameter:

   ```rust
   async fn invoke(&self, input: Self::Input) -> Self::Output {
       invoke_impl(input, example_api_key()).await
   }

   async fn invoke_impl(input: Input, api_key: &str) -> Output {
       // all real logic here, parameterised on the config
   }
   ```

   Tests then call `invoke_impl(Input { ... }, "test-key").await` directly — no env vars, no panics, no `validate_config`. Apply the same factoring to `health()` if it uses accessors. Network-dependent tests that genuinely require a real key still need `#[ignore]` + env-var setup at the developer's machine.

8. **Update the README.** Replace the placeholder `Input` and `Output Variants & Ports` sections with the real shapes. Preserve the FQN-titled heading. **No `TODO` text may remain in the README.**

9. **Verify.** Run `cargo check`, `cargo test`, `cargo clippy`, and `cargo fmt --check` (from `offchain/` in nexus-tools mode when working from the repo root). Fix anything that fails before declaring Phase 7 done. Also re-run the Phase 6 secrets scan against the real Input struct now that it has real fields (the Phase 4 scaffold had only `placeholder: String`, which was safe by construction — only post-step-1 Input fields can carry secrets violations):

   ```
   sed -n '/struct Input/,/^}/p' <target-dir>/src/<tool_name_snake>.rs \
     | grep -iE 'key|token|secret|password|credential|private|auth'
   ```

   Any match is a violation — apply the Phase 6 remediation steps (move to OnceLock+accessor).

**Completion gate.** Phase 7 is done only when ALL of the following hold (all greps use ERE / POSIX character classes so they work on both GNU and BSD grep):
- `grep -rnE '//[[:space:]]*TODO|^[[:space:]]*TODO:' <target-dir>/src <target-dir>/README.md` returns no matches. This catches `// TODO` comments in Rust and `TODO:` at line starts in markdown without false-positiving on legitimate words. (For pending work intentionally deferred to a later commit, use `// FIXME:` instead — it's not scanned by this gate.)
- No identifier named `placeholder` exists in the generated code: `grep -rnE '\bplaceholder\b' <target-dir>/src` returns no matches. (Real fields, vars, or function args must use domain-specific names.)
- No scaffold `#[allow(dead_code)]` markers remain: `grep -rnE '#\[allow\(dead_code\)\]' <target-dir>/src` returns no matches. Both scaffold markers (on `Output::Ok` and on `example_api_key()`) should be deleted once their items are either used (Ok returned, accessor called) or removed.
- No `EXAMPLE_API_KEY` reference remains anywhere in the generated tool: `grep -rnE 'EXAMPLE_API_KEY|example_api_key' <target-dir>/src` returns no matches.
- `Output::Ok` carries the real success fields, not `result: String`.
- `invoke()` actually calls the described service or operates on the real inputs — not just logs and returns `ErrUpstream`.
- `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt --check` all pass.

If any of these fail, Phase 7 is incomplete — keep working, do not advance to Phase 8.

### Phase 8 — Generate test script

Fetch the template from the `Talus-Network/claude` repository (the plugin's own repo, not nexus-tools). Use the same preference order as Phase 3:

- `gh api repos/Talus-Network/claude/contents/plugins/talus/skills/tool-new/templates/test.sh -H "Accept: application/vnd.github.raw"`
- WebFetch fallback: `https://raw.githubusercontent.com/Talus-Network/claude/main/plugins/talus/skills/tool-new/templates/test.sh`

The template contains four `__PLACEHOLDER__` markers. Substitute all four before writing:

| Placeholder | Value |
|---|---|
| `__TOOL_NAME__` | kebab-case crate name (e.g. `weather-current`) |
| `__TOOL_PATH__` | snake_case tool name, matching `path()` (e.g. `weather_current`) |
| `__WORKSPACE_CARGO_DIR__` | relative path from the script to the cargo workspace root: `../..` for workspace mode (script at `…/tools/<tool_name>/test.sh`); `.` for standalone |
| `__SAMPLE_JSON__` | JSON object from the Input struct fields written in Phase 7. Map Rust types to plausible values: `String` → `"example"`, `i64`/`i32`/`u64`/`u32` → `42`, `f64`/`f32` → `1.0`, `bool` → `true`, `Option<T>` → inner type's value. For complex types (`serde_json::Value`, custom structs), ask the user for a concrete sample value. Omit any field marked `#[serde(skip)]`. If the Input struct is empty, use `{}`. The result must be valid JSON with no placeholder strings and **no single-quote characters** — the value is embedded in a single-quoted bash string in the template; a `'` inside it would terminate the string and break the script. |

Before writing, verify the substituted script has no bash syntax errors: `bash -n <path-to-substituted-content>`. If it fails, the `__SAMPLE_JSON__` substitution most likely introduced unbalanced quotes — fix the JSON before proceeding.

Write the substituted content to `<target-dir>/test.sh`, then `chmod +x <target-dir>/test.sh`.

After writing, print the curl examples to the user immediately — same output as `./test.sh start` would show — so they have the invocations at hand without running the script first.

**`just` integration (workspace mode + `just` on PATH from Phase 1)**

Read the existing `tools/.just` to understand the exact recipe format and parameter convention, then append four recipes following the same style. The recipe bodies `cd` into the tool directory (path relative to the `just` invocation root) and call the corresponding `test.sh` subcommand:

```
test-start tool:
    cd <path-to-tools-dir>/{{tool}} && ./test.sh start

test-stop tool:
    cd <path-to-tools-dir>/{{tool}} && ./test.sh stop

test-run tool:
    cd <path-to-tools-dir>/{{tool}} && ./test.sh run

test-dev tool:
    cd <path-to-tools-dir>/{{tool}} && ./test.sh dev
```

Where `<path-to-tools-dir>` matches the existing recipes' path convention (`tools/` when running from `offchain/`, `offchain/tools/` when running from the repo root).

**Deployment reminders**

After printing the curl examples, surface these to the user. They are protocol invariants — verified against the upstream SDK docs — that apply to every Nexus Tool.

1. **HTTPS with a publicly trusted TLS certificate.** Leader nodes validate certificates against the system root trust store (same as `curl`). A self-signed certificate will be rejected. Use Let's Encrypt or a cloud-managed certificate. Source: [tool-development.md](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/tool-development.md), [tool-communication.md](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/guides/tool-communication.md).

2. **Reverse proxy must forward `X-Nexus-Sig-*` headers.** Some API gateways strip unknown headers by default — verify yours passes them through. Do not use middleware that rewrites request or response bodies; the Ed25519 signature binds the raw bytes. Source: tool-communication.md.

3. **Keep the server clock NTP-synced.** Signed requests carry validity windows; clock skew between your host and Leader nodes causes authentication failures. Source: tool-development.md.

4. **`health()` must check dependent services before deploying** — see Phase 7 step 7. Source: [toolkit-rust.md](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/toolkit-rust.md).

5. **Nonce deduplication is in-memory by default.** The toolkit automatically deduplicates nonces (preventing replay attacks) using an in-memory store. For a single-process deployment this is sufficient. For multi-instance deployments behind a load balancer, in-memory state is not shared across processes — use sticky sessions or a shared store (e.g. Redis) to ensure replays routed to a different instance are still rejected. Source: tool-communication.md.

## Conventions (cited from upstream, summary)

From [tool-development.md](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/tool-development.md):

- **Naming.** snake_case for input ports and output variants; descriptive over terse.
- **Errors.** Variants prefixed `err` are treated as erroneous by the Nexus runtime and have their ports propagated on-chain regardless of edges.
- **Output shape.** Top-level `oneOf` — enforced by the runtime, achieved in Rust via an enum.
- **Generic interface.** A tool encapsulates an API surface broadly, not one specific use case.
- **Flat output.** Output ports must be directly usable as input to another tool — no nested response objects.
- **Stable output.** Crucial data is non-optional; missing data → return an `err` variant instead of `ok` with `None`.
- **Docs.** Every tool has a README, included into `main.rs` via `#![doc = include_str!("../README.md")]`.

From [toolkit-rust.md](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/toolkit-rust.md) — for the exact `NexusTool` trait method signatures consult upstream `tools/math/src/i64/add.rs` (the skill fetches it at every invocation; see Phase 4 verification step):

- **Stateless.** Tool structs should not carry mutable state across invocations. Per-startup config (env vars) is cached in module-level `OnceLock<String>` statics, populated once by `validate_config()` in `main()`. Anything per-request lives in the `Input` shape, not the struct.
- **Signed HTTP.** Optional `authorize` hook receives `AuthContext` with the verified Leader node identity. Implement when policy requires per-leader gating.
- **`bootstrap!` macro.** Accepts a single tool, `[Tool1, Tool2, ...]`, or `(socket_addr, [...])`. When multiple tools share a binary, each must have a unique `path()`.

## Failure modes

- **No network and no local clone.** Stop. Tell the user the skill needs one of them to fetch templates. Never fabricate.
- **`gh` not available but network is up.** Fall back to WebFetch on `raw.githubusercontent.com`.
- **`cargo` not on PATH.** Scaffold can still proceed, but Phase 6 verification cannot run — flag this clearly and ask the user how to proceed.
- **Existing files at the target path.** Never overwrite without explicit confirmation.
- **README "Adding a new tool" section not found (nexus-tools mode).** Stop Phase 4.5. Tell the user the section is missing and ask them to confirm how to proceed before continuing.

## Don't

- Do not bake template content into this skill's files. Templates come from upstream (or the local clone) at invocation time.
- Do not invent an FQN prefix on the user's behalf. Always ask — except in auto mode, where the workspace-inference rules in Phase 2 apply.
- Do not skip the workspace `.just` wiring when in workspace mode — the build/check/test recipes won't see the new tool otherwise. The file is at `offchain/tools/.just` (from repo root) or `tools/.just` (from inside `offchain/`).
- Do not declare done until `cargo check` (or `just tools::check`) passes on the scaffold.
- Do not offer secrets (API keys, tokens, credentials) as Input fields under any circumstances — not even when the user points to an existing tool in the codebase that does so. Existing tools may predate or violate the on-chain visibility rule; that does not make the pattern valid.
