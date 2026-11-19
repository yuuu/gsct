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
}
