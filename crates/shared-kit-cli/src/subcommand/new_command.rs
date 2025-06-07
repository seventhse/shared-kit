use anyhow::{Context, Ok};
use clap::Args;
use std::env;
use std::path::PathBuf;

use crate::components::new_command::{ensure_target_directory, ensure_template_selected};
use crate::components::progress::copy_directory_with_progress;
use crate::config::Config;
use crate::constant::TemplateKind;
use crate::helper::path::expand_dir;
use crate::helper::repo::resolve_repo_to_dir;

#[derive(Args, Debug)]
pub struct NewCommand {
    /// Name of the new project
    pub name: String,

    /// Optional kind of template to filter (project, monorepo, package)
    #[arg(short = 'k', long = "kind", value_name = "KIND")]
    pub kind: Option<TemplateKind>,

    /// Direct local template path, bypass config & selection
    #[arg(short = 'p', long = "template", value_name = "TEMPLATE")]
    pub template: Option<String>,

    /// Direct remote repo URL, bypass config & selection
    #[arg(short = 'r', long = "repo", value_name = "REPO")]
    pub repo: Option<String>,

    /// Custom config file path (default: /home/(user)/.config/shared-kit-cli/new-config.toml)
    #[arg(short = 'c', long = "config", value_name = "CONFIG")]
    pub config: Option<String>,
}

pub fn new_command_action(config: &mut Config, args: &NewCommand) -> anyhow::Result<()> {
    if let Some(cfg) = &args.config {
        config.reload(Some(cfg.clone()))?;
    }

    let mut target = env::current_dir()?.join(&args.name);
    target = ensure_target_directory(target)?;

    info_msg!("📁 Project will be created in: '{}'", target.display());

    if try_apply_direct_template(&target, args.template.clone())? {
        return Ok(());
    }

    if try_apply_direct_repo(&target, args.repo.clone())? {
        return Ok(());
    }

    let new_template = ensure_template_selected(&config, args)?;

    if try_apply_direct_template(&target, new_template.template.clone())? {
        return Ok(());
    }

    if try_apply_direct_repo(&target, new_template.repo.clone())? {
        return Ok(());
    }

    Ok(())
}

fn try_apply_direct_template(target: &PathBuf, template: Option<String>) -> anyhow::Result<bool> {
    if template.is_none() {
        return Ok(false);
    }
    let template_path = template.unwrap();
    let path = expand_dir(&template_path)
        .with_context(|| format!("Failed to expand template path: {}", template_path))?;

    if !path.exists() {
        anyhow::bail!(
            "❌ Template path does not exist: '{}'. Please check the path and try again.",
            path.display()
        );
    }

    copy_directory_with_progress(&path, &target)?;

    Ok(true)
}

fn try_apply_direct_repo(target: &PathBuf, repo: Option<String>) -> anyhow::Result<bool> {
    if repo.is_none() {
        return Ok(false);
    }

    let repo_url = repo.unwrap();

    let repo = resolve_repo_to_dir(&repo_url)?;

    copy_directory_with_progress(&repo.root_dir, target)?;

    Ok(true)
}
