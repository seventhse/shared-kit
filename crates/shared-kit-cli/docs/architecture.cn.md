# Shared Kit CLI 架构文档

一个通用的工具

功能特性：
* 管理项目模板并创建新项目。
* 监听特定文件或文件夹并执行 Shell 命令、系统命令或内置函数。
* 并行执行开发脚本。

---

[toc]

## 关于

### ⚙️ 配置文件

> **默认配置路径：**  
> `$HOME/.config/shared-kit-cli/.config.{toml|json}`

---

### 📝 TOML 配置格式

```toml
[templates.package-example]
# 可选：指定模板类型
# 常用值：project | package | monorepo
kind = "package"

# template 和 repo 至少提供一个
# 本地模板路径（相对或绝对路径）
template = "./basic-package"

# 可选：远程仓库地址（GitHub 或 GitLab）
repo = "https://github.com/octocat/Hello-World"

# 可选：指定要包含的文件或目录
# 支持通过前缀 regex: 启用正则匹配
includes = [
  "/src",
  "package.json",
  "Cargo.toml",
  "regex:^README(\\.md)?$"
]

# 可选：指定要排除的文件或目录
# 支持通过前缀 regex: 启用正则匹配
excludes = [
  "/node_modules",
  "/target",
  "regex:^\\..*\\.swp$"
]

# 定义变量替换规则
[[templates.package-example.template_vars]]
# 必填：模板中的占位符（如 {{project_name}}）
placeholder = "{{project_name}}"

# 可选：提示用户输入变量值时显示的内容
prompt = "请输入项目名称"

# 可选：默认值（用户未输入时使用）
default = "new_project"

# 可选：模板生成完成后自动执行的命令列表。
# 每项为一条 Shell 命令或关键字，例如 "CD_TARGET" 表示切换到生成后的项目目录。
# 常用于后处理操作，如安装依赖、初始化 Git 仓库等。
# 示例：["CD_TARGET", "pnpm i"] 表示进入目标目录并执行 pnpm 安装。
completed_script = ["CD_TARGET","pnpm i"]

# 可选：仅替换指定路径下的文件
# 支持 regex: 前缀开启正则匹配
includes_paths = [
  "package.json",
  "index.html",
  "regex:^.*\\.rs$"
]

# 可选：排除不替换的路径
# 支持 regex: 前缀开启正则匹配
excludes_paths = [
  "/node_modules",
  "regex:^\\.git/"
]
```

---

### 📄 JSON 配置格式

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
          "prompt": "请输入项目名称",
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

### 📌 说明

- `includes` 和 `excludes` 支持文件或目录路径。
- 如需使用正则匹配，请以 `regex:` 前缀标识。
- 路径支持相对或绝对形式，相对路径相对于模板目录解析。
- `includes_paths` 和 `excludes_paths` 可使用 glob 模式（如 `"**/*.ts"`）。
- `template_vars` 可在生成过程中进行占位符替换。

---

### 🧩 支持的仓库地址格式

> 当前支持平台：`GitHub` 和 `GitLab`

---

#### ✅ 支持的地址语法

- **完整 URL 格式**  
  示例：  
  - `https://github.com/<用户名>/<仓库名>`

- **简写格式（推荐）**  
  示例：  
  - `<用户名>/<仓库名>`

使用示例：  
- `https://github.com/octocat/Hello-World`  
- `octocat/Hello-World`

---

#### 🏷️ 支持的版本标记

你可以在仓库地址后追加以下内容以指定版本：

- `#<分支名>` —— 指定分支（默认是 `main`）  
- `@<tag>` —— 指定发布标签  
- `@<commit>` —— 指定提交哈希值

示例：  
- `octocat/Hello-World#master` —— 使用 `master` 分支  
- `octocat/Hello-World@v1.0.0` —— 使用标签 `v1.0.0`  
- `octocat/Hello-World@abcdef1234567890` —— 使用指定提交

---

> ⚠️ 注意：请使用 `#` 或 `@` 来指定版本，避免混用。推荐使用标签或提交哈希以确保稳定性。

---

## 使用命令

### 命令列表

### `new` 命令

#### 1. 命令输入格式
```shell
shared-kit new <name> --template <模板路径> --repo <仓库地址> --kind <project | package | monorepo> --config <配置路径>
```

---

#### 2. 加载配置

- 加载默认配置路径。
- 检查是否提供 `--config` 参数：
  - **是**：加载用户提供的配置路径。
  - **否**：继续下一步。

---

#### 3. 执行流程

##### 3.1 检查模板路径（用户输入）
- **存在**：直接进入 “创建流程”。
- **不存在**：继续下一步。

##### 3.2 检查仓库路径（用户输入）
- **存在**：拉取仓库并进入 “创建流程”。
- **不存在**：检查是否已选中模板：
  - **是**：退出流程。
  - **否**：继续下一步。

##### 3.3 检查模板类型（--kind）
- **存在**：根据类型过滤可用配置模板。
- **不存在**：继续下一步。

##### 3.4 提供交互选择界面
- **用户选择模板**：回到 “执行流程” 重新检查。
- **用户未选择**：退出流程。

---

#### 4. 创建流程（Create）

##### 4.1 检查模板过滤规则
- **存在过滤器**：
  - 应用 `includes` / `excludes` 逻辑。
- **无过滤器**：继续下一步。

##### 4.2 检查模板变量
- **存在变量**：
  - 替换模板文件中的占位符。
- **无变量**：继续下一步。

##### 4.3 执行目录拷贝
- **成功**：
  - 检查是否定义成功后脚本：
    - **是**：执行脚本。
    - **否**：提示成功消息。
- **失败**：
  - 输出错误原因并退出。

---

### `watch` 命令
（待补充）

---

### `exec` 命令
（待补充）