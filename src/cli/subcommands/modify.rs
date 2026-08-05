use clap::Subcommand;

#[derive(Debug, Subcommand, PartialEq, Clone)]
pub enum ModifySubcommand {
    /// Configure caching feature settings
    Caching,

    /// Configure file compression feature settings
    Compression,

    /// Configure gitignore integration settings
    Gitignore,

    /// Manage global ignore patterns (append or rewrite)
    IgnorePatterns,

    /// Enable or disable the file size limit setting
    AllowSizeLimit,

    /// Set or update the maximum file size limit
    SizeLimit,

    /// Modify properties of the local source knot configuration
    Source {
        #[command(subcommand)]
        properties: KnotModifySubcommand,
    },

    /// Modify properties of a remote knot configuration
    Remote {
        #[command(subcommand)]
        properties: KnotModifySubcommand,

        /// Target index when multiple remote knots exist
        ///
        /// Prompts interactively if omitted
        #[arg(short = 'i', long)]
        index: Option<usize>,
    },
}

#[derive(Debug, Subcommand, PartialEq, Clone)]
pub enum KnotModifySubcommand {
    /// Set the adapter type for the knot (e.g., Local, Remote)
    Type,

    /// Update the directory or remote path for the knot
    Path,

    /// Set the maximum concurrent connection limit
    Connections,

    /// Interactively set or update knot authentication credentials
    Credentials,

    /// Update the authentication username
    Username,

    /// Update the network connection port
    Port,

    /// Update the host address or domain name
    Host,

    /// Update your authentication method
    Auth,

    /// Behavior of remote knot on conflict files
    ConflictBehavior,

    /// Behavior of remote knot on unique files
    UniqueBehavior,

    /// Rewrite/Delete your password
    Password,
}
