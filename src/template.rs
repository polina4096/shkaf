use serde::Deserialize;
use smart_default::SmartDefault;

#[derive(Deserialize, SmartDefault)]
pub struct Manifest {
  #[serde(default)]
  pub template: TemplateInfo,
  #[serde(default)]
  pub commands: TemplateCommands,
}

#[derive(Deserialize, SmartDefault)]
pub struct TemplateInfo {
  #[default("my_template".into())]
  pub name: String,
  #[default("A project template".into())]
  pub description: String,
  #[default(whoami::username().unwrap_or("me".into()))]
  pub author: String,
}

#[derive(Deserialize, Default)]
pub struct TemplateCommands {
  #[serde(default)]
  pub pre: Vec<String>,
  #[serde(default)]
  pub post: Vec<String>,
}
