use inquire::{
    Select,
    ui::{RenderConfig, Styled},
};
use shared_kit_common::console::style;

pub fn select_with_ui<T: std::fmt::Display + Clone>(
    prompt: &str,
    choices: Vec<T>,
) -> anyhow::Result<T> {
    let mut render_config = RenderConfig::default();

    let prefix = style("👉").cyan().bold().to_string();
    render_config.prompt_prefix = Styled::new(&prefix); // 不调用 to_string()
    let highlighted = style("➤").green().bold().to_string();
    render_config.highlighted_option_prefix = Styled::new(&highlighted);

    let selected = Select::new(prompt, choices)
        .with_render_config(render_config)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Failed to select: {}", e))?;

    Ok(selected)
}
