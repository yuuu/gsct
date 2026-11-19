use std::env;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::aliases;

/// Creates symlinks named after the aliases in dir
pub fn install(dir: &Path, force: bool, skip: &[String]) -> Result<()> {
    let exe = env::current_exe().context("failed to find the path of the gsct executable")?;
    fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let dir = fs::canonicalize(dir)?;
    let path_var = env::var_os("PATH").unwrap_or_default();

    let names = aliases::SPECIAL
        .iter()
        .map(|(name, _)| *name)
        .chain(aliases::ALIASES.iter().map(|alias| alias.name));

    for name in names {
        if skip.iter().any(|s| s == name) {
            println!("skip     {name} (--skip)");
            continue;
        }

        let link = dir.join(name);

        // A file with the same name already exists
        if link.symlink_metadata().is_ok() {
            if fs::read_link(&link).ok() == Some(exe.clone()) {
                println!("exists   {name}");
                continue;
            }
            if !force {
                eprintln!("skip     {name} ({} already exists)", link.display());
                continue;
            }
            fs::remove_file(&link)?;
        } else if let Some(found) = find_in_path(name, &path_var, &dir) {
            // A command with the same name exists on PATH
            if !force {
                eprintln!(
                    "skip     {name} (collides with the existing command {})",
                    found.display()
                );
                continue;
            }
        }

        symlink(&exe, &link).with_context(|| format!("failed to create {}", link.display()))?;
        println!("created  {}", link.display());
    }

    Ok(())
}

fn find_in_path(name: &str, path_var: &OsStr, exclude: &Path) -> Option<PathBuf> {
    env::split_paths(path_var)
        .filter(|dir| dir != exclude)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_command_in_path() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("gd"), "").unwrap();
        let path_var = env::join_paths([dir.path()]).unwrap();

        assert_eq!(
            find_in_path("gd", &path_var, Path::new("/nowhere")),
            Some(dir.path().join("gd"))
        );
        assert_eq!(find_in_path("ga", &path_var, Path::new("/nowhere")), None);
    }

    #[test]
    fn ignores_excluded_directory() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("gd"), "").unwrap();
        let path_var = env::join_paths([dir.path()]).unwrap();

        assert_eq!(find_in_path("gd", &path_var, dir.path()), None);
    }
}
