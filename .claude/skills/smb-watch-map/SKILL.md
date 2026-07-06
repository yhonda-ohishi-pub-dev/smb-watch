---
name: smb-watch-map
generated-from: smb-watch:a521a51619730760f2d9b79c1d406274762b8ff4
paths: [src/, deploy/]
description: ohishi-exp/smb-watch (SMB 共有フォルダ監視 → HTTP アップロードツール、Windows/Linux 両対応) の構造ナビゲーション。SMB アクセス方式 (Windows net use / Linux pure-Rust smb2 crate)・device-token 認証 (auth-worker pairing)・Linux 自動デプロイ (musl + systemd oneshot/timer/path + Cloudflare Tunnel SSH)・ohishi-data 実機運用メモをまとめる。トリガー:「smb-watch」「SMB 監視」「smb2 crate」「device pairing」「smb-watch pair」「ohishi-data」「systemd timer OnCalendar」「musl deploy」「smb-watch デプロイ」「failed_files.txt」等。
---

# smb-watch-map — 構造ナビゲーション

## 区画

| グループ | 主要ファイル | 役割 |
|---|---|---|
| ソース抽象化 | `src/source.rs` | `FileSource` interface (scanner/uploader 共通) |
| Windows SMB | `src/smb.rs` (`#[cfg(windows)]`) | `net use` マウント → ローカル FS 走査 |
| Linux SMB | `src/smb_fs.rs` (`#[cfg(not(windows))]`) | pure-Rust `smb2` crate 直アクセス |
| バージョン焼き込み | `build.rs` | `BUILD_SHA` / `BUILD_TIME` |
| デプロイ | `deploy/`, `scripts/deploy-remote.sh` | systemd unit テンプレ・SSH 配置スクリプト |
| インストーラ | `wix/main.wxs` | Windows MSI (WiX v3.11) |

## entrypoint

- CLI: `--smb-host` / `--smb-share` / `--smb-path` / `--local-path` / `--dry-run`
- 認証: `smb-watch pair` (device pairing, subcommand) / 通常 run (device-token → carins upload)
- example probe: `cargo run --example smb_probe -- ...` (実機疎通確認)

## CCoW/CI から見た立ち位置

- CI: `.github/workflows/ci.yml` (test / musl build → SSH deploy)、`release.yml` (`v*.*.*` tag → Windows MSI + Linux musl binary)
- deploy 先: ohishi-data (LAN 内 Linux host)、Cloudflare Tunnel SSH 経由。CCoW コンテナからは SSH 不可 (operator が実機で手動操作)

## 関連 skill

- `repo-map` — この map 自体の作成/更新手順
- `alcoholchecker-deploy` — 同種の OTA/deploy パターン (対照参考)
