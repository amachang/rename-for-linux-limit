# rename-for-linux-limit

Linux のファイル名長制限（255バイト）に対応するためのユーティリティ。

macOS などで作成した長いファイル名を Linux ファイルシステムにコピーする際に使用。

## バイナリ

### rename-for-linux-limit

単一ファイルのリネーム用。ファイル名が 255 バイトを超える場合、短縮した名前にリネームする。

```bash
# ファイル名を短縮してリネーム
rename-for-linux-limit /path/to/very-long-filename.txt

# 短縮後のファイル名を表示するだけ（リネームしない）
rename-for-linux-limit -s /path/to/very-long-filename.txt

# 別ディレクトリに移動しつつリネーム
rename-for-linux-limit -d /dst/dir /path/to/file.txt
```

### merge-dirs-for-linux-limit

ディレクトリ全体のマージ用。外付け HDD などから別のディレクトリにファイルを移動する際に使用。

**機能**:
- ファイル名の自動短縮（255バイト制限対応）
- 重複ファイルの検出（サイズ + SHA256）
- junk ファイルの自動スキップ（.DS_Store, .Trash-*, lost+found 等）
- dry-run モード

```bash
# dry-run（実際には移動しない）
merge-dirs-for-linux-limit /media/hdd3 /data1/backups/dest --dry-run

# 実行
merge-dirs-for-linux-limit /media/hdd3 /data1/backups/dest
```

**出力例**:
```
[move] /src/file.txt -> /dst/file.txt
[delete-src] /src/dup.txt -> /dst/dup.txt (duplicate)
[skip] /src/.DS_Store

Summary: 100 moved, 5 duplicates, 10 skipped, 0 errors (115 total)
```

## インストール

```bash
# ~/.cargo/bin/ にインストール（推奨）
cargo install --path .

# システム全体にインストール
cargo build --release
sudo cp target/release/rename-for-linux-limit /usr/local/bin/
sudo cp target/release/merge-dirs-for-linux-limit /usr/local/bin/
```

## スキップされる junk ファイル

| OS | パターン |
|----|----------|
| macOS | .DS_Store, .Trashes, .Spotlight-V100, .fseventsd, ._* など |
| Windows | desktop.ini, Thumbs.db, $RECYCLE.BIN など |
| Linux | lost+found, .Trash-* |

## 設定ファイル（オプション）

`~/.config/rename-for-linux-limit/config.json` で以下を設定可能：

```json
{
  "ignored_tags": ["tag_to_remove"],
  "conversions": {
    "long_tag": "short"
  }
}
```

## 開発

```bash
cargo build           # ビルド
cargo test            # テスト
cargo clippy          # Lint
cargo fmt             # フォーマット
```
