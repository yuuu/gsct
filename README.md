# gsct

scm_breeze の Git Status Shortcuts と git alias を Rust で再実装した CLI です。
シェル関数を使わず、`git` や `ls` などの既存コマンドも上書きしません。

## インストール

[Releases](https://github.com/yuuu/gsct/releases) に、macOS と Linux 向けのビルド済みバイナリがあります。Rust や cargo は必要ありません。

| OS | CPU | ファイル |
| --- | --- | --- |
| macOS | Apple Silicon | `gsct-aarch64-apple-darwin.tar.gz` |
| macOS | Intel | `gsct-x86_64-apple-darwin.tar.gz` |
| Linux | x86_64 | `gsct-x86_64-unknown-linux-musl.tar.gz` |
| Linux | ARM64 | `gsct-aarch64-unknown-linux-musl.tar.gz` |

Linux 版は静的リンクなので、ディストリビューションや glibc のバージョンを問わず動きます。各ファイルには `gsct` 本体と `LICENSE` が入っています。

### 1. ダウンロードして配置する

次のコマンドで、OS と CPU に合った最新版の `gsct` を `~/.local/bin` に置きます。

```bash
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64)  target=aarch64-apple-darwin ;;
  Darwin-x86_64) target=x86_64-apple-darwin ;;
  Linux-x86_64)  target=x86_64-unknown-linux-musl ;;
  Linux-aarch64) target=aarch64-unknown-linux-musl ;;
  *) target=; echo "未対応の環境です: $(uname -s)-$(uname -m)" >&2 ;;
esac
[ -n "$target" ] && mkdir -p ~/.local/bin &&
  curl -fsSL "https://github.com/yuuu/gsct/releases/latest/download/gsct-${target}.tar.gz" |
  tar xzf - -C ~/.local/bin gsct
```

`~/.local/bin` が `PATH` に入っていない場合は、シェルの設定ファイル（`~/.bashrc` や `~/.zshrc`）に追加してください。

```bash
export PATH="$HOME/.local/bin:$PATH"
```

ダウンロードしたファイルを確かめたい場合は、同じ場所にある `.sha256` と照合します（Linux では `shasum` の代わりに `sha256sum -c` も使えます）。

```bash
curl -fsSLO "https://github.com/yuuu/gsct/releases/latest/download/gsct-${target}.tar.gz"
curl -fsSLO "https://github.com/yuuu/gsct/releases/latest/download/gsct-${target}.tar.gz.sha256"
shasum -a 256 -c "gsct-${target}.tar.gz.sha256"
```

macOS でブラウザからダウンロードした場合は、署名のないアプリとして実行がブロックされます。展開した後に `xattr -d com.apple.quarantine ~/.local/bin/gsct` を実行してください（上の `curl` を使った手順では不要です）。

### 2. alias のリンクを作る

```bash
gsct install-links ~/.local/bin
```

`ga` や `gd` などの名前で `gsct` へのシンボリックリンクを作ります。既存のコマンドと名前が衝突する alias（例: Ghostscript の `gs`）は作りません。衝突しても作る場合は `--force` を付けます。

### 更新・アンインストール

更新は「1. ダウンロードして配置する」をもう一度実行します。リンクは `gsct` 本体を指しているので、作り直す必要はありません。

アンインストールは、`~/.local/bin` の `gsct` と、`gsct` を指しているリンクを削除します。

```bash
find ~/.local/bin -lname "$HOME/.local/bin/gsct" -delete
rm ~/.local/bin/gsct
```

### cargo でインストールする

Rust の開発環境がある場合は、ソースからビルドしてインストールすることもできます。

```bash
cargo install --git https://github.com/yuuu/gsct
gsct install-links ~/.local/bin
```

## 使い方

    gs            # 番号付きの git status
    ga 1 3-5      # git add（番号や範囲で指定）
    gd 2          # git diff
    ge vim 1      # 任意のコマンドで番号を使う
    gsct aliases  # alias の一覧

番号とファイルの対応は `gs` 実行時に `.git/gsct-status` へ保存されます。

## 開発

    mise run ci   # フォーマット・静的解析・テスト（CI と同じ）

### リリース

`Cargo.toml` の `version` を上げて `main` にマージした後、同じ番号のタグを push します。

```bash
git switch main && git pull
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions（`.github/workflows/release.yml`）が各プラットフォーム向けのバイナリをビルドし、GitHub Release に公開します。タグと `Cargo.toml` の `version` が一致しない場合は、公開せずに失敗します。

## ライセンス

[MIT](LICENSE)
