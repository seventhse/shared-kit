use anyhow::{Context, Ok};
use atty::Stream;
use inquire::{Select, Text};
use std::fmt::Display;
use std::io::{self, BufRead};
use std::path::PathBuf;

use crate::config::Config;
use crate::constant::TemplateItem;
use crate::helper::matcher::PatternSpec;
use crate::helper::matcher_group::ResolvedVar;
use crate::subcommand::new_command::NewCommand;

#[derive(Debug, Clone)]
enum TargetDirExistAction {
    Rename,
    Overwrite,
    Cancel,
}

impl Display for TargetDirExistAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            TargetDirExistAction::Rename => "🔁 Rename the project directory",
            TargetDirExistAction::Overwrite => "🧹 Overwrite the existing directory",
            TargetDirExistAction::Cancel => "❌ Cancel operation",
        };
        write!(f, "{}", label)
    }
}

pub fn ensure_target_directory(mut target: PathBuf) -> anyhow::Result<PathBuf> {
    while target.exists() {
        let choices = vec![
            TargetDirExistAction::Rename,
            TargetDirExistAction::Overwrite,
            TargetDirExistAction::Cancel,
        ];

        let selected =
            Select::new("⚠️ Target directory already exists. What would you like to do?", choices)
                .prompt()
                .with_context(|| "Failed to get user selection")?;

        match selected {
            TargetDirExistAction::Rename => {
                let new_name = Text::new("Please enter a new project name:")
                    .prompt()
                    .with_context(|| "Failed to read new project name")?;
                target = std::env::current_dir()
                    .with_context(|| "Failed to get current directory")?
                    .join(new_name);
            }
            TargetDirExistAction::Overwrite => {
                std::fs::remove_dir_all(&target)
                    .with_context(|| format!("Failed to remove directory: {}", target.display()))?;
                break;
            }
            TargetDirExistAction::Cancel => {
                anyhow::bail!("Operation canceled by user.");
            }
        }
    }

    Ok(target)
}

pub fn ensure_template_selected(
    config: &Config,
    args: &NewCommand,
) -> anyhow::Result<TemplateItem> {
    let available_templates = config.metadata.get_templates(args.kind.clone());

    if available_templates.is_empty() {
        let config_path_display = config
            .current_config_path
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "<none>".to_string());

        anyhow::bail!(
            "❌ No templates found in config '{}'. Please check your config file or use --template/--repo to specify a template directly.",
            config_path_display
        );
    }

    let options: Vec<String> = available_templates.keys().map(|name| name.to_string()).collect();

    let selected = if atty::is(Stream::Stdin) {
        // 正常交互
        inquire::Select::new("📦 Select a template to use", options.clone())
            .prompt()
            .with_context(|| "Failed to select a template")?
    } else {
        // 非交互测试：从 stdin 模拟读取
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines();
        let input = lines
            .next()
            .transpose()
            .context("Failed to read simulated input from stdin")?
            .unwrap_or_default();
        if !options.contains(&input) {
            anyhow::bail!("Invalid simulated input: '{}'", input);
        }
        input
    };

    let template = available_templates
        .get(&selected)
        .with_context(|| format!("Template '{}' not found in config metadata", selected))?;

    Ok(template.clone())
}

pub fn ensure_replace_var_input(template: &TemplateItem) -> anyhow::Result<Vec<ResolvedVar>> {
    let mut resolved_vars: Vec<ResolvedVar> = vec![];

    if let Some(vars) = &template.template_vars {
        for var in vars {
            let placeholder = var.placeholder.clone();
            let message = var
                .prompt
                .clone()
                .unwrap_or_else(|| format!("Enter new value for {}", placeholder));

            let default = var.default.clone();

            let input = if let Some(default_val) = default {
                Text::new(&message).with_initial_value(&default_val).prompt().unwrap_or(default_val)
            } else {
                Text::new(&message).prompt().unwrap_or_else(|_| "".to_string())
            };

            let resolve_var = ResolvedVar {
                placeholder: placeholder,
                replacement: input,
                includes: PatternSpec::from_option_vec(var.includes_paths.clone()),
                excludes: PatternSpec::from_option_vec(var.excludes_paths.clone()),
            };

            resolved_vars.push(resolve_var);
        }
    }

    Ok(resolved_vars)
}
