use std::collections::HashMap;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub const DEFAULT_CONFIG_DIR: &str = "shared-kit-cli";
pub const DEFAULT_CONFIG_FILENAME: &str = "metadata.toml";

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq, Deserialize, Serialize)]
pub enum TemplateKind {
    Project,
    Monorepo,
    Package,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TemplateItem {
    pub kind: TemplateKind,
    pub template: Option<String>,
    pub repo: Option<String>,
}

pub type Templates = HashMap<String, TemplateItem>;

