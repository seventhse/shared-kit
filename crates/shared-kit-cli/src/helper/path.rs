use dirs::home_dir;
use path_clean::PathClean;
use std::path::{Path, PathBuf};

pub fn expand_dir(path: &str) -> Option<PathBuf> {
    if let Some(stripped) = path.strip_prefix("~/") {
        home_dir().map(|home| home.join(stripped))
    } else {
        Some(PathBuf::from(path))
    }
}

pub fn join_with_config_dir(config_path: Option<&PathBuf>, relative: &Path) -> PathBuf {
    let path = match config_path {
        Some(base_path) => {
            let dir = base_path.parent().unwrap_or_else(|| Path::new("."));
            dir.join(relative)
        }
        None => PathBuf::from(relative),
    };

    path.clean()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_dir_with_tilde() {
        let home = dirs::home_dir().expect("Home dir should exist");
        let input = "~/test/path";
        let result = expand_dir(input).expect("expand_dir returned None");

        assert_eq!(result, home.join("test/path"));
    }

    #[test]
    fn test_expand_dir_without_tilde() {
        let input = "/usr/bin";
        let result = expand_dir(input).expect("expand_dir returned None");

        assert_eq!(result, PathBuf::from("/usr/bin"));
    }

    #[test]
    fn test_join_with_config_dir_with_parent_relative_path() {
        let config_path = PathBuf::from("/home/user/.config/shared-kit-cli/metadata.toml");
        let relative = Path::new("../templates/default");

        let result = join_with_config_dir(Some(&config_path), relative);
        let expected = PathBuf::from("/home/user/.config/templates/default");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_join_with_config_dir_with_dot_relative_path() {
        let config_path = PathBuf::from("/home/user/.config/shared-kit-cli/metadata.toml");
        let relative = Path::new("./templates/abc");

        let result = join_with_config_dir(Some(&config_path), relative);
        let expected = PathBuf::from("/home/user/.config/shared-kit-cli/templates/abc");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_join_with_config_dir_with_direct_file_name() {
        let config_path = PathBuf::from("/home/user/.config/shared-kit-cli/metadata.toml");
        let relative = Path::new("file.txt");

        let result = join_with_config_dir(Some(&config_path), relative);
        let expected = PathBuf::from("/home/user/.config/shared-kit-cli/file.txt");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_join_with_config_dir_none_config_path() {
        let relative = Path::new("something/else");

        let result = join_with_config_dir(None, relative);
        let expected = PathBuf::from("something/else").clean();

        assert_eq!(result, expected);
    }
}
