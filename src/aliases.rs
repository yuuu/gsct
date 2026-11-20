use anyhow::{Context, Result};

use crate::{exec, expand, status};

/// Definition of a single git alias
pub struct Alias {
    /// Alias name (e.g. "gd")
    pub name: &'static str,
    /// Arguments passed to git (e.g. ["diff"])
    pub args: &'static [&'static str],
    /// Whether numeric arguments are expanded to file paths
    pub expand: bool,
}

/// Creates an alias that expands numbers (for commands that take files)
const fn with_numbers(name: &'static str, args: &'static [&'static str]) -> Alias {
    Alias {
        name,
        args,
        expand: true,
    }
}

/// Creates an alias that does not expand numbers
const fn plain(name: &'static str, args: &'static [&'static str]) -> Alias {
    Alias {
        name,
        args,
        expand: false,
    }
}

const LOG_FORMAT: &str =
    "--pretty=format:%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset";

/// Special aliases that call gsct's own features
pub const SPECIAL: [(&str, &str); 2] = [("gs", "gsct status"), ("ge", "gsct exec")];

/// Aliases based on scm_breeze's git_aliases
pub const ALIASES: &[Alias] = &[
    // Commands that take files (numbers can be used)
    with_numbers("ga", &["add"]),
    with_numbers("gap", &["add", "-p"]),
    with_numbers("gau", &["add", "-u"]),
    with_numbers("gco", &["checkout"]),
    with_numbers("gc", &["commit"]),
    with_numbers("gcv", &["commit", "--verbose"]),
    with_numbers("grs", &["reset"]),
    with_numbers("grsh", &["reset", "--hard"]),
    with_numbers("grm", &["rm"]),
    with_numbers("grt", &["restore"]),
    with_numbers("gbl", &["blame"]),
    with_numbers("gd", &["diff"]),
    with_numbers("gdnw", &["diff", "-w"]),
    with_numbers("gdf", &["diff"]),
    with_numbers("gdw", &["diff", "--word-diff"]),
    with_numbers("gdc", &["diff", "--cached"]),
    with_numbers("gdt", &["difftool"]),
    with_numbers("gmt", &["mergetool"]),
    // Other commands
    plain("gaa", &["add", "--all", "."]),
    plain("gcb", &["checkout", "-b"]),
    plain("gcm", &["commit", "--amend"]),
    plain("gcmh", &["commit", "--amend", "-C", "HEAD"]),
    plain("gch", &["commit", "-C", "HEAD"]),
    plain("gcl", &["clone"]),
    plain("gf", &["fetch"]),
    plain("gfa", &["fetch", "--all"]),
    plain("gpl", &["pull"]),
    plain("gplr", &["pull", "--rebase"]),
    plain("gps", &["push"]),
    plain("gpsf", &["push", "-f"]),
    plain("gst", &["status"]),
    plain("gss", &["status", "-s"]),
    plain("gce", &["clean"]),
    plain("gcef", &["clean", "-fd"]),
    plain("gr", &["remote", "-v"]),
    plain("gb", &["branch"]),
    plain("gba", &["branch", "-a"]),
    plain("gbm", &["branch", "-m"]),
    plain("gbd", &["branch", "-d"]),
    plain("gbD", &["branch", "-D"]),
    plain("grb", &["rebase"]),
    plain("grbi", &["rebase", "-i"]),
    plain("grbc", &["rebase", "--continue"]),
    plain("grba", &["rebase", "--abort"]),
    plain("grsl", &["reset", "HEAD~"]),
    plain("gm", &["merge"]),
    plain("gmnff", &["merge", "--no-ff"]),
    plain("gmff", &["merge", "--ff"]),
    plain("gcp", &["cherry-pick"]),
    plain("gl", &["log", "--graph", LOG_FORMAT, "--abbrev-commit"]),
    plain(
        "gla",
        &[
            "log",
            "--graph",
            LOG_FORMAT,
            "--abbrev-commit",
            "--branches",
            "--remotes",
        ],
    ),
    plain("glst", &["log", "--stat", "--max-count=5"]),
    plain("glg", &["log", "--graph", "--max-count=5"]),
    plain("gsh", &["show"]),
    plain("gsm", &["show", "--summary"]),
    plain("gash", &["stash"]),
    plain("gasha", &["stash", "apply"]),
    plain("gashp", &["stash", "pop"]),
    plain("gashl", &["stash", "list"]),
    plain("gt", &["tag"]),
    plain("gsu", &["submodule", "update", "--init"]),
    plain("gsur", &["submodule", "update", "--init", "--recursive"]),
    plain("gtop", &["rev-parse", "--show-toplevel"]),
    plain("gwc", &["whatchanged"]),
    plain("gapp", &["apply"]),
    plain("gsw", &["switch"]),
];

/// Finds an alias by name
pub fn find(name: &str) -> Option<&'static Alias> {
    ALIASES.iter().find(|alias| alias.name == name)
}

/// Runs an alias. args[0] is the alias name and the rest are extra arguments
pub fn run(args: &[String]) -> Result<i32> {
    let (name, rest) = args.split_first().context("specify an alias name")?;

    match name.as_str() {
        "gs" => {
            status::run()?;
            Ok(0)
        }
        "ge" => exec::run(rest),
        _ => {
            let alias = find(name).with_context(|| format!("unknown alias: {name}"))?;

            let rest = if alias.expand {
                expand::expand_with_cache(rest)?
            } else {
                rest.to_vec()
            };

            let mut git_args: Vec<String> = alias.args.iter().map(|arg| arg.to_string()).collect();
            git_args.extend(rest);
            exec::run_command("git", &git_args)
        }
    }
}

/// Prints the list of aliases (like scm_breeze's git_aliases)
pub fn print_list() {
    for (name, command) in SPECIAL {
        println!("{name:>6}  =>  {command}");
    }

    for alias in ALIASES {
        // Quote arguments that contain spaces for readability
        let args: Vec<String> = alias
            .args
            .iter()
            .map(|arg| {
                if arg.contains(' ') {
                    format!("'{arg}'")
                } else {
                    arg.to_string()
                }
            })
            .collect();
        let mark = if alias.expand { "  (numbers)" } else { "" };
        println!("{:>6}  =>  git {}{mark}", alias.name, args.join(" "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_names_are_unique() {
        let mut names: Vec<&str> = SPECIAL.iter().map(|(name, _)| *name).collect();
        names.extend(ALIASES.iter().map(|alias| alias.name));

        let count = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), count, "duplicate alias names");
    }

    #[test]
    fn finds_alias_by_name() {
        let alias = find("gd").unwrap();
        assert_eq!(alias.args, &["diff"]);
        assert!(alias.expand);
        assert!(find("unknown").is_none());
    }
}
