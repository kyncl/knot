pub mod configuration;
pub mod remote;

pub const REMOTE_CONFIG_HELP_INFO: &str = r#"# === CONFIGURATION INFO ===
# All values that are string can be:
# - kebab-case
# - cammel_case
# - PascalCase
# - lowercase
# - space case
#
# Knot types:
# - Local
# - SSH
#
# Behaviors:
#   - Conflict Behavior (Knot found same files, but different content)
#       - Newer (Prioritize newer based on modified time)
#       - Older (Prioritize older based on modified time)
#       - Source (Prioritize source directory)
#       - Remote (Prioritize remote directory)
#       - Ask (Ask user, how to handle it with TUI)
#       - Skip (Skips all conflict files)
#
#   - Unique Behavior (How to handle unique files inside each directories)
#       - Archive
#       - MirrorSource
#       - MirrorRemote
#       - Ask
#       - Skip
#       - OnlyAdd

"#;

pub const CONFIG_TEMPLATE: &str = r#"# === CONFIGURATION INFO ===
# All values that are string can be:
# - kebab-case
# - cammel_case
# - PascalCase
# - lowercase
# - space case
#
# Knot types:
# - Local
# - SSH

[config.performance]
# Maximum size limit (e.g. "15.00 GB")
size_limit = "15.00 GiB"
allow_size_limit = false

[config.features]
# Enable cache layer
caching = false
# Respect .gitignore files
gitignore = false
# Enable response compression
compress = false

[source]
# Adapter driver type
type = "Local"
# Path to working directory
path = "./"
"#;
