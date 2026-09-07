use clap::Subcommand;

#[derive(Debug, Subcommand, PartialEq, Clone)]
pub enum RemoveSubcommand {
    /// Will remove specified remote Knot with CLI
    Remote,
}
