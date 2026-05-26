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
- [ ] `fqn()` returns the computed FQN via `fqn!("<fqn_prefix>.<tool_name_snake>@1")`
- [ ] `invoke()` does **not** return `Result` — failures are returned as `Output::Err*` variants
- [ ] Inline `#[cfg(test)] mod tests` with at least one `#[tokio::test]`

## `<target-dir>/README.md`

- [ ] Top-level heading is the FQN in backticks (e.g. `` # `xyz.taluslabs.weather.current@1` ``)
- [ ] `## Input` section lists every input port with type and description
- [ ] `## Output Variants & Ports` section lists every variant; for each variant, lists every output port with type and description

## Workspace integration (workspace mode only)

- [ ] `tools/.just` has the new package added to `build`, `check`, `test`, `fmt-check`, `clippy` recipes
- [ ] Existing recipe ordering and indentation preserved
- [ ] Workspace root `Cargo.toml` is **not** edited (the `members = ["tools/*"]` glob discovers the new member automatically)

## nexus-tools CI requirements (nexus-tools mode only)

- [ ] `tools.json` present at `<target-dir>/tools.json`; structure matches the reference from an existing tool
- [ ] `build.rs` present; copied from an existing tool (e.g. `offchain/tools/math/build.rs`); not fabricated
- [ ] `[[bin]]` section in `Cargo.toml` with `name = "<tool_name>"` (must equal `[package].name`)
- [ ] `fqn!()` uses `concat!("<prefix>.<name>@", env!("TOOL_FQN_VERSION"))`, not a literal `@1`
- [ ] `cargo check` runs from `offchain/` (not the repo root)

## Verification

- [ ] `cargo +stable check --package <tool_name>` (workspace) or `cargo check` (standalone) passes with no errors

## Convention sanity (from upstream tool-development.md)

- [ ] All input port names are snake_case
- [ ] All output variant names are snake_case; failure variants are prefixed `err`
- [ ] Output ports have no nested objects — flat structure
- [ ] Crucial output ports are not `Option<...>` — return an `err` variant instead
- [ ] Read every field name in the `Input` struct. Any name containing `key`, `token`, `secret`, `password`, `credential`, `private`, `auth`, or similar is a violation — remove the field and read the value via `std::env::var(...)` in `invoke` (or `new`) instead. Input ports go on-chain and are permanently visible.
