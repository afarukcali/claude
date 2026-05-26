# Post-scaffold checklist

Walk through these after Phase 6 (cargo check passed). Each item is a property the scaffold must satisfy; if any fails, fix before declaring scaffold done.

## `<target-dir>/Cargo.toml`

- [ ] `[package].name` matches the kebab-case `tool_name`
- [ ] `[package].description` matches the user-provided description
- [ ] **If workspace member:** every inherited field uses `*.workspace = true` (edition, version, repository, homepage, license, readme, authors, keywords, categories) — matches upstream `tools/math/Cargo.toml`
- [ ] **If standalone:** explicit values for edition (matches upstream workspace's `edition`), version, license; explicit `git` + `tag` for `nexus-sdk` and `nexus-toolkit` matching the upstream workspace
- [ ] `nexus-sdk` and `nexus-toolkit` versions match upstream's `[workspace.dependencies]` (re-read at invocation time, never hardcoded)

## `<target-dir>/src/main.rs`

- [ ] First line is `#![doc = include_str!("../README.md")]`
- [ ] `use nexus_toolkit::bootstrap;` import
- [ ] `mod <tool_name_snake>;` declaration
- [ ] `#[tokio::main] async fn main()` with `bootstrap!([<tool_name_snake>::<tool_name_pascal>])` — array form even for a single tool, matching upstream `tools/math/src/main.rs`

## `<target-dir>/src/<tool_name_snake>.rs`

- [ ] Module doc comment starts with the FQN in backticks
- [ ] `use` block: `nexus_sdk::{fqn, ToolFqn}`, `nexus_toolkit::*`, `schemars::JsonSchema`, `serde::{Deserialize, Serialize}`
- [ ] `Input` struct has `#[derive(Deserialize, JsonSchema)]` and `#[serde(deny_unknown_fields)]`
- [ ] `Output` enum has `#[derive(Serialize, JsonSchema)]` and `#[serde(rename_all = "snake_case")]`
- [ ] `Output` has at least one success variant and at least one `err`-prefixed variant
- [ ] `impl NexusTool` provides: `type Input`, `type Output`, `async fn new`, `fn fqn`, `fn path`, `fn description`, `async fn health`, `async fn invoke`
- [ ] `fqn()` form matches the mode: generic/standalone → `fqn!("<fqn_prefix>.<tool_name_snake>@1")`; nexus-tools → `fqn!(concat!("<fqn_prefix>.<tool_name_snake>@", env!("TOOL_FQN_VERSION")))`
- [ ] `invoke()` does **not** return `Result` — failures are returned as `Output::Err*` variants
- [ ] Inline `#[cfg(test)] mod tests` with one `#[tokio::test]` per output variant (at minimum `Ok` and `Err`) plus one health check

## `<target-dir>/README.md`

- [ ] Top-level heading is the FQN in backticks (e.g. `` # `xyz.taluslabs.weather.current@1` ``)
- [ ] `## Input` section lists every input port with type and description — no TODO or placeholder text
- [ ] `## Output Variants & Ports` section lists every variant; for each variant, lists every output port with type and description — no TODO or placeholder text

## Workspace integration (workspace mode only)

- [ ] The workspace `.just` file (`offchain/tools/.just` from repo root, or `tools/.just` from inside `offchain/`) has the new package added to `build`, `check`, `test`, `fmt-check`, `clippy` recipes
- [ ] Existing recipe ordering and indentation preserved
- [ ] Workspace root `Cargo.toml` is **not** edited (the `members = ["tools/*"]` glob discovers the new member automatically)

## nexus-tools CI requirements (nexus-tools mode only)

- [ ] `tools.json` present at `<target-dir>/tools.json`; copied from a reference tool (not fabricated); contains at minimum `"tool_name"`, `"command"` (must equal the binary/crate name), and `"environment"`
- [ ] `build.rs` present; copied from an existing tool (e.g. `offchain/tools/math/build.rs`); not fabricated
- [ ] `[build-dependencies]` in `Cargo.toml` includes `serde_json.workspace = true` and `toml = "0.8"` (required by `build.rs`)
- [ ] `[[bin]]` section in `Cargo.toml` with `name = "<tool_name>"` (must equal `[package].name` and `tools.json["command"]`)
- [ ] `fqn!()` uses `concat!("<prefix>.<name>@", env!("TOOL_FQN_VERSION"))`, not a literal `@1`
- [ ] When working from the repo root: `cargo check` (and all cargo commands) run from `offchain/`, not the repo root

## Test script

- [ ] `<target-dir>/test.sh` exists, is executable (`chmod +x`), and passes `bash -n test.sh`
- [ ] All four `__PLACEHOLDER__` markers are substituted — no `__` strings remain in the file
- [ ] `SAMPLE_JSON` is valid JSON, reflects the actual Input fields, and contains no placeholder strings
- [ ] `./test.sh run` starts the server, gets a response, and stops cleanly
- [ ] **Workspace mode with `just`:** `test-start`, `test-stop`, `test-run` recipes present in `tools/.just` and `just --list` produces no errors

## Verification

- [ ] `cargo +stable check --package <tool_name>` (workspace) or `cargo check` (standalone) passes with no errors

## Convention sanity (from upstream tool-development.md)

- [ ] All input port names are snake_case
- [ ] All output variant names are snake_case; failure variants are prefixed `err`
- [ ] Output ports have no nested objects — flat structure
- [ ] Crucial output ports are not `Option<...>` — return an `err` variant instead
- [ ] Read every field name in the `Input` struct. Any name containing `key`, `token`, `secret`, `password`, `credential`, `private`, `auth`, or similar is a violation — remove the field and read the value via `std::env::var(...)` in `invoke` (or `new`) instead. Input ports go on-chain and are permanently visible.
