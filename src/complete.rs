use std::collections::BTreeSet;

use anyhow::{Result, bail};

use crate::{aliases, git};

/// What an alias completes
#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    /// Local branches, remote-tracking branches and their names without the remote
    /// (checkout and switch create a local branch from "origin/foo" when given "foo")
    Checkout,
    /// No candidate for the new branch name, then a start point (checkout -b)
    StartPoint,
    /// Local branches only (branch -m, -d, -D)
    LocalBranch,
    /// Local and remote-tracking branches (merge, rebase, log, ...)
    Ref,
    /// A remote first, then branches (push, pull, fetch)
    Remote,
}

/// Decides what an alias completes from the git arguments it runs
pub fn kind(args: &[&str]) -> Option<Kind> {
    match args {
        ["checkout" | "switch"] => Some(Kind::Checkout),
        ["checkout", "-b"] => Some(Kind::StartPoint),
        ["branch", "-m" | "-d" | "-D"] => Some(Kind::LocalBranch),
        ["merge", ..] | ["rebase"] | ["rebase", "-i"] | ["cherry-pick"] | ["log", ..] => {
            Some(Kind::Ref)
        }
        ["show", ..] => Some(Kind::Ref),
        ["push"] | ["push", "-f"] | ["pull"] | ["pull", "--rebase"] | ["fetch"] => {
            Some(Kind::Remote)
        }
        _ => None,
    }
}

/// Names of the aliases that have completion
pub fn alias_names() -> impl Iterator<Item = &'static str> {
    aliases::ALIASES
        .iter()
        .filter(|alias| kind(alias.args).is_some())
        .map(|alias| alias.name)
}

/// Branches and remotes of the repository
#[derive(Default)]
pub struct Repo {
    /// e.g. "main"
    pub local: Vec<String>,
    /// e.g. "origin/main"
    pub remote: Vec<String>,
    /// e.g. "origin"
    pub remotes: Vec<String>,
}

impl Repo {
    fn load() -> Result<Self> {
        let refs = git::output(&[
            "for-each-ref",
            "--format=%(refname)",
            "refs/heads",
            "refs/remotes",
        ])?;
        let mut repo = Repo::default();
        for line in refs.lines() {
            if let Some(name) = line.strip_prefix("refs/heads/") {
                repo.local.push(name.to_string());
            } else if let Some(name) = line.strip_prefix("refs/remotes/") {
                // Skip symbolic refs such as "origin/HEAD"
                if !name.ends_with("/HEAD") {
                    repo.remote.push(name.to_string());
                }
            }
        }
        repo.remotes = git::output(&["remote"])?
            .lines()
            .map(str::to_string)
            .collect();
        Ok(repo)
    }

    /// Remote-tracking branches with the remote name removed (only the given remote if specified)
    fn remote_branch_names(&self, remote: Option<&str>) -> Vec<String> {
        self.remote
            .iter()
            .filter_map(|name| {
                let (r, branch) = self.split_remote(name)?;
                (remote.is_none_or(|remote| remote == r)).then(|| branch.to_string())
            })
            .collect()
    }

    /// Splits "origin/feature/x" into ("origin", "feature/x"), even if the remote name contains "/"
    fn split_remote<'a>(&self, name: &'a str) -> Option<(&'a str, &'a str)> {
        self.remotes
            .iter()
            .filter_map(|remote| {
                let branch = name.strip_prefix(remote.as_str())?.strip_prefix('/')?;
                Some((&name[..remote.len()], branch))
            })
            .max_by_key(|(remote, _)| remote.len())
    }
}

/// Implements `gsct __complete <alias> <words...>`: prints candidates for the last word
pub fn run(args: &[String]) -> Result<()> {
    let Some((name, words)) = args.split_first() else {
        bail!("specify an alias name");
    };
    let Some(kind) = aliases::find(name).and_then(|alias| kind(alias.args)) else {
        return Ok(());
    };
    // Outside a repository there is nothing to complete
    let Ok(repo) = Repo::load() else {
        return Ok(());
    };
    for candidate in candidates(&kind, words, &repo) {
        println!("{candidate}");
    }
    Ok(())
}

/// Candidates for the last of words (the word being completed), sorted and without duplicates
pub fn candidates(kind: &Kind, words: &[String], repo: &Repo) -> Vec<String> {
    let Some((current, before)) = words.split_last() else {
        return Vec::new();
    };
    // Options are left to the shell
    if current.starts_with('-') {
        return Vec::new();
    }
    // Arguments that are not options before the current word
    let positional: Vec<&str> = before
        .iter()
        .map(String::as_str)
        .filter(|word| !word.starts_with('-'))
        .collect();
    // Paths come after "--"
    if before.iter().any(|word| word == "--") {
        return Vec::new();
    }

    let names: Vec<String> = match kind {
        Kind::Checkout => [
            repo.local.clone(),
            repo.remote.clone(),
            repo.remote_branch_names(None),
        ]
        .concat(),
        Kind::StartPoint if positional.is_empty() => Vec::new(),
        Kind::StartPoint | Kind::Ref => [repo.local.clone(), repo.remote.clone()].concat(),
        Kind::LocalBranch => repo.local.clone(),
        Kind::Remote => match positional.as_slice() {
            [] => repo.remotes.clone(),
            [remote, ..] => [repo.local.clone(), repo.remote_branch_names(Some(remote))].concat(),
        },
    };

    names
        .into_iter()
        .filter(|name| name.starts_with(current.as_str()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Prints the completion script for a shell
pub fn print_script(shell: &str) -> Result<()> {
    let names: Vec<&str> = alias_names().collect();
    let names = names.join(" ");
    match shell {
        "bash" => print!(
            r#"# gsct completion for bash
_gsct_complete() {{
  local IFS=$'\n'
  COMPREPLY=($(gsct __complete "${{COMP_WORDS[0]##*/}}" "${{COMP_WORDS[@]:1:COMP_CWORD}}" 2>/dev/null))
}}
complete -o bashdefault -o default -F _gsct_complete {names}
"#
        ),
        "zsh" => print!(
            r#"# gsct completion for zsh (requires compinit)
_gsct_complete() {{
  local -a candidates
  candidates=("${{(@f)$(gsct __complete "${{words[1]:t}}" "${{(@)words[2,CURRENT]}}" 2>/dev/null)}}")
  candidates=(${{candidates:#}})
  if (( ${{#candidates}} )); then
    compadd -a candidates
  else
    _files
  fi
}}
compdef _gsct_complete {names}
"#
        ),
        _ => bail!("unsupported shell: {shell} (bash or zsh)"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn repo() -> Repo {
        Repo {
            local: strings(&["main", "feature/a"]),
            remote: strings(&["origin/main", "origin/feature/b", "upstream/main"]),
            remotes: strings(&["origin", "upstream"]),
        }
    }

    #[test]
    fn decides_kind_from_git_arguments() {
        assert_eq!(kind(&["checkout"]), Some(Kind::Checkout));
        assert_eq!(kind(&["checkout", "-b"]), Some(Kind::StartPoint));
        assert_eq!(kind(&["push", "-f"]), Some(Kind::Remote));
        assert_eq!(kind(&["rebase", "--continue"]), None);
        assert_eq!(kind(&["fetch", "--all"]), None);
        assert_eq!(kind(&["diff"]), None);
    }

    #[test]
    fn every_alias_with_completion_exists() {
        assert!(alias_names().any(|name| name == "gco"));
        assert!(alias_names().any(|name| name == "gps"));
        assert!(!alias_names().any(|name| name == "gd"));
    }

    #[test]
    fn checkout_completes_branches_and_remote_branch_names() {
        let words = strings(&["f"]);
        assert_eq!(
            candidates(&Kind::Checkout, &words, &repo()),
            ["feature/a", "feature/b"]
        );
        let words = strings(&["o"]);
        assert_eq!(
            candidates(&Kind::Checkout, &words, &repo()),
            ["origin/feature/b", "origin/main"]
        );
    }

    #[test]
    fn push_completes_remote_then_branch() {
        let words = strings(&[""]);
        assert_eq!(
            candidates(&Kind::Remote, &words, &repo()),
            ["origin", "upstream"]
        );
        let words = strings(&["-u", "origin", ""]);
        assert_eq!(
            candidates(&Kind::Remote, &words, &repo()),
            ["feature/a", "feature/b", "main"]
        );
    }

    #[test]
    fn start_point_skips_new_branch_name() {
        assert!(candidates(&Kind::StartPoint, &strings(&["m"]), &repo()).is_empty());
        assert_eq!(
            candidates(&Kind::StartPoint, &strings(&["new", "up"]), &repo()),
            ["upstream/main"]
        );
    }

    #[test]
    fn options_and_paths_are_not_completed() {
        assert!(candidates(&Kind::Ref, &strings(&["-"]), &repo()).is_empty());
        assert!(candidates(&Kind::Checkout, &strings(&["--", "m"]), &repo()).is_empty());
    }

    #[test]
    fn local_branch_completes_only_local() {
        assert_eq!(
            candidates(&Kind::LocalBranch, &strings(&[""]), &repo()),
            ["feature/a", "main"]
        );
    }
}
