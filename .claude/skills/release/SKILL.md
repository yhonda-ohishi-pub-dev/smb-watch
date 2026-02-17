---
name: release
description: smb-watch のリリースを行う。cargo release でバージョンを bump し、タグを push して GitHub Actions で MSI をビルドする。
argument-hint: "patch|minor|major|x.y.z"
---

smb-watch のリリースを行います。引数: `$ARGUMENTS`

## 手順

1. **現在の状態を確認する**（Bash / Read ツールを使うこと）
   - `Cargo.toml` を読んで現在の version を確認
   - `git status` で未コミットの変更がないか確認
   - 未コミットの変更がある場合はユーザーに警告して中断する

2. **引数の解釈**
   - 引数なし → `patch` として扱う
   - `patch` / `minor` / `major` → そのまま使用
   - `x.y.z` 形式 → バージョン直接指定

3. **ドライランを実行して変更内容を表示**
   ```
   cargo release <level>
   ```
   ドライランの出力をユーザーに見せて、続行するか確認する。

4. **ユーザーの確認を得てから実行**
   ```
   cargo release <level> --execute
   ```

5. **完了後に確認**
   - `git tag --sort=-version:refname` で新しいタグが作成されたか確認
   - GitHub Actions の URL を案内する（リポジトリの Actions タブ）

## 注意事項

- `release.toml` に `publish = false` が設定済みのため crates.io への publish は行われない
- タグ push 後、GitHub Actions が自動で MSI をビルドして GitHub Release を公開する
- UpgradeCode (`D802E510-9F08-408B-BFFD-B0B491E7F908`) は変更禁止
- MSI の ProductVersion は `Cargo.toml` の version から自動同期される
