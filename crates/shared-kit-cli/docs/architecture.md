# Shared Kit CLI Architecture

A common tool

Features:
* Manage project templates and create new projects.
* Watch specific files or folders to execute shell commands, system commands, or built-in functions.
* Parallel execution of development scripts.

---

[toc]

## About

### ⚙️ Configuration File

> **Default Config Path:**  
> `$HOME/.config/shared-kit-cli/.config.{toml|json}`

---

### 📝 TOML Configuration Format

```toml
[templates.package-example]
# Specify the template type.
# Common values: project | package | monorepo
kind = "package"

# One of either `template` or `repo` must be provided.
# Local template directory (relative or absolute)
template = "./basic-package"

# Optional: Remote repository address (GitHub or GitLab)
repo = "https://github.com/octocat/Hello-World"

# Optional: Files or directories to include in the final output
# Supports regex by prefixing with `regex:`
includes = [
  "/src",
  "package.json",
  "Cargo.toml",
  "regex:^README(\\.md)?$"
]

# Optional: Files or directories to exclude from the output
# Supports regex by prefixing with `regex:`
excludes = [
  "/node_modules",
  "/target",
  "regex:^\\..*\\.swp$"
]

# Define variable substitutions for this template
[[templates.package-example.template_vars]]
# Required: The placeholder used in the template (e.g., {{project_name}})
placeholder = "{{project_name}}"

# Optional: A message shown to the user when prompting for input
prompt = "Please input your new project name"

# Optional: A default value to use if no input is provided
default = "new_project"

# Optional: Commands to run after the template has been fully generated and variables substituted.
# Each entry is a shell command or keyword. Can be used for post-processing steps like dependency installation,
# setting permissions, or initializing git.
# Example: ["CD_TARGET", "pnpm i"] means switch to the generated project directory and run `pnpm install`.
completed_script = ["CD_TARGET","pnpm i"]

# Optional: Limit replacement to specific files only
# Supports regex by prefixing with `regex:`
includes_paths = [
  "package.json",
  "index.html",
  "regex:^.*\\.rs$"
]

# Optional: Skip replacement in these files
# Supports regex by prefixing with `regex:`
excludes_paths = [
  "/node_modules",
  "regex:^\\.git/"
]
```

---

### 📄 JSON Configuration Format

```json5
{
  "templates": {
    "package-example": {
      "kind": "package",
      "template": "./basic-package",
      "repo": "https://github.com/octocat/Hello-World",
      "includes": [
        "/src",
        "package.json",
        "Cargo.toml",
        "regex:^README(\\.md)?$"
      ],
      "excludes": [
        "/node_modules",
        "/target",
        "regex:^\\..*\\.swp$"
      ],
      "template_vars": [
        {
          "placeholder": "{{project_name}}",
          "prompt": "Please input your new project name",
          "default": "new_project",
          "completed_script": ["CD_TARGET","pnpm i"],
          "includes_paths": [
            "package.json",
            "index.html",
            "regex:^.*\\.rs$"
          ],
          "excludes_paths": [
            "/node_modules",
            "regex:^\\.git/"
          ]
        }
      ]
    }
  }
}
```

---

### 📌 Notes

- `includes` and `excludes` support both file and directory paths.
- To use regex in paths, prefix with `regex:`.
- Paths can be relative or absolute, relative paths are resolved from the template directory.
- Glob patterns (e.g., `"**/*.ts"`) are supported in `includes_paths` and `excludes_paths`.
- `template_vars` allow dynamic placeholder replacement during generation.

#### 🧩 Supported Repository Address Formats

> Currently supported platforms: `GitHub` and `GitLab`

---

##### ✅ Supported Address Syntax

- **Full URL Format**  
  Example:  
  - `https://github.com/<username>/<reponame>`

- **Short Format (recommended)**  
  Example:  
  - `<username>/<reponame>`

Usage examples:  
- `https://github.com/octocat/Hello-World`  
- `octocat/Hello-World`

---

##### 🏷️ Supported Version Specifiers

You can append the following specifiers to target a specific version:

- `#<branch>` – Specify a branch (defaults to `main` if omitted)  
- `@<tag>` – Specify a release tag  
- `@<commit>` – Specify an exact commit hash

Examples:  
- `octocat/Hello-World#master` – Use the `master` branch  
- `octocat/Hello-World@v1.0.0` – Use tag `v1.0.0`  
- `octocat/Hello-World@abcdef1234567890` – Use commit `abcdef1234567890`

---

> ⚠️ Note: You may use either `#` or `@` to specify versions. Avoid combining them in a single reference. For stable results, prefer tags or commit hashes when possible.

### Usage command

## Commands

### `new` Command

#### 1. Input Command
```shell
shared-kit new <name> --template <template_path> --repo <repo_address > --kind <project | package | monorepo> --config <config_path>
```
---

#### 2. Load Configuration
- Load the default configuration.
- Check if the user provided the `--config` parameter:
  - **Yes**: Load the configuration from the provided path.
  - **No**: Proceed to the next step.

---

#### 3. Execution Flow

##### 3.1 Check Template Path (User-Provided)
- **Exists**: Proceed directly to the "Create" flow.
- **Does Not Exist**: Check the next item.

##### 3.2 Check Repository Path (User-Provided)
- **Exists**: Download the repository and proceed to the "Create" flow.
- **Does Not Exist**: Check if a template has already been selected:
  - **Yes**: Exit.
  - **No**: Proceed to the next step.

##### 3.3 Check Template Type (`--kind` Parameter)
- **Exists**: Filter matching configuration templates and proceed.
- **Does Not Exist**: Proceed to the next step.

##### 3.4 Provide Interactive Selection Interface
- **User Selected a Template**: Return to the "Check Flow" and re-evaluate.
- **User Did Not Select a Template**: Exit the flow.

---

#### 4. Template Creation Flow (Create)

##### 4.1 Check Template Filters (Filter)
- **Filters Exist**:
  - Apply `include` / `exclude` filtering logic.
- **No Filters**: Proceed to the next step.

##### 4.2 Check Template Variables (Vars)
- **Variables Exist**:
  - Replace variable placeholders in the target files.
- **No Variables**: Proceed to the next step.

##### 4.3 Execute Directory Copy Operation
- **Success**:
  - Check if there are post-success scripts:
    - **Yes**: Execute the script.
    - **No**: Display a success message.
- **Failure**:
  - Display the error reason and exit.

---

### `watch` Command
(To be documented)

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

### `exec` Command
(To be documented)

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