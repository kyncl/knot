use std::env;
use std::io::{self, IsTerminal};

/// Returns `true` if the app is likely running in a CI
/// or non-interactive environment.
pub fn is_ci_environment() -> bool {
    if !io::stdout().is_terminal() {
        return true;
    }

    if let Ok(ci_val) = env::var("CI") {
        let ci_lower = ci_val.to_lowercase();
        if ci_lower == "1" || ci_lower == "true" {
            return true;
        }
    }

    env::var("TERM")
        .map(|v| v.to_lowercase() == "dumb")
        .unwrap_or(false)
}
