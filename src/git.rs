use std::process::Command;

use anyhow::{Context, Result, bail};

/// git コマンドを実行し、標準出力を文字列で返す
pub fn output(args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .context("git を実行できませんでした")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!("git {} が失敗しました: {}", args.join(" "), stderr.trim());
    }

    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
