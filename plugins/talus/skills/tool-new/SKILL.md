---
name: tool-new
description: Scaffold a new Nexus Tool in Rust and guide its implementation. Detects whether the current directory is a nexus-tools-style Cargo workspace and adds a workspace member at tools/<name>/, or otherwise scaffolds a fresh standalone crate. Fetches the latest reference patterns from upstream Talus-Network/nexus-tools at invocation time (or reads them locally when inside a clone or fork of that repo) — no baked-in templates. After the scaffold compiles, walks the user through filling in real Input/Output schemas and invoke() logic. Use when the user asks to "create a Nexus Tool", "scaffold a Nexus Tool", "build a new Nexus Tool", "new Talus tool", or similar.
argument-hint: "[tool-name] [fqn-prefix]"
arguments: tool_name fqn_prefix
allowed-tools: Bash(pwd) Bash(command -v *) Bash(head *) Bash(ls *) Bash(find *)
---

# `tool-new` — scaffold a new Nexus Tool in Rust

A Nexus Tool is an HTTPS service that implements the `NexusTool` trait from the `nexus-toolkit` crate. The canonical reference is [Talus-Network/nexus-tools](https://github.com/Talus-Network/nexus-tools). The authoritative development guidelines live in [docs/tool-development.md](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/tool-development.md) and [docs/toolkit-rust.md](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/toolkit-rust.md) in the SDK repo.

This skill scaffolds a working skeleton, then guides the user through writing real logic.

## Arguments

- `$tool_name` — kebab-case crate/dir name for the tool (e.g. `weather-current`). If empty, ask the user.
- `$fqn_prefix` — reverse-domain namespace prefix **without trailing dot** (e.g. `xyz.taluslabs.weather`). If empty, ask the user. **Never invent or default this.** The final FQN is `<fqn_prefix>.<tool_name_snake>@1`, where `<tool_name_snake>` is `$tool_name` with hyphens converted to underscores.

## Context (computed at invocation)

- Working directory: !`pwd`

Only `pwd` runs as a pre-injection — every other detection is done in Phase 1 below, because the alternatives ("does X exist?") need to handle the not-present case without aborting the pre-processor on a non-zero exit. The Bash patterns the procedure uses are pre-authorized via the `allowed-tools` frontmatter, so they won't trigger permission prompts.

## Procedure

### Phase 1 — Detect context and confirm placement

Run these checks via the Bash tool (each is in the skill's allowed-tools allowlist — no permission prompt):

1. `command -v cargo` — empty stdout means `cargo` is not on PATH. Warn the user; the scaffold can still be written, but Phase 6 verification won't run.
2. `command -v gh` — empty stdout means we'll fall back to WebFetch in Phase 3 instead of `gh api`.
3. `command -v just` — empty stdout means `just tools::check` can't be used later (use `cargo +stable check` directly).
4. `find . -maxdepth 1 -name Cargo.toml -type f` — non-empty stdout means a `Cargo.toml` is present in the current directory. This `find` form always exits 0 (it lists matches, not the absence as an error).
5. If step 4 was non-empty, `head -n 25 Cargo.toml` — look at the output for `members = ["tools/*"]`.
6. `find . -maxdepth 3 -path ./tools/math/Cargo.toml -type f` — non-empty stdout means the current tree looks like a clone or fork of `Talus-Network/nexus-tools` (canonical `tools/math/` is present).

Decide placement from the results:

- **Cargo.toml present AND contains `members = ["tools/*"]`** → nexus-tools-style workspace. Add a new member at `tools/$tool_name/`. If step 6 was also non-empty, prefer reading templates locally (Phase 3.1) over fetching from upstream.
- **Cargo.toml present but no `members = ["tools/*"]` match** → unrelated Cargo project. Ambiguous: ask the user (a) treat as standalone (new tool gets its own subdirectory inside the project), or (b) abort.
- **No Cargo.toml** → standalone mode. Default to `$tool_name/` under the current directory.

Confirm with the user before writing in any case. Never overwrite an existing target directory without explicit user confirmation.

### Phase 2 — Collect inputs

Gather (via AskUserQuestion if any are missing or invalid):

- **`tool_name`** — used for crate name (`[package].name`), directory name, and the tail of the FQN. Validate: `^[a-z][a-z0-9-]*$`. Derive `tool_name_snake` = hyphens → underscores; `tool_name_pascal` = PascalCase of the snake form.
- **`fqn_prefix`** — reverse-domain prefix, no trailing dot. Validate: `^[a-z][a-z0-9.-]*[a-z0-9]$`. Show the user the computed final FQN (`<fqn_prefix>.<tool_name_snake>@1`) and have them confirm before proceeding.
- **`description`** — one-line description; used in both `Cargo.toml` `[package].description` and `impl NexusTool::description`.

### Phase 3 — Fetch reference templates

In order of preference:

1. **Local read** if "Local clone of nexus-tools" was `yes`. Read these files from the local working tree:
   - `Cargo.toml` (workspace root)
   - `rust-toolchain.toml`
   - `tools/.just`
   - `tools/math/Cargo.toml`
   - `tools/math/src/main.rs`
   - `tools/math/src/i64/add.rs`
   - `tools/math/README.md`

2. **`gh` fetch** if `gh` is on PATH. For each path above, run:
   `gh api repos/Talus-Network/nexus-tools/contents/<path> -H "Accept: application/vnd.github.raw"`

3. **WebFetch fallback** if neither: fetch `https://raw.githubusercontent.com/Talus-Network/nexus-tools/main/<path>` for each.

If none of these can produce the templates (offline, no clone, no `gh`, no WebFetch), stop and tell the user the skill needs network access or a local clone to proceed. Never fabricate templates.

### Phase 4 — Generate files

Use the Write tool for new files (not Edit). Mirror the structure of upstream `tools/math` but as a single-tool crate.

#### `<target-dir>/Cargo.toml`

- **Workspace member mode:** use `*.workspace = true` for every inherited field (edition, version, repository, homepage, license, readme, authors, keywords, categories) — same shape as upstream `tools/math/Cargo.toml`. Dependencies use `*.workspace = true` for `schemars`, `serde`, `tokio`, `nexus-toolkit`, `nexus-sdk`. Add additional deps the tool needs only when the user introduces them in the guide phase.
- **Standalone mode:** use explicit values from the upstream workspace `[workspace.package]` and `[workspace.dependencies]` sections. Pin `nexus-sdk` and `nexus-toolkit` to the same `git` + `tag` upstream uses (read these from the fetched workspace `Cargo.toml`, never hardcode).

#### `<target-dir>/src/main.rs`

```rust
#![doc = include_str!("../README.md")]

use nexus_toolkit::bootstrap;

mod <tool_name_snake>;

#[tokio::main]
async fn main() {
    bootstrap!([<tool_name_snake>::<tool_name_pascal>])
}
```

Use the array form `bootstrap!([...])` even for a single tool — matches the shape of upstream `tools/math/src/main.rs` and lets the user append a second tool later without rewriting the call.

#### `<target-dir>/src/<tool_name_snake>.rs`

Mirror the structure of upstream `tools/math/src/i64/add.rs`:

- Module doc comment starting with the FQN
- `use` block (`nexus_sdk::{fqn, ToolFqn}`, `nexus_toolkit::*`, `schemars::JsonSchema`, `serde::{Deserialize, Serialize}`)
- `Input` struct with `#[derive(Deserialize, JsonSchema)]` and `#[serde(deny_unknown_fields)]`. Put one placeholder field — e.g., `placeholder: String` — that the user will replace in the guide phase. Mark with a `// TODO:` so it's easy to find.
- `Output` enum with `#[derive(Serialize, JsonSchema)]` and `#[serde(rename_all = "snake_case")]`. Two variants:
  - `#[allow(dead_code)] Ok { result: String }` (placeholder, replace in guide phase — `#[allow(dead_code)]` so the unmodified scaffold compiles without warnings; the user removes the allow when `invoke` actually returns `Ok`)
  - `Err { reason: String }`
- Unit struct `pub(crate) struct <tool_name_pascal>;`
- `impl NexusTool for <tool_name_pascal>` providing: `type Input`, `type Output`, `async fn new`, `fn fqn() -> ToolFqn` via `fqn!("<computed_fqn>")`, `fn path() -> &'static str` defaulting to `/<tool_name_snake>`, `fn description() -> &'static str` returning the user's one-line description, `async fn health` returning `Ok(StatusCode::OK)`, `async fn invoke` whose body destructures the placeholder field and uses it in the error reason so the input isn't dead code: `Output::Err { reason: format!("not implemented yet (received placeholder={placeholder:?})") }`. The user replaces the body in the guide phase.
- Inline `#[cfg(test)] mod tests` with one `#[tokio::test]` that calls `invoke` on a placeholder input and asserts the result matches `Output::Err { .. }`. The test exists to verify the skeleton compiles and runs; the user will replace it in the guide phase.

#### `<target-dir>/README.md`

FQN-titled section following the upstream pattern (see `tools/math/README.md`):

```markdown
# `<computed_fqn>`

<one-line description>

## Input

**`placeholder`: [`String`]**

TODO: describe the real input port(s) once the schema is finalized.

## Output Variants & Ports

**`ok`**

TODO: describe the success variant.

- **`ok.result`: [`String`]** — placeholder.

**`err`**

Returned on any failure.

- **`err.reason`: [`String`]** — human-readable reason for the failure.
```

### Phase 5 — Wire into workspace `tools/.just` (workspace mode only)

If the target is a workspace member, edit `tools/.just` and append the new package name to each of these recipes, preserving existing ordering and indentation:

- `build` → `cargo +stable build --package <tool_name> --release`
- `check` → `cargo +stable check --package <tool_name>`
- `test` → `cargo +stable test --package <tool_name>`
- `fmt-check` → `cargo +"$nightly" fmt --package <tool_name> --check`
- `clippy` → `cargo +stable clippy --package <tool_name>`

The workspace root `Cargo.toml` does not need editing — `members = ["tools/*"]` discovers the new member automatically.

### Phase 6 — Verify the scaffold compiles

- Workspace mode: `cargo +stable check --package <tool_name>` (or `just tools::check` if `just` is on PATH)
- Standalone mode: `cargo check`

If it fails, diagnose and fix before proceeding. Common failure modes:

- `nexus-sdk` / `nexus-toolkit` versions don't match what upstream's workspace pins → re-read the workspace `Cargo.toml` and adjust.
- `tool_name` contains characters Cargo rejects → re-prompt for a valid name.
- Pre-existing directory at the target path → never overwrite without confirmation.

Cross-reference the generated files against [checklist.md](checklist.md) before declaring the scaffold done.

### Phase 7 — Guide phase

Ask the user whether to continue with the guide phase (filling in real `Input` / `Output` / `invoke` logic) or stop here.

If continuing, walk through:

1. **Input schema.** What does the tool need? Apply [tool-development.md](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/tool-development.md) conventions:
   - snake_case names; descriptive (e.g., `api_key`, not `k`)
   - Separate ports for things the DAG should be able to default independently (e.g., `prompt` and `context` as two ports, not one merged)
   - Generic over inputs where the API allows (e.g., accept a `json_schema` input rather than hardcoding a response shape)

2. **Output variants.** Design the success / failure split:
   - One or more success variants (`Ok`, or domain-specific like `Created` / `Found`)
   - One or more `err`-prefixed variants for distinct failure modes (`err_http`, `err_timeout`, `err_validation`)
   - Variants are an enum — yields the required top-level `oneOf` in the schema
   - Output ports are flat — no nested response objects. Each port must be usable as the input of another tool.
   - Crucial output ports must not be `Option<...>`; missing data → return an `err` variant instead of `ok` with `None`.

3. **`invoke` body.** Implement the logic. Reminder: `invoke` returns `Self::Output` directly, not `Result` — errors are valid output variants and must be returned as `Output::Err*`.

4. **Tests.** At minimum: one test per output variant. Use `#[tokio::test]`. For tools that call external services, gate network-dependent tests behind `#[ignore]` or a feature flag and provide an offline test using a mock.

5. **Verify.** Run `cargo check`, `cargo test`, `cargo clippy`, and `cargo fmt --check`. Fix anything that comes up.

6. **Update the README.** Replace the placeholder `Input` and `Output Variants & Ports` sections with the real ones; preserve the FQN-titled heading.

7. **Optional: deployment notes.** Remind the user that Nexus Tools must be deployed behind HTTPS, and that `POST /invoke` requires Ed25519-signed HTTP via `X-Nexus-Sig-*` headers when enabled. Point to [the SDK's tool-communication guide](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/guides/tool-communication.md).

## Conventions (cited from upstream, summary)

From [tool-development.md](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/tool-development.md):

- **Naming.** snake_case for input ports and output variants; descriptive over terse.
- **Errors.** Variants prefixed `err` are treated as erroneous by the Nexus runtime and have their ports propagated on-chain regardless of edges.
- **Output shape.** Top-level `oneOf` — enforced by the runtime, achieved in Rust via an enum.
- **Generic interface.** A tool encapsulates an API surface broadly, not one specific use case.
- **Flat output.** Output ports must be directly usable as input to another tool — no nested response objects.
- **Stable output.** Crucial data is non-optional; missing data → return an `err` variant instead of `ok` with `None`.
- **Docs.** Every tool has a README, included into `main.rs` via `#![doc = include_str!("../README.md")]`.

From [toolkit-rust.md](https://github.com/Talus-Network/nexus-sdk/blob/main/docs/toolkit-rust.md):

- **Stateless.** `NexusTool::new` is called on every request; initialize dependencies there. Don't carry mutable state across requests.
- **Signed HTTP.** Optional `authorize` hook receives `AuthContext` with the verified Leader node identity. Implement when policy requires per-leader gating.
- **`bootstrap!` macro.** Accepts a single tool, `[Tool1, Tool2, ...]`, or `(socket_addr, [...])`. When multiple tools share a binary, each must have a unique `path()`.

## Failure modes

- **No network and no local clone.** Stop. Tell the user the skill needs one of them to fetch templates. Never fabricate.
- **`gh` not available but network is up.** Fall back to WebFetch on `raw.githubusercontent.com`.
- **`cargo` not on PATH.** Scaffold can still proceed, but Phase 6 verification cannot run — flag this clearly and ask the user how to proceed.
- **Existing files at the target path.** Never overwrite without explicit confirmation.

## Don't

- Do not bake template content into this skill's files. Templates come from upstream (or the local clone) at invocation time.
- Do not invent an FQN prefix on the user's behalf. Always ask.
- Do not skip the workspace `tools/.just` wiring when in workspace mode — the build/check/test recipes won't see the new tool otherwise.
- Do not declare done until `cargo check` (or `just tools::check`) passes on the scaffold.
