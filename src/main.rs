use std::path::PathBuf;
use std::process;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod cache;
mod exec;
mod expand;
mod git;
mod status;

/// scm_breeze 風の git status shortcuts と git alias
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 番号付きで git status を表示する（gs）
    Status,
    /// 番号をファイルパスに展開して任意のコマンドを実行する（ge）
    Exec {
        /// 実行するコマンドと引数
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// alias の一覧を表示する
    Aliases,
    /// alias 名のシンボリックリンクを作成する
    InstallLinks {
        /// リンクを作成するディレクトリ（例: ~/.local/bin）
        dir: PathBuf,
        /// 既存のコマンドと名前が衝突しても作成する
        #[arg(long)]
        force: bool,
        /// 作成しない alias 名（カンマ区切り。例: --skip gs,gm）
        #[arg(long, value_delimiter = ',')]
        skip: Vec<String>,
    },
    /// alias を実行する（例: gsct gd 1）
    #[command(external_subcommand)]
    Alias(Vec<String>),
}

fn main() {
    match run() {
        Ok(code) => process::exit(code),
        Err(err) => {
            eprintln!("gsct: {err:#}");
            process::exit(1);
        }
    }
}

fn run() -> Result<i32> {
    let cli = Cli::parse();
    let code = match cli.command {
        Command::Status => {
            status::run()?;
            0
        }
        Command::Exec { args } => exec::run(&args)?,
        Command::Aliases => {
            println!("TODO: aliases");
            0
        }
        Command::InstallLinks { dir, force, skip } => {
            println!("TODO: install-links {dir:?} force={force} skip={skip:?}");
            0
        }
        Command::Alias(args) => {
            println!("TODO: alias {args:?}");
            0
        }
    };
    Ok(code)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::{CommandFactory, Parser};

    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn unknown_subcommand_is_treated_as_alias() {
        let cli = Cli::try_parse_from(["gsct", "gd", "1", "--cached"]).unwrap();
        let Command::Alias(args) = cli.command else {
            panic!("alias として解釈されていません");
        };
        assert_eq!(args, ["gd", "1", "--cached"]);
    }

    #[test]
    fn exec_keeps_hyphen_arguments() {
        let cli = Cli::try_parse_from(["gsct", "exec", "ls", "-l", "1"]).unwrap();
        let Command::Exec { args } = cli.command else {
            panic!("exec として解釈されていません");
        };
        assert_eq!(args, ["ls", "-l", "1"]);
    }

    #[test]
    fn install_links_accepts_comma_separated_skip() {
        let cli =
            Cli::try_parse_from(["gsct", "install-links", "/tmp/bin", "--skip", "gs,gm"]).unwrap();
        let Command::InstallLinks { dir, force, skip } = cli.command else {
            panic!("install-links として解釈されていません");
        };
        assert_eq!(dir, PathBuf::from("/tmp/bin"));
        assert!(!force);
        assert_eq!(skip, ["gs", "gm"]);
    }

    #[test]
    fn exec_requires_arguments() {
        assert!(Cli::try_parse_from(["gsct", "exec"]).is_err());
    }
}
