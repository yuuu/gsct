use anyhow::{Context, Result};

use crate::{exec, expand, status};

/// git の alias 1つ分の定義
pub struct Alias {
    /// alias 名（例: "gd"）
    pub name: &'static str,
    /// git に渡す引数（例: ["diff"]）
    pub args: &'static [&'static str],
    /// 番号の引数をファイルパスに展開するか
    pub expand: bool,
}

/// 番号を展開する alias（ファイルを扱うコマンド）を作る
const fn with_numbers(name: &'static str, args: &'static [&'static str]) -> Alias {
    Alias {
        name,
        args,
        expand: true,
    }
}

/// 番号を展開しない alias を作る
const fn plain(name: &'static str, args: &'static [&'static str]) -> Alias {
    Alias {
        name,
        args,
        expand: false,
    }
}

const LOG_FORMAT: &str =
    "--pretty=format:%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset";

/// gsct 自身の機能を呼び出す特別な alias
pub const SPECIAL: [(&str, &str); 2] = [("gs", "gsct status"), ("ge", "gsct exec")];

/// scm_breeze の git_aliases を元にした alias の一覧
pub const ALIASES: &[Alias] = &[
    // ファイルを扱うコマンド（番号を使える）
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
    // その他のコマンド
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

/// 名前から alias を探す
fn find(name: &str) -> Option<&'static Alias> {
    ALIASES.iter().find(|alias| alias.name == name)
}

/// alias を実行する。args[0] が alias 名、残りが追加の引数
pub fn run(args: &[String]) -> Result<i32> {
    let (name, rest) = args.split_first().context("alias 名を指定してください")?;

    match name.as_str() {
        "gs" => {
            status::run()?;
            Ok(0)
        }
        "ge" => exec::run(rest),
        _ => {
            let alias = find(name).with_context(|| format!("不明な alias です: {name}"))?;

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

/// alias の一覧を表示する（scm_breeze の git_aliases 相当）
pub fn print_list() {
    for (name, command) in SPECIAL {
        println!("{name:>6}  =>  {command}");
    }

    for alias in ALIASES {
        // 空白を含む引数は見やすいように引用符で囲む
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
        let mark = if alias.expand { "  (番号可)" } else { "" };
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
        assert_eq!(names.len(), count, "alias 名が重複しています");
    }

    #[test]
    fn finds_alias_by_name() {
        let alias = find("gd").unwrap();
        assert_eq!(alias.args, &["diff"]);
        assert!(alias.expand);
        assert!(find("unknown").is_none());
    }
}
