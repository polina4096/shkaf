use camino::Utf8Path;
use color_eyre::eyre::{Context, Result};
use owo_colors::OwoColorize as _;

use crate::{TEMPLATES_DIR, template::Manifest};

pub fn run() -> Result<()> {
  let templates_path = &*TEMPLATES_DIR;

  // Check if templates directory exists.
  if !templates_path.exists() {
    log::info!("{} Create templates in `{}`", "No templates found.".yellow(), templates_path.bold());

    return Ok(());
  }

  let mut templates = collect_templates(templates_path)?;

  // Check if there are no templates.
  if templates.is_empty() {
    log::info!("{} Create templates in `{}`", "No templates found.".yellow(), templates_path.bold());

    return Ok(());
  }

  // Sort templates alphabetically by ID.
  templates.sort_by(|(a, _), (b, _)| a.cmp(b));

  log::info!("{}", format_args!("Available templates ({}):", templates.len()).bold());

  print_templates(&templates);

  return Ok(());
}

fn collect_templates(templates_path: &Utf8Path) -> Result<Vec<(String, Manifest)>> {
  let mut templates: Vec<(String, Manifest)> = Vec::new();

  for entry in fs_err::read_dir(templates_path)? {
    let entry = entry.wrap_err("failed to walk stored templates directory")?;

    match entry.file_name().into_string() {
      Ok(name) if entry.file_type()?.is_dir() => {
        let template_dir = templates_path.join(&name);
        let manifest_path = template_dir.join("template.toml");

        // Skip directories without a manifest file.
        if !manifest_path.exists() {
          log::trace!("No manifest file found in `{}`", template_dir);

          continue;
        }

        let manifest = fs_err::read_to_string(&manifest_path)?;
        let manifest: Manifest = toml::from_str(&manifest) //
          .wrap_err("failed to parse template manifest")?;

        templates.push((name, manifest));
      }

      Ok(name) => {
        let path = templates_path.join(&name);
        log::trace!("Skipping non-directory file `{}`", path);
      }

      Err(name) => {
        let path = templates_path.join_os(name);
        log::trace!("Skipping invalid directory `{:?}`", path);
      }
    }
  }

  return Ok(templates);
}

fn print_templates(templates: &[(String, Manifest)]) {
  let max_id_width = templates.iter().map(|(id, _)| id.len()).max().unwrap_or(0);

  for (id, manifest) in templates {
    let info = &manifest.template;

    log::info!(
      "  {:<max_id_width$} {} {} {} {}",
      id.bold().cyan(),
      info.name,
      format_args!("({})", info.author).dimmed(),
      "-".dimmed(),
      info.description.dimmed(),
    );
  }
}
