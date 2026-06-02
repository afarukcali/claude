#![doc = include_str!("../README.md")]

use nexus_toolkit::bootstrap;

mod __TOOL_NAME_SNAKE__;

#[tokio::main]
async fn main() {
    // Initialise the logger before validate_config so its log::error! calls
    // are visible. bootstrap! also calls try_init() internally; the second
    // call is a no-op.
    let _ = nexus_toolkit::env_logger::try_init();
    __TOOL_NAME_SNAKE__::validate_config();
    bootstrap!([__TOOL_NAME_SNAKE__::__TOOL_NAME_PASCAL__])
}
