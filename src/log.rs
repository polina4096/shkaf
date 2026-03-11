use log::{Level, LevelFilter};
use owo_colors::OwoColorize;

pub fn init(verbose: bool, quiet: bool) -> color_eyre::Result<()> {
  fern::Dispatch::new()
    .format(|out, message, record| {
      match record.level() {
        Level::Info => out.finish(format_args!("{message}")),
        Level::Debug => out.finish(format_args!("[{}] {message}", "debug".dimmed())),
        Level::Warn => out.finish(format_args!("[{}] {message}", "warn".yellow().bold())),
        Level::Error => out.finish(format_args!("[{}] {message}", "error".red().bold())),
        Level::Trace => out.finish(format_args!("[{}] {message}", "trace".dimmed())),
      }
    })
    .level(match () {
      _ if quiet => LevelFilter::Off,
      _ if verbose => LevelFilter::Debug,
      _ => LevelFilter::Info,
    })
    .chain(std::io::stderr())
    .apply()?;

  return Ok(());
}
