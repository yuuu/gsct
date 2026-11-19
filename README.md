# gsct

A Rust reimplementation of scm_breeze's Git Status Shortcuts and git aliases.
It uses no shell functions and never overrides existing commands such as `git` or `ls`.

## Installation

Prebuilt binaries for macOS and Linux are available on the [Releases](https://github.com/yuuu/gsct/releases) page. You don't need Rust or cargo.

| OS | CPU | File |
| --- | --- | --- |
| macOS | Apple Silicon | `gsct-aarch64-apple-darwin.tar.gz` |
| macOS | Intel | `gsct-x86_64-apple-darwin.tar.gz` |
| Linux | x86_64 | `gsct-x86_64-unknown-linux-musl.tar.gz` |
| Linux | ARM64 | `gsct-aarch64-unknown-linux-musl.tar.gz` |

The Linux binaries are statically linked, so they work on any distribution regardless of the glibc version. Each archive contains the `gsct` binary and `LICENSE`.

### 1. Download the binary

The following commands put the latest `gsct` for your OS and CPU into `~/.local/bin`.

```bash
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64)  target=aarch64-apple-darwin ;;
  Darwin-x86_64) target=x86_64-apple-darwin ;;
  Linux-x86_64)  target=x86_64-unknown-linux-musl ;;
  Linux-aarch64) target=aarch64-unknown-linux-musl ;;
  *) target=; echo "Unsupported platform: $(uname -s)-$(uname -m)" >&2 ;;
esac
[ -n "$target" ] && mkdir -p ~/.local/bin &&
  curl -fsSL "https://github.com/yuuu/gsct/releases/latest/download/gsct-${target}.tar.gz" |
  tar xzf - -C ~/.local/bin gsct
```

If `~/.local/bin` is not on your `PATH`, add it in your shell configuration file (such as `~/.bashrc` or `~/.zshrc`).

```bash
export PATH="$HOME/.local/bin:$PATH"
```

To verify the download, check it against the `.sha256` file published alongside it (on Linux, `sha256sum -c` works as well as `shasum`).

```bash
curl -fsSLO "https://github.com/yuuu/gsct/releases/latest/download/gsct-${target}.tar.gz"
curl -fsSLO "https://github.com/yuuu/gsct/releases/latest/download/gsct-${target}.tar.gz.sha256"
shasum -a 256 -c "gsct-${target}.tar.gz.sha256"
```

On macOS, a binary downloaded with a web browser is blocked because it is not signed. After extracting it, run `xattr -d com.apple.quarantine ~/.local/bin/gsct` (this is not needed with the `curl` commands above).

### 2. Create the alias links

```bash
gsct install-links ~/.local/bin
```

This creates symlinks to `gsct` named `ga`, `gd`, and so on. Aliases whose names collide with existing commands (such as Ghostscript's `gs`) are skipped. Add `--force` to create them anyway.

### Updating and uninstalling

To update, run step 1 again. The links point to the `gsct` binary, so you don't need to recreate them.

To uninstall, remove `gsct` and the links pointing to it from `~/.local/bin`.

```bash
find ~/.local/bin -lname "$HOME/.local/bin/gsct" -delete
rm ~/.local/bin/gsct
```

### Installing with cargo

If you have a Rust toolchain, you can also build and install from source.

```bash
cargo install --git https://github.com/yuuu/gsct
gsct install-links ~/.local/bin
```

## Usage

    gs            # git status with numbered files
    ga 1 3-5      # git add (by number or range)
    ga 1,3        # git add (comma-separated numbers)
    gd 2          # git diff
    ge vim 1      # use the numbers with any command
    gsct aliases  # list the aliases

`gs` saves the mapping between numbers and files to `.git/gsct-status`.

## Development

    mise run ci   # format check, lint and tests (same as CI)

### Releasing

Bump `version` in `Cargo.toml`, merge it into `main`, and then push a tag with the same version.

```bash
git switch main && git pull
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions (`.github/workflows/release.yml`) builds the binaries for each platform and publishes them to a GitHub Release. If the tag does not match `version` in `Cargo.toml`, the workflow fails without publishing.

## License

[MIT](LICENSE)
