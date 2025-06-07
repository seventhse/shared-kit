use predicates::prelude::*;
use shared_kit_cli::config::{Config, ConfigMetadata};
use shared_kit_cli::constant::{TemplateItem, TemplateKind, Templates};
use shared_kit_cli::subcommand::new_command::{NewCommand, new_command_action};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn dummy_config_with_template(template_path: PathBuf) -> Config {
    let mut map: Templates = HashMap::new();

    map.insert(
        "test-template".to_string(),
        TemplateItem {
            kind: TemplateKind::Project,
            template: Some(template_path.to_string_lossy().to_string()),
            repo: None,
        },
    );

    Config { metadata: ConfigMetadata { templates: map }, current_config_path: None }
}

#[test]
fn test_successful_local_template_copy() {
    let temp = tempdir().unwrap();
    let template = temp.path().join("template");
    fs::create_dir_all(&template).unwrap();
    fs::write(template.join("file.txt"), "hello").unwrap();

    let args = NewCommand {
        name: "my_app".into(),
        kind: None,
        template: Some(template.to_string_lossy().into_owned()),
        repo: None,
        config: None,
    };

    let mut config = Config::default();
    std::env::set_current_dir(temp.path()).unwrap();
    let result = new_command_action(&mut config, &args);
    assert!(result.is_ok());
    assert!(temp.path().join("my_app/file.txt").exists());
}

#[test]
fn test_nonexistent_template_path() {
    let temp = tempdir().unwrap();
    let fake_path = temp.path().join("not_exist_template");

    let args = NewCommand {
        name: "fail_app".into(),
        kind: None,
        template: Some(fake_path.to_string_lossy().into_owned()),
        repo: None,
        config: None,
    };

    let mut config = Config::default();
    std::env::set_current_dir(temp.path()).unwrap();
    let result = new_command_action(&mut config, &args);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("Template path does not exist"));
}

#[test]
fn test_direct_repo_todo_path() {
    let temp = tempdir().unwrap();
    let args = NewCommand {
        name: "repo_app".into(),
        kind: None,
        template: None,
        repo: Some("https://github.com/some/repo.git".to_string()),
        config: None,
    };

    let mut config = Config::default();
    std::env::set_current_dir(temp.path()).unwrap();
    let result = new_command_action(&mut config, &args);
    assert!(result.is_err());
    let err_string = format!("{:?}", result.unwrap_err());
    assert!(err_string.contains("not implemented") || err_string.contains("todo"));
}

#[test]
fn test_template_from_config_selection() {
    let temp = tempdir().unwrap();
    let template = temp.path().join("tpl_dir");
    fs::create_dir_all(&template).unwrap();
    fs::write(template.join("in.txt"), "world").unwrap();

    let config_path = temp.path().join("metadata.toml");

    let config = dummy_config_with_template(template.clone());
    fs::write(&config_path, toml::to_string(&config.metadata).unwrap())
        .expect("Failed to write config file");

    assert!(config_path.exists(), "Config file does not exist");

    let args = NewCommand {
        name: "via_config".into(),
        kind: Some(TemplateKind::Project),
        template: None,
        repo: None,
        config: None,
    };

    std::env::set_current_dir(temp.path()).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("shared-kit").unwrap();

    cmd.arg("new")
        .arg(&args.name)
        .arg("--config")
        .arg(config_path)
        .write_stdin("test-template")
        .assert()
        .success()
        .stdout(predicate::str::contains("via_config"))
        .stdout(predicate::str::contains("Template copied"))
        .stderr(predicate::str::is_empty()); // CLI 没有异常输出

    let output_dir = temp.path().join("via_config");
    let output_file = output_dir.join("in.txt");

    // 检查生成的目录和文件是否存在
    assert!(output_dir.exists(), "Output directory does not exist");
    assert!(output_file.exists(), "Output file does not exist");

    // 验证生成的文件内容是否正确
    let content = fs::read_to_string(output_file).expect("Failed to read output file");
    assert_eq!(content, "world", "Output file content does not match expected");

    // 验证模板目录未被修改
    let original_content =
        fs::read_to_string(template.join("in.txt")).expect("Failed to read template file");
    assert_eq!(original_content, "world", "Template file content was unexpectedly modified");
}

#[test]
fn test_config_empty_should_fail() {
    let temp = tempdir().unwrap();

    let mut config = Config::default();
    let args = NewCommand {
        name: "empty".into(),
        kind: Some(TemplateKind::Project),
        template: None,
        repo: None,
        config: None,
    };

    std::env::set_current_dir(temp.path()).unwrap();
    let result = new_command_action(&mut config, &args);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("No templates found"));
}
