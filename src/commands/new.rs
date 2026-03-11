use std::{
  collections::BTreeMap,
  io::{BufRead, BufReader},
  process::{Command, Stdio},
};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::{Context, Result, bail, eyre};
use handlebars::Handlebars;
use owo_colors::OwoColorize as _;
use tap::Tap as _;
use walkdir::WalkDir;

use crate::{TEMPLATES_DIR, template::Manifest};

pub fn run(template_name: &str, out_str: &str) -> Result<()> {
  let (manifest, files_dir) = load_template(template_name)?;

  // Resolve to an absolute path to avoid relative path issues.
  let cwd = std::env::current_dir().wrap_err("failed to get current directory")?;
  let cwd = Utf8PathBuf::try_from(cwd).map_err(|_| eyre!("current directory path is not valid UTF-8"))?;
  let out_dir = cwd.join(out_str);

  // Check that the output directory does not already exist.
  if out_dir.exists() {
    bail!("directory `{out_dir}` already exists");
  }

  // Create a temp dir on the same filesystem as output.
  let (temp_dir, temp_path) = create_temp_dir(&out_dir)?;

  // Prepare handlebars template engine.
  let handlebars = Handlebars::new().tap_mut(|hb| {
    hb.set_strict_mode(true);
  });

  // Inject built-in variables.
  let package_name = out_dir.file_name().ok_or_else(|| eyre!("could not determine project name from `{out_str}`"))?;

  let manifest = manifest.tap_mut(|manifest| {
    manifest.variables.insert("package_name".into(), package_name.into());
  });

  // Scaffold into the temp dir.
  log::info!("{}", format_args!("Scaffolding project `{package_name}` from template `{}`...", manifest.template.name).cyan());
  scaffold(&manifest, &files_dir, &temp_path, &handlebars)?;

  // Atomically move temp dir to final output dir.
  let temp_path = temp_dir.keep();
  fs_err::rename(&temp_path, &out_dir) //
    .wrap_err("failed to move scaffolded project to output directory")?;

  log::info!("{}", format_args!("Done! Project scaffolded at `{out_dir}`.").green().bold());

  return Ok(());
}

fn scaffold(manifest: &Manifest, files_dir: &Utf8Path, output_dir: &Utf8Path, handlebars: &Handlebars) -> Result<()> {
  // Run pre commands.
  run_commands(&manifest.commands.pre, handlebars, &manifest.variables, output_dir)?;

  // Render template files and directories.
  for entry in WalkDir::new(files_dir.as_std_path()) {
    let entry = entry.wrap_err("failed to walk template directory")?;

    let entry_path = Utf8Path::from_path(entry.path());
    let entry_path = entry_path.ok_or_else(|| eyre!("non-UTF-8 path in template: `{}`", entry.path().display()))?;
    let relative_path = entry_path.strip_prefix(files_dir).wrap_err("failed to compute relative path")?;

    let dest = output_dir.join(relative_path);

    if entry.file_type().is_dir() {
      fs_err::create_dir_all(&dest)?;
      continue;
    }

    if let Some(parent) = dest.parent() {
      fs_err::create_dir_all(parent)?;
    }

    let bytes = fs_err::read(entry.path())?;

    match std::str::from_utf8(&bytes) {
      Ok(content) => {
        log::debug!("{}", format_args!("Rendering file `{relative_path}`").cyan());

        let rendered = handlebars
          .render_template(content, &manifest.variables)
          .wrap_err_with(|| format!("failed to render template file: `{relative_path}`"))?;

        fs_err::write(&dest, rendered)?;
      }

      Err(_) => {
        log::debug!("{}", format_args!("Copying binary file `{relative_path}`").cyan());

        fs_err::write(&dest, &bytes)?;
      }
    }
  }

  // Run post commands.
  run_commands(&manifest.commands.post, handlebars, &manifest.variables, output_dir)?;

  return Ok(());
}

fn run_commands(
  commands: &[String],
  handlebars: &Handlebars,
  data: &BTreeMap<String, String>,
  cwd: &Utf8Path,
) -> Result<()> {
  for cmd in commands {
    let rendered = handlebars
      .render_template(cmd, data) //
      .wrap_err_with(|| format!("failed to render command: `{cmd}`"))?;

    log::info!("{} {}", "$".dimmed(), rendered.bold());

    let quiet = !log::log_enabled!(log::Level::Info);

    let mut child = Command::new("sh")
      .arg("-c")
      .arg(&rendered)
      .current_dir(cwd)
      .stdout(if quiet { Stdio::null() } else { Stdio::piped() })
      .stderr(if quiet { Stdio::null() } else { Stdio::piped() })
      .spawn()
      .wrap_err_with(|| format!("failed to execute command: `{rendered}`"))?;

    if !quiet {
      let prefix = "  │ ".dimmed().to_string();

      // Stream stderr in a separate thread.
      let stderr = child.stderr.take().expect("captured stderr");
      let prefix_clone = prefix.clone();
      let stderr_thread = std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
          eprintln!("{prefix_clone}{line}");
        }
      });

      // Stream stdout.
      let stdout = child.stdout.take().expect("captured stdout");
      for line in BufReader::new(stdout).lines() {
        let line = line.wrap_err("failed to read command output")?;
        eprintln!("{prefix}{line}");
      }

      stderr_thread.join().expect("stderr thread panicked");
    }

    let status = child.wait().wrap_err_with(|| format!("failed to wait for command: `{rendered}`"))?;

    if !status.success() {
      bail!("command `{rendered}` failed with {status}");
    }
  }

  return Ok(());
}

fn create_temp_dir(near: &Utf8Path) -> Result<(tempfile::TempDir, Utf8PathBuf)> {
  let parent = near.parent().expect("absolute path always has a parent");

  fs_err::create_dir_all(parent)?;

  let temp_dir = tempfile::tempdir_in(parent.as_std_path()) //
    .wrap_err("failed to create temporary directory")?;

  let temp_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf()) //
    .map_err(|_| eyre!("temporary directory path is not valid UTF-8"))?;

  return Ok((temp_dir, temp_path));
}

fn load_template(template_id: &str) -> Result<(Manifest, Utf8PathBuf)> {
  // Check if the specific template directory exists.
  let template_dir = TEMPLATES_DIR.join(template_id);

  if !template_dir.exists() {
    bail!("template `{template_id}` not found at `{template_dir}`\nRun `shkaf list` to see available templates.");
  }

  // Guard against path traversal (e.g. `../../etc`).
  let canonical_templates = TEMPLATES_DIR.canonicalize_utf8().wrap_err("failed to resolve templates directory")?;
  let canonical = template_dir
    .canonicalize_utf8()
    .wrap_err_with(|| format!("failed to resolve template path `{template_dir}`"))?;

  if !canonical.starts_with(&canonical_templates) {
    bail!("template id `{template_id}` escapes the templates directory");
  }

  // Check if the template manifest exists.
  let manifest_path = template_dir.join("template.toml");

  if !manifest_path.exists() {
    bail!("template `{template_id}` is missing `template.toml`");
  }

  // Check if the template files directory exists.
  let files_dir = template_dir.join("files");

  if !files_dir.exists() {
    bail!("template `{template_id}` is missing `files/` directory");
  }

  // Parse the template manifest.
  log::debug!("{}", format_args!("Loading manifest from `{manifest_path}`").cyan());

  let manifest = fs_err::read_to_string(&manifest_path)?;
  let manifest: Manifest = toml::from_str(&manifest) //
    .wrap_err("failed to parse template manifest")?;

  return Ok((manifest, files_dir));
}
