use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "shkaf", about = "Scaffold projects from templates")]
pub struct Cli {
  /// Enable verbose logging
  #[arg(short, long, global = true)]
  pub verbose: bool,

  /// Suppress all output
  #[arg(short, long, global = true, conflicts_with = "verbose")]
  pub quiet: bool,

  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
  /// Scaffold a new project from a template
  New {
    /// Id of the template
    template: String,

    /// Path to the project
    name: String,
  },

  /// List available templates
  List,
}
