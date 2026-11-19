use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::git;

/// ファイルが属するグループ（表示順に並べている）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Staged,
    Unmerged,
    Unstaged,
    Untracked,
}

impl Group {
    /// 表示する順番
    const ALL: [Group; 4] = [
        Group::Staged,
        Group::Unmerged,
        Group::Unstaged,
        Group::Untracked,
    ];

    fn heading(self) -> &'static str {
        match self {
            Group::Staged => "Changes to be committed",
            Group::Unmerged => "Unmerged paths",
            Group::Unstaged => "Changes not staged for commit",
            Group::Untracked => "Untracked files",
        }
    }

    /// ANSI カラーコードの色番号（33 = 黄、31 = 赤、32 = 緑、36 = シアン）
    fn color(self) -> &'static str {
        match self {
            Group::Staged => "33",
            Group::Unmerged => "31",
            Group::Unstaged => "32",
            Group::Untracked => "36",
        }
    }
}

/// ファイルの変更の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Modified,
    NewFile,
    Deleted,
    Renamed,
    Copied,
    TypeChange,
    Untracked,
    BothDeleted,
    AddedByUs,
    DeletedByThem,
    AddedByThem,
    DeletedByUs,
    BothAdded,
    BothModified,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Kind::Modified => "modified",
            Kind::NewFile => "new file",
            Kind::Deleted => "deleted",
            Kind::Renamed => "renamed",
            Kind::Copied => "copied",
            Kind::TypeChange => "typechange",
            Kind::Untracked => "untracked",
            Kind::BothDeleted => "both deleted",
            Kind::AddedByUs => "added by us",
            Kind::DeletedByThem => "deleted by them",
            Kind::AddedByThem => "added by them",
            Kind::DeletedByUs => "deleted by us",
            Kind::BothAdded => "both added",
            Kind::BothModified => "both modified",
        }
    }

    fn color(self) -> &'static str {
        match self {
            Kind::Deleted | Kind::BothDeleted | Kind::DeletedByThem | Kind::DeletedByUs => "0;31",
            Kind::Modified | Kind::BothModified => "0;32",
            Kind::NewFile
            | Kind::Copied
            | Kind::AddedByUs
            | Kind::AddedByThem
            | Kind::BothAdded => "0;33",
            Kind::Renamed => "0;34",
            Kind::TypeChange => "0;35",
            Kind::Untracked => "0;36",
        }
    }
}

/// git status の1行分（1ファイル分）の情報
#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    pub group: Group,
    pub kind: Kind,
    pub path: String,
}

impl Entry {
    fn new(group: Group, kind: Kind, path: &str) -> Entry {
        Entry {
            group,
            kind,
            path: path.to_string(),
        }
    }
}

/// git status 全体の情報
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Status {
    pub branch: String,
    pub ahead: u32,
    pub behind: u32,
    pub entries: Vec<Entry>,
}

/// `git status --porcelain -b -z` の出力を解析する
pub fn parse(output: &str) -> Status {
    let mut status = Status::default();
    let mut fields = output.split('\0');

    while let Some(field) = fields.next() {
        // 先頭の "## " で始まる項目はブランチ情報
        if let Some(branch_line) = field.strip_prefix("## ") {
            parse_branch(branch_line, &mut status);
            continue;
        }

        // それ以外は "XY パス" の形式（X = ステージ、Y = ワークツリーの状態）
        let mut chars = field.chars();
        let (Some(x), Some(y)) = (chars.next(), chars.next()) else {
            continue;
        };
        let Some(path) = field.get(3..) else {
            continue;
        };

        // リネーム・コピーの場合は、次の項目に元のパスが入っているので読み飛ばす
        if x == 'R' || x == 'C' {
            fields.next();
        }

        status.entries.extend(classify(x, y, path));
    }

    status
}

/// ブランチ情報を解析する
/// 例: "main...origin/main [ahead 1, behind 2]"、"No commits yet on main"
fn parse_branch(line: &str, status: &mut Status) {
    let line = line.strip_prefix("No commits yet on ").unwrap_or(line);

    let (name, tracking) = match line.split_once(" [") {
        Some((name, tracking)) => (name, tracking.trim_end_matches(']')),
        None => (line, ""),
    };

    status.branch = match name.split_once("...") {
        Some((local, _remote)) => local.to_string(),
        None => name.to_string(),
    };

    for item in tracking.split(", ") {
        if let Some(n) = item.strip_prefix("ahead ") {
            status.ahead = n.parse().unwrap_or(0);
        } else if let Some(n) = item.strip_prefix("behind ") {
            status.behind = n.parse().unwrap_or(0);
        }
    }
}

/// XY の2文字から、どのグループのどの変更かを判定する
/// 1つのファイルが「ステージ済み」と「未ステージ」の両方に入ることもあるので Vec で返す
fn classify(x: char, y: char, path: &str) -> Vec<Entry> {
    // コンフリクト中のファイル
    let unmerged = match (x, y) {
        ('D', 'D') => Some(Kind::BothDeleted),
        ('A', 'U') => Some(Kind::AddedByUs),
        ('U', 'D') => Some(Kind::DeletedByThem),
        ('U', 'A') => Some(Kind::AddedByThem),
        ('D', 'U') => Some(Kind::DeletedByUs),
        ('A', 'A') => Some(Kind::BothAdded),
        ('U', 'U') => Some(Kind::BothModified),
        _ => None,
    };
    if let Some(kind) = unmerged {
        return vec![Entry::new(Group::Unmerged, kind, path)];
    }

    // 未追跡のファイル
    if x == '?' && y == '?' {
        return vec![Entry::new(Group::Untracked, Kind::Untracked, path)];
    }

    let mut entries = Vec::new();

    // X: ステージ済みの変更
    let staged = match x {
        'M' => Some(Kind::Modified),
        'A' => Some(Kind::NewFile),
        'D' => Some(Kind::Deleted),
        'R' => Some(Kind::Renamed),
        'C' => Some(Kind::Copied),
        'T' => Some(Kind::TypeChange),
        _ => None,
    };
    if let Some(kind) = staged {
        entries.push(Entry::new(Group::Staged, kind, path));
    }

    // Y: まだステージしていない変更
    let unstaged = match y {
        'M' => Some(Kind::Modified),
        'D' => Some(Kind::Deleted),
        'T' => Some(Kind::TypeChange),
        _ => None,
    };
    if let Some(kind) = unstaged {
        entries.push(Entry::new(Group::Unstaged, kind, path));
    }

    entries
}

/// `gs` の本体: git status を番号付きで表示する
pub fn run() -> Result<()> {
    let root = PathBuf::from(git::output(&["rev-parse", "--show-toplevel"])?.trim());
    let output = git::output(&["status", "--porcelain", "-b", "-z"])?;
    let status = parse(&output);

    let cwd = std::env::current_dir()?;
    let color = std::io::stdout().is_terminal();
    let (text, files) = render(&status, &root, &cwd, color);
    print!("{text}");

    cache::save(&files)?;

    Ok(())
}

/// 文字に色を付ける（color が false なら何もしない）
struct Painter {
    enabled: bool,
}

impl Painter {
    fn paint(&self, code: &str, text: &str) -> String {
        if self.enabled {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

/// 表示用の文字列と、番号順に並べたファイルの絶対パスの一覧を作る
pub fn render(status: &Status, root: &Path, cwd: &Path, color: bool) -> (String, Vec<PathBuf>) {
    let p = Painter { enabled: color };
    let mut out = String::new();
    let mut files = Vec::new();

    // リモートとのコミット差分（例: "-2/+1"）
    let mut diff = Vec::new();
    if status.behind > 0 {
        diff.push(format!("-{}", status.behind));
    }
    if status.ahead > 0 {
        diff.push(format!("+{}", status.ahead));
    }
    let diff = if diff.is_empty() {
        String::new()
    } else {
        format!(
            "  {}  {}",
            p.paint("2;37", "|"),
            p.paint("0;33", &diff.join("/"))
        )
    };

    let header = format!(
        "{} On branch: {}{}",
        p.paint("2;37", "#"),
        p.paint("1", &status.branch),
        diff
    );

    if status.entries.is_empty() {
        let clean = p.paint("0;32", "No changes (working directory clean)");
        out.push_str(&format!("{header}  {}  {clean}\n", p.paint("2;37", "|")));
        return (out, files);
    }

    out.push_str(&format!("{header}\n{}\n", p.paint("2;37", "#")));

    // 10件以上あるときは番号の桁をそろえる
    let width = status.entries.len().to_string().len();

    for group in Group::ALL {
        let entries: Vec<&Entry> = status.entries.iter().filter(|e| e.group == group).collect();
        if entries.is_empty() {
            continue;
        }

        let hash = p.paint(&format!("0;{}", group.color()), "#");
        let arrow = p.paint(&format!("1;{}", group.color()), "➤");
        out.push_str(&format!("{arrow} {}\n{hash}\n", group.heading()));

        for entry in entries {
            let absolute = root.join(&entry.path);
            let relative = pathdiff::diff_paths(&absolute, cwd).unwrap_or(absolute.clone());
            files.push(absolute);

            let label = p.paint(entry.kind.color(), &format!("{:>15}", entry.kind.label()));
            let number = format!(
                "{}{:>width$}{}",
                p.paint("2;37", "["),
                files.len(),
                p.paint("2;37", "]")
            );
            let path = p.paint(
                &format!("0;{}", group.color()),
                &relative.display().to_string(),
            );
            out.push_str(&format!("{hash} {label}: {number} {path}\n"));
        }

        out.push_str(&format!("{hash}\n"));
    }

    (out, files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_branch_with_ahead_and_behind() {
        let status = parse("## main...origin/main [ahead 1, behind 2]\0");
        assert_eq!(status.branch, "main");
        assert_eq!(status.ahead, 1);
        assert_eq!(status.behind, 2);
    }

    #[test]
    fn parses_branch_without_commits() {
        let status = parse("## No commits yet on main\0");
        assert_eq!(status.branch, "main");
    }

    #[test]
    fn parses_each_group() {
        let status = parse("## main\0M  staged.rs\0 M unstaged.rs\0UU conflict.rs\0?? new.txt\0");
        assert_eq!(
            status.entries,
            vec![
                Entry::new(Group::Staged, Kind::Modified, "staged.rs"),
                Entry::new(Group::Unstaged, Kind::Modified, "unstaged.rs"),
                Entry::new(Group::Unmerged, Kind::BothModified, "conflict.rs"),
                Entry::new(Group::Untracked, Kind::Untracked, "new.txt"),
            ]
        );
    }

    #[test]
    fn file_can_be_both_staged_and_unstaged() {
        let status = parse("## main\0AM both.rs\0");
        assert_eq!(
            status.entries,
            vec![
                Entry::new(Group::Staged, Kind::NewFile, "both.rs"),
                Entry::new(Group::Unstaged, Kind::Modified, "both.rs"),
            ]
        );
    }

    #[test]
    fn skips_original_path_of_rename() {
        let status = parse("## main\0R  new.rs\0old.rs\0?? a.txt\0");
        assert_eq!(
            status.entries,
            vec![
                Entry::new(Group::Staged, Kind::Renamed, "new.rs"),
                Entry::new(Group::Untracked, Kind::Untracked, "a.txt"),
            ]
        );
    }

    #[test]
    fn renders_numbered_files_relative_to_cwd() {
        let status = parse("## main [ahead 1]\0M  src/main.rs\0?? memo.txt\0");
        let (text, files) = render(&status, Path::new("/repo"), Path::new("/repo/src"), false);

        let expected = "\
# On branch: main  |  +1
#
➤ Changes to be committed
#
#        modified: [1] main.rs
#
➤ Untracked files
#
#       untracked: [2] ../memo.txt
#
";
        assert_eq!(text, expected);
        assert_eq!(
            files,
            vec![
                PathBuf::from("/repo/src/main.rs"),
                PathBuf::from("/repo/memo.txt")
            ]
        );
    }
}
