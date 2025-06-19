use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DEFAULT_CONFIG_DIR: &str = "shared-kit-cli";
pub const DEFAULT_CONFIG_FILENAME: &str = "metadata.toml";

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[clap(rename_all = "lowercase")]
pub enum TemplateKind {
    Project,
    Monorepo,
    Package,
}

/// Defines a single variable placeholder used in template replacements.
///
/// This struct supports interactive prompting, default values,
/// and path-based filtering to control where replacements apply.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TemplateVar {
    /// The placeholder string used in template files, e.g. `{{project_name}}`.
    pub placeholder: String,

    /// Optional prompt message shown to the user when filling this variable interactively.
    pub prompt: Option<String>,

    /// Optional default value if the user provides no input.
    pub default: Option<String>,

    /// Optional list of glob patterns specifying files where this variable **should** be replaced.
    pub includes_paths: Option<Vec<String>>,
}

/// A type alias for a list of template variables.
pub type TemplateVars = Vec<TemplateVar>;

/// Represents a single template configuration item.
///
/// Contains metadata about the template, including its kind,
/// source location (local path or remote repo),
/// file filtering rules, and replacement variables.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TemplateItem {
    /// The category/kind of the template.
    pub kind: TemplateKind,
    pub template: Option<String>,
    pub repo: Option<String>,
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
    pub template_vars: Option<TemplateVars>,
    pub completed_script: Option<Vec<String>>,
}

/// A map of template names to their corresponding `TemplateItem`.
///
/// This represents the overall configuration of available templates.
pub type Templates = HashMap<String, TemplateItem>;
