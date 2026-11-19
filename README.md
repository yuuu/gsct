# gsct

scm_breeze の Git Status Shortcuts と git alias を Rust で再実装した CLI です。
シェル関数を使わず、`git` や `ls` などの既存コマンドも上書きしません。

## インストール

    cargo install --path .
    gsct install-links ~/.local/bin

## 使い方

    gs            # 番号付きの git status
    ga 1 3-5      # git add（番号や範囲で指定）
    gd 2          # git diff
    ge vim 1      # 任意のコマンドで番号を使う
    gsct aliases  # alias の一覧

番号とファイルの対応は `gs` 実行時に `.git/gsct-status` へ保存されます。

## 開発

    mise run ci   # フォーマット・静的解析・テスト（CI と同じ）
