use std::fs;
use std::path::Path;
use std::process::{Command, Output};

/// テスト対象の gsct をディレクトリ dir で実行する
fn gsct(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gsct"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

/// git を実行して標準出力を返す
fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn status_numbers_files_and_alias_uses_the_numbers() {
    let repo = tempfile::tempdir().unwrap();
    git(repo.path(), &["init", "-q"]);
    fs::write(repo.path().join("a.txt"), "a").unwrap();
    fs::write(repo.path().join("b.txt"), "b").unwrap();

    let status = gsct(repo.path(), &["status"]);
    assert!(status.status.success());
    let stdout = String::from_utf8(status.stdout).unwrap();
    assert!(stdout.contains("[1] a.txt"), "{stdout}");
    assert!(stdout.contains("[2] b.txt"), "{stdout}");

    let add = gsct(repo.path(), &["ga", "2"]);
    assert!(add.status.success());
    assert_eq!(
        git(repo.path(), &["diff", "--cached", "--name-only"]),
        "b.txt\n"
    );
}

#[test]
fn unknown_alias_fails() {
    let repo = tempfile::tempdir().unwrap();
    let output = gsct(repo.path(), &["gxx"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("不明な alias です")
    );
}

#[test]
fn install_links_skips_names_of_existing_commands() {
    let bin = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    fs::write(other.path().join("gd"), "").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_gsct"))
        .arg("install-links")
        .arg(bin.path())
        .env("PATH", other.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(bin.path().join("ga").is_symlink());
    assert!(!bin.path().join("gd").exists());
}

#[test]
fn alias_link_runs_the_alias() {
    let repo = tempfile::tempdir().unwrap();
    git(repo.path(), &["init", "-q"]);
    let bin = tempfile::tempdir().unwrap();
    let install = gsct(
        repo.path(),
        &["install-links", bin.path().to_str().unwrap(), "--force"],
    );
    assert!(install.status.success());

    // gtop = git rev-parse --show-toplevel
    let output = Command::new(bin.path().join("gtop"))
        .current_dir(repo.path())
        .output()
        .unwrap();

    let toplevel = String::from_utf8(output.stdout).unwrap();
    let expected = fs::canonicalize(repo.path()).unwrap();
    assert_eq!(toplevel.trim(), expected.to_str().unwrap());
}
