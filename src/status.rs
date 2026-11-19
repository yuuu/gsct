/// ファイルが属するグループ（表示順に並べている）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Staged,
    Unmerged,
    Unstaged,
    Untracked,
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
    todo!()
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
}
