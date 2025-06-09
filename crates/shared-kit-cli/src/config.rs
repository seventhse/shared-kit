use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
};

use anyhow::{Context, Ok, Result};
use console::style;
use serde::{Deserialize, Serialize};

use crate::{
    constant::{DEFAULT_CONFIG_DIR, DEFAULT_CONFIG_FILENAME, TemplateKind, Templates},
    helper::path::expand_dir,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ConfigMetadata {
    pub templates: Templates,
}

impl ConfigMetadata {
    pub fn get_templates(&self, kind: Option<TemplateKind>) -> Templates {
        match kind {
            Some(k) => self
                .templates
                .iter()
                .filter(|(_, template)| template.kind == k)
                .map(|(name, template)| (name.clone(), template.clone()))
                .collect(),
            None => self.templates.clone(),
        }
    }
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        ConfigMetadata { templates: HashMap::new() }
    }
}

#[derive(Debug)]
pub struct Config {
    pub current_config_path: Option<PathBuf>,
    pub metadata: ConfigMetadata,
}

impl Config {
    pub fn from_path(path: Option<String>) -> Result<Self> {
        let (config_path, metadata) = Config::parse_config(path)?;
        Ok(Config { current_config_path: config_path, metadata: metadata })
    }

    pub fn reload(self: &mut Self, path: Option<String>) -> Result<()> {
        let (config_path, metadata) = Config::parse_config(path)?;
        self.current_config_path = config_path;
        self.metadata = metadata;

        Ok(())
    }

    fn parse_config(path: Option<String>) -> anyhow::Result<(Option<PathBuf>, ConfigMetadata)> {
        let config_path = path.as_deref().and_then(expand_dir).or_else(get_default_config_path);

        let config_path = match config_path {
            Some(p) => p,
            None => {
                info_msg!("No config path provided; using default configuration.");
                return Ok((config_path, ConfigMetadata::default()));
            }
        };

        if !config_path.exists() {
            warn_msg!(
                "Config file not found at: {:?}",
                style(&config_path.display().to_string()).yellow()
            );
            return Ok((Some(config_path), ConfigMetadata::default()));
        }

        let metadata = parse_config(&config_path)
            .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

        Ok((Some(config_path), metadata))
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            current_config_path: get_default_config_path(),
            metadata: ConfigMetadata::default(),
        }
    }
}

pub fn get_default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join(DEFAULT_CONFIG_DIR).join(DEFAULT_CONFIG_FILENAME))
}

fn parse_config(path: &PathBuf) -> Result<ConfigMetadata> {
    if !&path.is_file() {
        anyhow::bail!("The config path is not a valid file: {:?}", path);
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file at {:?}", path))?;
    let config: ConfigMetadata = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config TOML from {:?}", path))?;

    Ok(config)
}
