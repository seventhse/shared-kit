use anyhow::{Context, Ok};
use clap::Args;
use shared_kit_common::matcher::{Matcher, MatcherBuilder};
use shared_kit_common::{log_warn, output};
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
use crate::helper::run_scripts::run_completed_scripts;
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
    // 🚀 Step 1: 加载配置文件
    if let Some(cfg) = &args.config {
        output!(title: "📄 Loading Configuration");
        config.reload(Some(cfg.clone()))?;
    }

    // 📁 Step 2: 解析目标路径
    let mut target = env::current_dir()?.join(&args.name);
    target = ensure_target_directory(target)?;

    output!(space);
    output!(title: "📁 Project Target Directory");
    output!(line: "Project will be created in: {}", target.display());

    // ⚡ Step 3: 使用指定模板或仓库直接生成（短路径）
    if try_apply_direct_template(&target, args.template.clone(), config, None)? {
        output!(space);
        output!(line: "✅ Project created from template.");
        return Ok(());
    }

    if try_apply_direct_repo(&target, args.repo.clone(), None)? {
        output!(space);
        output!(line: "✅ Project created from remote repo.");
        return Ok(());
    }

    // 📦 Step 4: 交互式选择模板
    output!(space);
    let new_template_item = ensure_template_selected(&config, args)?;

    // 🔤 Step 5: 收集变量
    let file_matches = ensure_replace_var_input(&new_template_item)
        .with_context(|| format!("❌ Failed to input replace variables"))?;

    // 🛠️ Step 6: 应用模板
    output!(space);
    output!(title: "🛠️ Applying Template");
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

    for file_matcher in &file_matches {
        matcher_builder = matcher_builder
            .with_include_strs(file_matcher.includes.clone(), Some(file_matcher.clone()));
    }

    let matcher = Arc::new(matcher_builder.build());

    // 🧱 应用本地模板
    if try_apply_direct_template(
        target,
        template_item.template.clone(),
        config,
        Some(matcher.clone()),
    )? {
        output!(line: "✅ Project created from local template.");
    }
    // 🌍 或从远程仓库拉取
    else if try_apply_direct_repo(target, template_item.repo.clone(), Some(matcher.clone()))? {
        output!(line: "✅ Project created from remote repository.");
    } else {
        output!(line: "❌ Failed to apply template or repo.");
        return Ok(()); // 或考虑 Err
    }

    // 🧩 后处理脚本执行（预留）
    if let Some(completed_script) = template_item.completed_script {
        output!(title: "🎯 Running post-generation script");
        output!(space);
        run_completed_scripts(&completed_script, &target)?;
    }

    Ok(())
}

fn try_apply_direct_template(
    target: &PathBuf,
    template: Option<String>,
    config: &Config,
    matcher: Option<Arc<Matcher<FileMatcherItem>>>,
) -> anyhow::Result<bool> {
    let Some(template) = template else {
        return Ok(false);
    };

    let current_config_path = match &config.current_config_path {
        Some(path) => path,
        None => {
            log_warn!("❌ Current config path is not set.");
            return Ok(false);
        }
    };

    let path = compose_path(&current_config_path.parent().unwrap(), &PathBuf::from(&template));

    let Some(template_path) = path else {
        log_warn!("❌ Template path is invalid.");
        return Ok(false);
    };

    if !template_path.exists() {
        log_warn!("❌ Template path does not exist: '{}'", template_path.display());
        return Ok(false);
    }

    output!(title: "📦 Applying local template");
    output!(line: "→ From: {}", template_path.display());
    output!(line: "→ To:   {}", target.display());

    copy_directory_with_progress(&template_path, target, matcher)?;

    Ok(true)
}

fn try_apply_direct_repo(
    target: &PathBuf,
    repo: Option<String>,
    matcher: Option<Arc<Matcher<FileMatcherItem>>>,
) -> anyhow::Result<bool> {
    let Some(repo_url) = repo else {
        return Ok(false);
    };

    output!(title: "🌍 Cloning template from remote repository");
    output!(line: "→ {}", repo_url);

    let repo = resolve_repo_to_dir(&repo_url)?;

    copy_directory_with_progress(&repo.root_dir, target, matcher)?;

    Ok(true)
}
