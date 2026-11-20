use std::env;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod aliases;
mod cache;
mod complete;
mod exec;
mod expand;
mod git;
mod links;
mod status;

/// Git status shortcuts and git aliases, inspired by scm_breeze
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show git status with numbered files (gs)
    Status,
    /// Run any command with numbers expanded to file paths (ge)
    Exec {
        /// Command to run and its arguments
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// List the aliases
    Aliases,
    /// Create symlinks named after the aliases
    InstallLinks {
        /// Directory to create the links in (e.g. ~/.local/bin)
        dir: PathBuf,
        /// Create links even if they collide with existing commands
        #[arg(long)]
        force: bool,
        /// Alias names to skip (comma-separated, e.g. --skip gs,gm)
        #[arg(long, value_delimiter = ',')]
        skip: Vec<String>,
    },
    /// Print a shell completion script (e.g. eval "$(gsct completion bash)")
    Completion {
        /// Shell to generate the script for
        #[arg(value_parser = ["bash", "zsh"])]
        shell: String,
    },
    /// Print completion candidates (used by the completion script)
    #[command(name = "__complete", hide = true)]
    Complete {
        /// Alias name followed by the words typed so far
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run an alias (e.g. gsct gd 1)
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
    let cli = Cli::parse_from(args_with_alias_name(env::args().collect()));
    let code = match cli.command {
        Command::Status => {
            status::run()?;
            0
        }
        Command::Exec { args } => exec::run(&args)?,
        Command::Aliases => {
            aliases::print_list();
            0
        }
        Command::InstallLinks { dir, force, skip } => {
            links::install(&dir, force, &skip)?;
            0
        }
        Command::Completion { shell } => {
            complete::print_script(&shell)?;
            0
        }
        Command::Complete { args } => {
            complete::run(&args)?;
            0
        }
        Command::Alias(args) => aliases::run(&args)?,
    };
    Ok(code)
}

/// When called through a symlink such as `gd 1`, treat it as `gsct gd 1`
fn args_with_alias_name(mut args: Vec<String>) -> Vec<String> {
    let program = args
        .first()
        .and_then(|arg| Path::new(arg).file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("gsct")
        .to_string();

    if program != "gsct" {
        args[0] = program;
        args.insert(0, "gsct".to_string());
    }

    args
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
            panic!("not parsed as an alias");
        };
        assert_eq!(args, ["gd", "1", "--cached"]);
    }

    #[test]
    fn exec_keeps_hyphen_arguments() {
        let cli = Cli::try_parse_from(["gsct", "exec", "ls", "-l", "1"]).unwrap();
        let Command::Exec { args } = cli.command else {
            panic!("not parsed as exec");
        };
        assert_eq!(args, ["ls", "-l", "1"]);
    }

    #[test]
    fn install_links_accepts_comma_separated_skip() {
        let cli =
            Cli::try_parse_from(["gsct", "install-links", "/tmp/bin", "--skip", "gs,gm"]).unwrap();
        let Command::InstallLinks { dir, force, skip } = cli.command else {
            panic!("not parsed as install-links");
        };
        assert_eq!(dir, PathBuf::from("/tmp/bin"));
        assert!(!force);
        assert_eq!(skip, ["gs", "gm"]);
    }

    #[test]
    fn exec_requires_arguments() {
        assert!(Cli::try_parse_from(["gsct", "exec"]).is_err());
    }

    /// Converts an array of &str into Vec<String> for tests
    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn keeps_args_when_called_as_gsct() {
        let args = strings(&["/home/me/.cargo/bin/gsct", "status"]);
        assert_eq!(args_with_alias_name(args.clone()), args);
    }

    #[test]
    fn treats_alias_link_name_as_subcommand() {
        let args = strings(&["/home/me/.local/bin/gd", "1"]);
        assert_eq!(args_with_alias_name(args), ["gsct", "gd", "1"]);
    }
}
