use anyhow::{Context, Result};
use log::info;
use std::process::Command;
use tempfile::TempDir;

use std::sync::OnceLock;

static GIT_CHECK: OnceLock<Result<(), String>> = OnceLock::new();

fn check_git_installed() -> Result<()> {
    GIT_CHECK
        .get_or_init(|| match Command::new("git").arg("--version").output() {
            Ok(output) if output.status.success() => Ok(()),
            _ => Err(
                "Git is not installed or not in PATH. Please install Git to clone repositories."
                    .to_string(),
            ),
        })
        .clone()
        .map_err(|e| anyhow::anyhow!(e))
}

pub fn clone_repo(url: &str, branch: Option<&str>) -> Result<TempDir> {
    check_git_installed()?;
    let temp_dir = TempDir::new()?;
    let target_path = temp_dir.path();

    info!("Cloning {url} into {}", target_path.display());

    let mut cmd = Command::new("git");
    cmd.arg("clone").arg("--depth").arg("1");

    if let Some(b) = branch {
        cmd.arg("--branch").arg(b);
    }

    cmd.arg(url).arg(target_path);

    let status = cmd.status().context("Failed to execute git clone")?;

    if !status.success() {
        anyhow::bail!("git clone failed with exit code: {:?}", status.code());
    }

    Ok(temp_dir)
}
