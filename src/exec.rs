use std::process::Command;

use anyhow::{Context, Result};

use crate::expand;

/// `ge` の本体: 番号を展開して任意のコマンドを実行する（例: ge vim 1）
pub fn run(args: &[String]) -> Result<i32> {
    let args = expand::expand_with_cache(args)?;
    let (program, rest) = args
        .split_first()
        .context("実行するコマンドを指定してください")?;
    run_command(program, rest)
}

/// コマンドを実行し、終了コードを返す
pub fn run_command(program: &str, args: &[String]) -> Result<i32> {
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("{program} を実行できませんでした"))?;
    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_exit_code_of_command() {
        let args = ["-c".to_string(), "exit 3".to_string()];
        assert_eq!(run_command("sh", &args).unwrap(), 3);
    }

    #[test]
    fn missing_command_is_an_error() {
        assert!(run_command("gsct-no-such-command", &[]).is_err());
    }
}
