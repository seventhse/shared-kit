use anyhow::{Context, Ok};
use clap::Args;
use shared_kit_common::matcher::{Matcher, MatcherBuilder};
use shared_kit_common::{log_info, log_warn};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use crate::components::new_command::{
    ensure_replace_var_input, ensure_target_directory, ensure_template_selected,
};
use crate::components::progress::copy_directory_with_progress;
use crate::config::Config;
use crate::constant::{TemplateItem, TemplateKind};
use crate::helper::file_transform_middleware::FileMatcherItem;
use crate::helper::repo::resolve_repo_to_dir;
use shared_kit_common::file_utils::path::compose_path;

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

    log_info!("📁 Project will be created in: '{}'", target.display());

    if try_apply_direct_template(&target, args.template.clone(), config, None)? {
        return Ok(());
    }

    if try_apply_direct_repo(&target, args.repo.clone(), None)? {
        return Ok(());
    }

    let new_template_item = ensure_template_selected(&config, args)?;

    let file_matches = ensure_replace_var_input(&new_template_item)
        .with_context(|| format!("Failed to input replace var"))?;

    try_apply_direct(&target, new_template_item, file_matches, &config)
}

fn try_apply_direct(
    target: &PathBuf,
    template_item: TemplateItem,
    file_matches: Vec<FileMatcherItem>,
    config: &Config,
) -> anyhow::Result<()> {
    let mut matcher_builder: MatcherBuilder<FileMatcherItem> = MatcherBuilder::new()
        .with_exclude_strs_opt(template_item.includes, None)
        .with_exclude_strs_opt(template_item.excludes, None);

    for file_matcher in file_matches {
        matcher_builder = matcher_builder
            .with_include_strs(file_matcher.includes.clone(), Some(file_matcher.clone()));
    }

    let matcher = Arc::new(matcher_builder.build());

    let mut result =
        try_apply_direct_template(target, template_item.template, config, Some(matcher.clone()))?;

    if !result {
        result = try_apply_direct_repo(target, template_item.repo, Some(matcher.clone()))?;
    }

    if result && template_item.completed_script.is_some() {
        let _computed_script = template_item.completed_script.unwrap();
        todo!("exec computed script")
    }

    Ok(())
}

fn try_apply_direct_template(
    target: &PathBuf,
    template: Option<String>,
    config: &Config,
    matcher: Option<Arc<Matcher<FileMatcherItem>>>,
) -> anyhow::Result<bool> {
    if template.is_none() {
        return Ok(false);
    }
    let template_path = PathBuf::from(template.unwrap());
    let current_config_path = config.current_config_path.clone().unwrap();
    let path = compose_path(&current_config_path.parent().unwrap(), &template_path);

    if path.is_none() {
        log_warn!("Template path is error, please check.");

        return Ok(false);
    }

    let path = path.unwrap();

    if !path.exists() {
        log_warn!(
            "Template path does not exist: '{}'. Please check the path and try again.",
            path.display()
        );

        return Ok(false);
    }

    copy_directory_with_progress(&path, &target, matcher)?;

    Ok(true)
}

fn try_apply_direct_repo(
    target: &PathBuf,
    repo: Option<String>,
    matcher: Option<Arc<Matcher<FileMatcherItem>>>,
) -> anyhow::Result<bool> {
    if repo.is_none() {
        return Ok(false);
    }

    let repo_url = repo.unwrap();

    let repo = resolve_repo_to_dir(&repo_url)?;

    copy_directory_with_progress(&repo.root_dir, target, matcher)?;

    Ok(true)
}
