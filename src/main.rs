mod cli;
mod commands;
mod log;
mod template;

use std::sync::LazyLock;

use camino::Utf8PathBuf;
use clap::Parser;
use etcetera::BaseStrategy;

use crate::cli::{Cli, Command};

pub const APP_NAME: &str = "shkaf";

pub static CONFIG_DIR: LazyLock<Utf8PathBuf> = LazyLock::new(|| {
  let strategy = etcetera::choose_base_strategy() //
    .expect("failed to determine config directory");

  return Utf8PathBuf::try_from(strategy.config_dir())
    .expect("config directory path is not valid UTF-8")
    .join(APP_NAME);
});

pub static TEMPLATES_DIR: LazyLock<Utf8PathBuf> = LazyLock::new(|| CONFIG_DIR.join("templates"));

fn main() -> color_eyre::Result<()> {
  color_eyre::install()?;

  let cli = Cli::parse();

  log::init(cli.verbose, cli.quiet)?;

  match cli.command {
    Command::New { template, name } => commands::new::run(&template, &name)?,
    Command::List => commands::list::run()?,
  }

  return Ok(());
}
