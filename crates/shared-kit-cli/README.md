# Shared Kit CLI

A modular command-line toolkit designed to simplify project scaffolding, configuration sharing, and developer tool automation across multiple languages and environments.

---

## Installation

To install `shared-kit`, use the following command:

```bash
cargo install shared-kit
```

---

## Usage

### `new` Command

Scaffold a new project, monorepo, or package using built-in or user-defined templates.

#### Basic Usage

```bash
shared-kit new <name>
```

This creates a new project with the specified name using the interactive template selection process.

#### Command Options

```bash
shared-kit new <name> [OPTIONS]
```

| Option                      | Description                                           |
| --------------------------- | ----------------------------------------------------- |
| `-k, --kind <KIND>`         | Filter templates by kind (project, monorepo, package) |
| `-p, --template <TEMPLATE>` | Use a specific local template path                    |
| `-r, --repo <REPO>`         | Use a remote repository as template source            |
| `-c, --config <CONFIG>`     | Use a custom configuration file                       |

#### Template Sources

The `new` command can create projects from three different sources:

1. **Built-in Templates** - Default templates included in your configuration
2. **Local Templates** - Custom templates from your local filesystem
3. **Remote Repositories** - Templates from GitHub, GitLab, or other git hosts

#### Using Local Templates

```bash
shared-kit new my-app --template ./templates/react-app
```

This bypasses the template selection process and directly uses the specified local template.

#### Using Remote Repositories

```bash
shared-kit new my-app --repo username/repo-name
# or with a specific branch/tag
shared-kit new my-app --repo username/repo-name#branch-name
shared-kit new my-app --repo username/repo-name@v1.0.0
# or with full URL
shared-kit new my-app --repo https://github.com/username/repo-name
```

Supports GitHub, GitLab, and Gitea repositories with branch, tag, and commit specifications.

#### Template Types

The `new` command supports three project types:

- **project** – Initialize a standalone project with standard setup
- **monorepo** – Create a new monorepo with common shared configurations  
- **package** – Generate a reusable library or module

Filter available templates by type with the `--kind` option:

```bash
shared-kit new my-utils --kind package
shared-kit new frontend-core --kind project
shared-kit new my-monorepo --kind monorepo
```

#### Template Variables

Templates can include variable placeholders (e.g., `{{project_name}}`) that will be interactively replaced during project generation. You'll be prompted to provide values for each variable, with default values shown when available.

#### Target Directory Handling

If a target directory already exists, you'll be prompted with options to:
- Rename the project
- Overwrite the existing directory
- Cancel the operation

#### Post-Generation Scripts

Templates can define `completed_script` actions that run automatically after generation, such as:
- Installing dependencies
- Initializing git repositories
- Setting up configuration

#### Examples

```bash
# Interactive template selection (default)
shared-kit new my-app

# Using a specific template kind
shared-kit new my-lib --kind package

# Using a local template
shared-kit new my-app --template ./templates/react-app

# Using a remote repository template
shared-kit new my-app --repo octocat/Hello-World#master

# Using a custom configuration file
shared-kit new my-app --config ./my-templates.json
```

---

### `watch` Command

Monitor specified directories or files for changes and trigger actions automatically.

#### Basic Usage

```bash
shared-kit watch <path_name | directory_path>
```

#### Advanced Usage

```bash
shared-kit watch --path ./src --cmd "echo changed!"
shared-kit watch --path ./config.toml --rust-fn reload_config
shared-kit watch --config watch_rules.json
```

---

### `run` Command

Parallel monitoring of multiple directories or files, re-executing commands on changes.

#### Basic Usage

```bash
shared-kit run --watch ./src --cmd "cargo build" --watch ./docs --cmd "make html"
```

#### Advanced Usage

```bash
shared-kit run --watch ./a --watch ./b --rust-fn reload_all
shared-kit run --config run_rules.json
```

---

### Unified Configuration File

To simplify configuration management, `shared-kit` supports a unified configuration file named `shared-kit.json`. This file can define settings for multiple commands, such as `watch` and `run`, in a single place.

#### Example Configuration (`shared-kit.json`)

```json
{
  "watch": [
    {
      "path": "./src",
      "cmd": "echo changed!"
    },
    {
      "path": "./config.toml",
      "rust_fn": "reload_config"
    }
  ],
  "run": [
    {
      "watch": "./src",
      "cmd": "cargo build"
    },
    {
      "watch": "./docs",
      "cmd": "make html"
    }
  ]
}
```

#### Usage with Unified Configuration

```bash
# Watch using unified configuration
shared-kit watch --config shared-kit.json

# Run using unified configuration
shared-kit run --config shared-kit.json
```

---

## Development

### Prerequisites

Ensure you have the following installed:
- **Rust**: Install via [rustup](https://rustup.rs/).
- **Cargo**: Comes with Rust installation.

### Building the Project

Clone the repository and build the project:

```bash
git clone https://github.com/seventhse/shared-kit.git
cd crates/shared-kit
cargo build --release
```

The compiled binary will be available in the `target/release` directory.

### Running Tests

Run the test suite to ensure everything is working correctly:

```bash
cargo test
```

### Adding a New Command

1. Create a new file in the `src/subcommand/` directory (e.g., `my_command.rs`).
2. Implement the command logic.
3. Register the command in `src/cli.rs`.

### Debugging

Use `cargo run` to debug the CLI:

```bash
cargo run -- <command> [options]
```

For example:

```bash
cargo run -- new my-project --kind project
```

---

## Contribution

Contributions are welcome! Please follow the guidelines in `CONTRIBUTING.md` to submit issues or pull requests.

---

## License

This project is licensed under the MIT License. See `LICENSE` for details.