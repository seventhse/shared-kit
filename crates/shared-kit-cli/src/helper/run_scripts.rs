use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::Context;
use shared_kit_common::output;

pub fn run_completed_scripts(scripts: &[String], target_path: &Path) -> anyhow::Result<()> {
    let current_dir = target_path.to_path_buf();

    for script in scripts {
        output!(line: "──────────────────────────────────────────────");
        output!(space);
        output!(line: "🛠️  Running command:   {}", script);
        output!(space);
        output!(line: "📂 Working directory:  {}", current_dir.display());
        output!(space);
        output!(line: "──────────────────────────────────────────────");
        output!(space);

        #[cfg(target_os = "windows")]
        let mut cmd = {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(script);
            c
        };
        #[cfg(not(target_os = "windows"))]
        let mut cmd = {
            let mut c = Command::new("sh");
            c.arg("-c").arg(script);
            c
        };

        cmd.current_dir(&current_dir).stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd.spawn().with_context(|| format!("Failed to run script: {}", script))?;

        // 实时读取 stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    output!(line: "{}", line);
                }
            }
        }

        // 实时读取 stderr
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    output!(line: "{}", line);
                }
            }
        }

        let status = child.wait()?;
        if !status.success() {
            output!(line: "❌ Script failed: `{}` (exit code: {:?})", script, status.code());
            anyhow::bail!("Script failed: {}\nExit code: {:?}", script, status.code());
        } else {
            output!(space);
            output!(line: "✅ Done: {}", script);
        }
    }

    output!(space);
    output!(title: "🎉 All post-generation scripts executed.");
    output!(space);
    output!(line: "📂 Final working directory: {}", current_dir.display());
    output!(space);
    output!(line: "💡 Tip: You can now:");
    output!(line: "   cd {}", current_dir.display());

    Ok(())
}
