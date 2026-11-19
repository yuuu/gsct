use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::git;

/// キャッシュファイルの場所（通常は .git/gsct-status）
fn cache_path() -> Result<PathBuf> {
    let path = git::output(&["rev-parse", "--git-path", "gsct-status"])?;
    Ok(PathBuf::from(path.trim()))
}

/// 番号順のファイル一覧をキャッシュに保存する
pub fn save(files: &[PathBuf]) -> Result<()> {
    write_to(&cache_path()?, files)
}

/// ファイル一覧を NUL 区切りで path に書き込む
fn write_to(path: &Path, files: &[PathBuf]) -> Result<()> {
    let content = files
        .iter()
        .map(|file| file.to_string_lossy())
        .collect::<Vec<_>>()
        .join("\0");
    fs::write(path, content).with_context(|| format!("{} に書き込めませんでした", path.display()))
}

/// キャッシュからファイル一覧を読み込む
pub fn load() -> Result<Vec<String>> {
    read_from(&cache_path()?)
}

/// path から NUL 区切りのファイル一覧を読み込む
fn read_from(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)
        .context("番号とファイルの対応が見つかりません。先に gs を実行してください")?;
    Ok(content
        .split('\0')
        .filter(|file| !file.is_empty())
        .map(String::from)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_files_separated_by_nul() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("gsct-status");
        let files = [PathBuf::from("/repo/a.rs"), PathBuf::from("/repo/b c.rs")];

        write_to(&path, &files).unwrap();

        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "/repo/a.rs\0/repo/b c.rs"
        );
    }

    #[test]
    fn reads_back_written_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("gsct-status");
        let files = [PathBuf::from("/repo/a.rs"), PathBuf::from("/repo/b c.rs")];

        write_to(&path, &files).unwrap();

        assert_eq!(read_from(&path).unwrap(), ["/repo/a.rs", "/repo/b c.rs"]);
    }

    #[test]
    fn missing_cache_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read_from(&dir.path().join("gsct-status")).is_err());
    }
}
