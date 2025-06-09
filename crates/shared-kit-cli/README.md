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

By default, this creates a new **package** using the built-in template.

#### Custom Template Usage

```bash
shared-kit new <name> --template <template-path>
```

Use a custom template from a local directory or remote source.

#### Supported Template Types

The `new` command supports scaffolding for the following project types:

- **monorepo** – Create a new Rust or Node monorepo with common shared configurations.  
- **project** – Initialize a standalone Node/Rust/other project with standard setup.  
- **package** – Generate a reusable shared library or module inside an existing monorepo.

Use the `--type` option to specify the type (default is `package`):

```bash
shared-kit new my-utils --type package
shared-kit new frontend-core --type project
shared-kit new my-monorepo --type monorepo
```

#### Example

```bash
# Using default package template
shared-kit new my-lib

# Using a custom project template
shared-kit new my-app --template ./templates/react-app

# Creating a new monorepo
shared-kit new my-kit --type monorepo
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
cargo run -- new my-project --type project
```

---

## Contribution

Contributions are welcome! Please follow the guidelines in `CONTRIBUTING.md` to submit issues or pull requests.

---

## License

This project is licensed under the MIT License. See `LICENSE` for details.