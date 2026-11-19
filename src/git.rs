use std::process::Command;

use anyhow::{Context, Result, bail};

/// Runs git and returns its standard output as a string
pub fn output(args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .context("failed to run git")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }

    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
