# smb-watch

SMB 共有フォルダを監視し、変更されたファイルを HTTP でアップロードする Windows 向けツール。

## 概要

起動のたびに SMB 共有フォルダをスキャンし、前回実行以降に変更されたファイルを検出して HTTP エンドポイントへアップロードします。アップロードに失敗したファイルは次回実行時に自動でリトライします。

## インストール

[GitHub Releases](https://github.com/yhonda-ohishi-pub-dev/smb-watch/releases) から最新の MSI インストーラーをダウンロードして実行してください。

`smb-watch.exe` が `C:\Program Files\smb-watch\` にインストールされます。

## 使い方

```powershell
smb-watch.exe `
  --smb-host 192.168.1.10 `
  --smb-share 共有 `
  --smb-path 新車検証 `
  --smb-user ユーザー名 `
  --smb-pass パスワード `
  --upload-url https://example.com
```

## オプション

| オプション | デフォルト値 | 環境変数 | 説明 |
|---|---|---|---|
| `--smb-host` | `172.18.21.102` | - | SMB サーバーのホスト名または IP |
| `--smb-share` | `共有` | - | SMB 共有名 |
| `--smb-path` | `新車検証` | - | 共有内の監視対象パス |
| `--smb-user` | - | `SMB_USER` | SMB 接続ユーザー名 |
| `--smb-pass` | - | `SMB_PASS` | SMB 接続パスワード |
| `--smb-domain` | `` | `SMB_DOMAIN` | SMB ドメイン名（省略可） |
| `--upload-url` | `https://nuxt-pwa-carins.mtamaramu.com` | `UPLOAD_URL` | アップロード先 URL |
| `--drive-letter` | `Z:` | - | SMB マウントに使用するドライブレター |
| `--dry-run` | `false` | - | アップロードを行わずに検出のみ実行 |
| `--since` | - | - | 指定した RFC3339 タイムスタンプ以降のファイルを対象にする |
| `--log-level` | `info` | - | ログレベル（trace / debug / info / warn / error） |

パスワードなどの機密情報は環境変数での指定を推奨します。

## 動作の流れ

1. `--drive-letter` に SMB 共有をマウント（`net use`）
2. 前回の実行記録（`last_run.txt`）から基準時刻を取得
3. 監視対象パスを再帰スキャンし、基準時刻以降に変更されたファイルを検出（パス昇順）
4. 前回失敗したファイル（`failed_files.txt`）と統合
5. `POST {upload-url}/api/recieve` へ multipart/form-data でアップロード
6. 失敗したファイルを `failed_files.txt` に保存（次回リトライ）
7. SMB アンマウント

## 状態ファイル

| ファイル | 説明 |
|---|---|
| `last_run.txt` | 実行履歴。次回スキャンの基準時刻として使用される |
| `failed_files.txt` | アップロードに失敗したファイルの一覧 |

## 要件

- Windows x64
- SMB 共有への読み取りアクセス権

## ビルド

```powershell
# リリースビルド
cargo build --release --target x86_64-pc-windows-msvc

# MSI インストーラービルド（WiX v3.11 が必要）
cargo wix --target x86_64-pc-windows-msvc
```
