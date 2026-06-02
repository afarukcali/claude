//! `__FQN_PREFIX__.__TOOL_NAME_FQN_TAIL__@1`
//!
//! __DESCRIPTION__

use nexus_sdk::{fqn, ToolFqn};
use nexus_toolkit::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

// ── config ────────────────────────────────────────────────────────────────────
//
// All required env vars are READ AT STARTUP by validate_config() and cached
// in module-level OnceLock<String> statics. Accessors return the cached value
// (no further env reads). Process aborts at startup if any var is missing —
// fail-fast over silent runtime failures.
//
// To add a new required env var:
//   1. Declare `static <NAME>: OnceLock<String> = OnceLock::new();`
//   2. Add `<NAME>.set(load_required("<NAME>")).expect("validate_config called twice");`
//      to validate_config()
//   3. Add an accessor function returning &'static str

static EXAMPLE_API_KEY: OnceLock<String> = OnceLock::new();

/// Reads and caches all required env vars. Called from main() before bootstrap!.
/// Assumes env_logger is already initialised.
pub(crate) fn validate_config() {
    EXAMPLE_API_KEY
        .set(load_required("EXAMPLE_API_KEY"))
        .expect("validate_config called twice");
    // TODO: add one `<STATIC>.set(load_required("<VAR>")).expect(...);` line per secret
}

/// Reads a required env var or aborts the process. Logs the name (never the value).
fn load_required(name: &str) -> String {
    match std::env::var(name) {
        Ok(v) => {
            log::debug!(target: "__TOOL_NAME_SNAKE__", "env var {name} loaded");
            v
        }
        Err(_) => {
            log::error!(target: "__TOOL_NAME_SNAKE__", "fatal: required env var {name} is not set");
            std::process::exit(1);
        }
    }
}

#[allow(dead_code)] // used by invoke() once real logic is wired
fn example_api_key() -> &'static str {
    EXAMPLE_API_KEY
        .get()
        .expect("validate_config must run before any accessor")
}

// ── types ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    // TODO: replace with real input fields
    pub(crate) placeholder: String,
}

// TODO: replace Ok's fields with real output ports; add domain-specific error
// variants for distinct failure modes (e.g. err_not_found, err_rate_limited,
// err_invalid_input) so DAG edges can route on specific failure types.
#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Output {
    #[allow(dead_code)] // remove when invoke() actually returns Ok
    Ok { result: String },
    ErrUpstream { reason: String },
    #[allow(dead_code)] // remove when invoke() actually returns ErrConfig
    ErrConfig { reason: String },
}

pub(crate) struct __TOOL_NAME_PASCAL__;

// ── impl ──────────────────────────────────────────────────────────────────────

impl NexusTool for __TOOL_NAME_PASCAL__ {
    type Input = Input;
    type Output = Output;

    async fn new() -> Self {
        Self
    }

    fn fqn() -> ToolFqn {
        // Generic workspace / standalone:
        fqn!("__FQN_PREFIX__.__TOOL_NAME_FQN_TAIL__@1")
        // nexus-tools mode — uncomment the line below and remove the line above:
        // fqn!(concat!("__FQN_PREFIX__.__TOOL_NAME_FQN_TAIL__@", env!("TOOL_FQN_VERSION")))
    }

    fn path() -> &'static str {
        "/__TOOL_NAME_FQN_TAIL__" // explicitly overrides the trait default ("")
    }

    fn description() -> &'static str {
        "__DESCRIPTION__"
    }

    async fn health(&self) -> AnyResult<StatusCode> {
        // TODO: probe every service this tool depends on.
        // Return Err(...) if any dependency is unhealthy — leader nodes use
        // this endpoint to decide whether to route invocations.
        Ok(StatusCode::OK)
    }

    async fn invoke(&self, input: Self::Input) -> Self::Output {
        let Input { placeholder } = input;
        log::debug!(target: "__TOOL_NAME_SNAKE__", "invoke called: placeholder={:?}", placeholder);

        // TODO: access secrets via the module-level accessors (e.g.
        // example_api_key()). Never call std::env::var() directly here —
        // that defeats the fail-fast guarantee from validate_config and
        // re-reads the env on every request.
        //
        // TODO: set an explicit timeout on every external call; a slow upstream
        // will hold this invocation open indefinitely otherwise.
        //
        // TODO: implement real logic; return the appropriate Output variant.
        // Do NOT call example_api_key() until you also arrange for
        // validate_config to run in your tests (or refactor the call site).
        Output::ErrUpstream {
            reason: format!("not implemented yet (placeholder={placeholder:?})"),
        }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the unimplemented scaffold returns ErrUpstream; catches
    /// regressions if the variant name or return type changes.
    #[tokio::test]
    async fn invoke_returns_err_upstream() {
        let tool = __TOOL_NAME_PASCAL__::new().await;
        let input = Input { placeholder: "test".to_string() };
        let output = tool.invoke(input).await;
        assert!(matches!(output, Output::ErrUpstream { .. }));
    }

    /// Verifies health() returns 200 OK on the unimplemented scaffold;
    /// catches regressions before real dependency checks are wired.
    #[tokio::test]
    async fn health_returns_ok() {
        let tool = __TOOL_NAME_PASCAL__::new().await;
        let status = tool.health().await.unwrap();
        assert_eq!(status, StatusCode::OK);
    }
}
