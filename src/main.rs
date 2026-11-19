fn main() {
    println!("Hello, world!");
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
}
