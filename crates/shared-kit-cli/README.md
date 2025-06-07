# Shared kit cli

A modular command-line toolkit designed to simplify project scaffolding, configuration sharing, and developer tool automation across multiple languages and environments.


## Features

- 🧱 Project scaffolding (apps, packages, monorepos, etc.)
- 🧰 Config management for TypeScript, ESLint, Rust, etc.
- ⚙️ Developer utilities: formatters, linters, watchers, task runners
- 🎯 Support for custom templates and project types

## SubCommand

### `new` Command

Scaffold a new project, monorepo, or package using built-in or user-defined templates.

#### 📦 Basic Usage

```bash
shared-kit-cli new <name>
```

By default, this creates a new **package** using the built-in template.

#### ⚙️ Custom Template Usage

```bash
shared-kit-cli new <name> --template <template-path>
```

Use a custom template from a local directory or remote source.

---

#### 🧱 Supported Template Types

The `new` command supports scaffolding for the following project types:

- **monorepo** – Create a new Rust or Node monorepo with common shared configurations  
- **project** – Initialize a standalone Node/Rust/other project with standard setup  
- **package** – Generate a reusable shared library or module inside an existing monorepo

Use the `--type` option to specify the type (default is `package`):

```bash
shared-kit-cli new my-utils --type package
shared-kit-cli new frontend-core --type project
shared-kit-cli new my-monorepo --type monorepo
```

---

#### 🧩 Custom Templates

You can provide your own project templates via the `--template` flag. A custom template should be a folder with the following structure:

```
my-template/
├── template.json   # metadata and placeholders
├── package.json    # optional (for Node templates)
├── src/            # actual content to copy
└── ...
```

Custom templates can be:
- **Local directories**
- **Remote git repositories** *(planned)*

---

#### ✅ Example

```bash
# Using default package template
shared-kit-cli new my-lib

# Using a custom project template
shared-kit-cli new my-app --template ./templates/react-app

# Creating a new monorepo
shared-kit-cli new my-kit --type monorepo
```

---
### `Watch` command

* basic usage:   
  `shared-kit-cli watch <path_name | directory_path>`


### `Help` command
- **Purpose**: Display detailed information about available commands and their usage.
- **Key Features**:
  - Lists all supported commands (e.g., `new`, `watch`, `run`).
  - Provides examples for each command.
  - Explains available options and flags.
- **Example Usage**:
  ```sh
  shared-kit-cli help
  shared-kit-cli help new
  shared-kit-cli help watch
  ```

### `Version` command
- **Purpose**: Display the current version of `shared-kit-cli`.
- **Key Features**:
  - Outputs the version number.
  - Useful for debugging and ensuring compatibility.
- **Example Usage**:
  ```sh
  shared-kit-cli version
  ```

---

## Advanced Features: Watch & Run

### `watch` Command
- **Purpose**: Monitor specified directories or files for changes and trigger actions automatically.
- **Key Features**:
  - Reload and execute:
    - Built-in Rust functions (e.g., template rendering, config refresh)
    - External shell scripts (sh, cmd, bat, etc.)
  - Configurable command arguments and environment variables
  - Recursive subdirectory watching supported
  - Ignore patterns for files/directories (e.g., .git, node_modules)
  - Cross-platform (Linux/macOS/Windows)
  - **Batch configuration via `.json` file** (e.g., `watch_rules.json`)
- **Example Usage**:
  ```sh
  shared-kit-cli watch --path ./src --cmd "echo changed!"
  shared-kit-cli watch --path ./config.toml --rust-fn reload_config
  shared-kit-cli watch --config watch_rules.json
  ```

### `run` Command
- **Purpose**: Parallel monitoring of multiple directories or files, re-executing commands on changes.
- **Key Features**:
  - Re-execute:
    - Built-in Rust functions
    - External shell/cmd commands
  - Per-watch or unified command configuration
  - Debounce/throttle support to avoid frequent triggers
  - Pre/post execution hooks
  - **Batch configuration via `.json` file** (e.g., `run_rules.json`)
- **Example Usage**:
  ```sh
  shared-kit-cli run --watch ./src --cmd "cargo build" --watch ./docs --cmd "make html"
  shared-kit-cli run --watch ./a --watch ./b --rust-fn reload_all
  shared-kit-cli run --config run_rules.json
  ```

---

## Unified Configuration File

To simplify configuration management, `shared-kit-cli` supports a unified configuration file named `shared-kit.json`. This file can define settings for multiple commands, such as `watch` and `run`, in a single place.

### Example Configuration (`shared-kit.json`)
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

### Benefits of Unified Configuration
- **Centralized Management**: All command configurations are stored in a single file.
- **Simplified CLI Usage**: Use the `--config` flag to specify the configuration file for any command.
- **Extensibility**: Easily add new commands or features by extending the JSON structure.

### Usage with Unified Configuration
```bash
# Watch using unified configuration
shared-kit-cli watch --config shared-kit.json

# Run using unified configuration
shared-kit-cli run --config shared-kit.json
```

---