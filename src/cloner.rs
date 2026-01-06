use anyhow::{Context, Result};
use log::info;
use std::process::Command;
use tempfile::TempDir;

pub fn clone_repo(url: &str, branch: Option<&str>) -> Result<TempDir> {
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
