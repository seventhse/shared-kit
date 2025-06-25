use anyhow::{Context, Ok};
use atty::Stream;
use inquire::Text;
use shared_kit_common::output;
use std::fmt::Display;
use std::io::{self, BufRead};
use std::path::PathBuf;

use crate::components::ui::select_with_ui;
use crate::config::Config;
use crate::constant::TemplateItem;
use crate::helper::file_transform_middleware::FileMatcherItem;
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
            TargetDirExistAction::Rename => "Rename the project directory",
            TargetDirExistAction::Overwrite => "Overwrite the existing directory",
            TargetDirExistAction::Cancel => "Cancel operation",
        };
        write!(f, "{}", label)
    }
}

pub fn ensure_target_directory(mut target: PathBuf) -> anyhow::Result<PathBuf> {
    while target.exists() {
        output!(title: "⚠️ Target directory already exists \n");
        let choices = vec![
            TargetDirExistAction::Rename,
            TargetDirExistAction::Overwrite,
            TargetDirExistAction::Cancel,
        ];

        let selected = select_with_ui("What would you like to do?", choices)
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
                anyhow::bail!("❌ Operation canceled by user.");
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
            "❌ No templates found in config '{}'.\nPlease check your config file or use --template / --repo to specify a template.",
            config_path_display
        );
    }

    let options: Vec<String> = available_templates.keys().map(|name| name.to_string()).collect();

    output!(title: "📦 Select a template");
    output!(space);
    let selected = if atty::is(Stream::Stdin) {
        select_with_ui("Choose one of the following templates:", options.clone())
            .with_context(|| "Failed to select a template")?
    } else {
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines();
        let input = lines
            .next()
            .transpose()
            .context("Failed to read simulated input from stdin")?
            .unwrap_or_default();
        if !options.contains(&input) {
            anyhow::bail!("❌ Invalid template name in simulated input: '{}'", input);
        }
        input
    };

    let template = available_templates
        .get(&selected)
        .with_context(|| format!("Template '{}' not found in config metadata", selected))?;

    Ok(template.clone())
}

pub fn ensure_replace_var_input(template: &TemplateItem) -> anyhow::Result<Vec<FileMatcherItem>> {
    let mut file_matcher_items: Vec<FileMatcherItem> = vec![];

    if let Some(vars) = &template.template_vars {
        if vars.is_empty() {
            output!(title: "🔧 No variables to configure");
            output!(line: "Using default values. Skipping input step.");
        } else {
            output!(title: "🔧 Configure Template Variables");
            output!(space);
            for var in vars {
                let placeholder = var.placeholder.clone();

                let prompt_message = var
                    .prompt
                    .clone()
                    .unwrap_or_else(|| format!("Please enter a value for `{}`", placeholder));

                let default_val = var.default.clone();

                let message = if let Some(ref default_str) = default_val {
                    format!("{} [default: {}]", prompt_message, default_str)
                } else {
                    prompt_message
                };

                let input = if let Some(default_val) = default_val {
                    Text::new(&message)
                        .with_initial_value(&default_val)
                        .prompt()
                        .unwrap_or(default_val)
                } else {
                    Text::new(&message).prompt().unwrap_or_else(|_| "".to_string())
                };

                file_matcher_items.push(FileMatcherItem {
                    pattern_val: placeholder,
                    includes: var.includes_paths.clone().unwrap_or_default(),
                    replace_val: input,
                });
            }
        }
    }

    Ok(file_matcher_items)
}
