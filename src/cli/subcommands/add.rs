use clap::Subcommand;

#[derive(Debug, Subcommand, PartialEq, Clone)]
pub enum AddSubcommand {
    /// For adding new remote Knot
    Remote,
}
