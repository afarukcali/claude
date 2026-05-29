# Post-scaffold checklist

Walk through these after Phase 6 (cargo check passed). Each item is a property the scaffold must satisfy; if any fails, fix before declaring scaffold done.

## `<target-dir>/Cargo.toml`

- [ ] `[package].name` matches the kebab-case `tool_name`
- [ ] `[package].description` matches the user-provided description
- [ ] **If workspace member:** every inherited field uses `*.workspace = true` (edition, version, repository, homepage, license, readme, authors, keywords, categories) — matches upstream `tools/math/Cargo.toml`
- [ ] **If standalone:** explicit values for edition (matches upstream workspace's `edition`), version, license; explicit `git` + `tag` for `nexus-sdk` and `nexus-toolkit` matching the upstream workspace
- [ ] `nexus-sdk` and `nexus-toolkit` versions match upstream's `[workspace.dependencies]` (re-read at invocation time, never hardcoded)
- [ ] `log` is a direct dep (`log.workspace = true` or `log = "0.4"`) — the template uses `log::*` macros which are not re-exported by `nexus_toolkit::*`

## `<target-dir>/src/main.rs`

- [ ] First line is `#![doc = include_str!("../README.md")]`
- [ ] `use nexus_toolkit::bootstrap;` import
- [ ] `mod <tool_name_snake>;` declaration
- [ ] `let _ = nexus_toolkit::env_logger::try_init();` called before `validate_config` so `log::*` calls work during startup
- [ ] `<tool_name_snake>::validate_config();` called before `bootstrap!`
- [ ] `#[tokio::main] async fn main()` with `bootstrap!([<tool_name_snake>::<tool_name_pascal>])` — array form even for a single tool, matching upstream `tools/math/src/main.rs`

## `<target-dir>/src/<tool_name_snake>.rs`

- [ ] Module doc comment starts with the FQN in backticks
- [ ] `use` block: `nexus_sdk::{fqn, ToolFqn}`, `nexus_toolkit::*`, `schemars::JsonSchema`, `serde::{Deserialize, Serialize}`, `std::sync::OnceLock`
- [ ] `pub(crate) fn validate_config()` is present; reads every required env var at startup via `load_required` and stores each in its `OnceLock` static. Aborts via `log::error!` + `std::process::exit(1)` — no `eprintln!`
- [ ] One `static <NAME>: OnceLock<String>` per secret env var, with a matching `<NAME>.set(load_required("<NAME>")).expect(...)` line inside `validate_config`
- [ ] One private accessor function per secret env var; returns the cached value via `.get().expect(...)` — never calls `std::env::var`
- [ ] `Input` struct has `#[derive(Deserialize, JsonSchema)]` and `#[serde(deny_unknown_fields)]`
- [ ] `Output` enum has `#[derive(Serialize, JsonSchema)]` and `#[serde(rename_all = "snake_case")]`
- [ ] `Output` has at least one success variant, `ErrUpstream`, and `ErrConfig`
- [ ] `impl NexusTool` provides — with exactly these signatures:
  - [ ] `type Input`, `type Output`
  - [ ] `async fn new() -> Self` (no args, no Result)
  - [ ] `fn fqn() -> ToolFqn`
  - [ ] `fn path() -> &'static str`
  - [ ] `fn description() -> &'static str`
  - [ ] `async fn health(&self) -> AnyResult<StatusCode>` — note `&self`, not `&()`
  - [ ] `async fn invoke(&self, input: Self::Input) -> Self::Output` — note `&self`, not `self`
- [ ] `fqn()` form matches the mode: generic/standalone → `fqn!("<fqn_prefix>.<tool_name_fqn_tail>@1")`; nexus-tools → `fqn!(concat!("<fqn_prefix>.<tool_name_fqn_tail>@", env!("TOOL_FQN_VERSION")))`. The `tool_name_fqn_tail` segment uses the case convention inferred from existing workspace tools (kebab in nexus-tools mode and any workspace with hyphenated FQN tails; snake elsewhere) and **matches** the value `path()` returns.
- [ ] `invoke()` does **not** return `Result` — failures are returned as `Output::Err*` variants
- [ ] `invoke()` accesses secrets via module-level accessors, not `std::env::var` directly
- [ ] Inline `#[cfg(test)] mod tests` with one `#[tokio::test]` per output variant (at minimum `Ok`, `ErrUpstream`, and `ErrConfig`) plus one health check. Each test constructs the tool via `__TOOL_NAME_PASCAL__::new().await` (not by bare struct literal) so the construction path is exercised.

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
- [ ] `tools.json`'s `"environment"` map lists every env var name the code passes to `load_required(...)` and contains no leftover scaffold names (`EXAMPLE_API_KEY` must be gone after Phase 7 step 3)
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
- [ ] **Workspace mode with `just`:** `test-start`, `test-stop`, `test-run`, `test-dev` recipes present in `tools/.just` and `just --list` produces no errors

## Verification

- [ ] `cargo +stable check --package <tool_name>` (workspace) or `cargo check` (standalone) passes with no errors

## Convention sanity (from upstream tool-development.md)

- [ ] All input port names are snake_case
- [ ] All output variant names are snake_case; failure variants are prefixed `err`
- [ ] Output ports have no nested objects — flat structure
- [ ] Crucial output ports are not `Option<...>` — return an `err` variant instead
- [ ] Read every field name in the `Input` struct (use `sed -n '/struct Input/,/^}/p' src/<tool_name_snake>.rs | grep -iE 'key|token|secret|password|credential|private|auth'` to scope the search; ERE form is portable across GNU and BSD grep). Any match is a violation — remove the field, add a `static <NAME>: OnceLock<String>` declaration, a `.set(load_required("<NAME>"))` line in `validate_config`, and an accessor function. Input ports go on-chain and are permanently visible. `new()` takes no args and cannot read per-request secrets.
- [ ] No direct `std::env::var` calls inside `invoke` or `new` — all env var access goes through the module-level accessor functions generated in the config section.

## Naming conformance (applies in every mode, including standalone / new repos)

The skill's auto-mode composition is description-driven and works without existing tools in the workspace. These items verify the output reflects the description's richness — not a fallback to a flat `<root>.<action>` shape.

- [ ] When the user's description names a domain noun ("weather", "social", "storage"), `fqn_prefix` includes that as a middle category segment.
- [ ] When the user's description names a well-known service (OpenAI, Open-Meteo, Twitter, etc.), `fqn_prefix` includes that service kebab-cased as a middle source segment (`openai`, `open-meteo`, `twitter`) — not omitted, not paraphrased.
- [ ] The action segment (`tool_name_fqn_tail`) is the canonical endpoint or method name for the named service when one exists (e.g. Open-Meteo `/forecast` → `forecast`, OpenAI `chat/completions` → `chat-completion`) — not a paraphrase of the description prose.
- [ ] `tool_name` (= directory name) is `"-".join(fqn_prefix.segments_past_workspace_root + [action])`. With segments, that's e.g. `weather-open-meteo-forecast`. Without segments, it's just the action. No silent fallback either way.
- [ ] In workspace mode, the **workspace root prefix** matches what `grep -rEh 'fqn!\(' <tools_root>/*/src/` shows existing tools use. In a brand-new repo, the root falls back to `com.example` **and** auto mode printed a warning to update it before publishing.
- [ ] In workspace mode, if the assembled FQN lands inside a known multi-tool crate's namespace, the user was warned and either confirmed a new sibling crate or aborted to extend the existing crate manually. The skill did not silently modify an existing multi-tool crate.
- [ ] `tool_name_fqn_tail` case style: kebab-case by default; snake_case only if at least one existing FQN tail in the workspace uses `_`. (A new repo gets kebab.)
