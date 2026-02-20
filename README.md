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
| `--local-path` | - | - | ローカルディレクトリを監視（SMB マウントをスキップ） |

### 認証オプション

Worker 経由のユーザー名/パスワード認証か、Google OAuth 認証のいずれかを使用します。両方を同時に指定することはできません。

**ユーザー名/パスワード認証:**

| オプション | 環境変数 | 説明 |
|---|---|---|
| `--auth-user` | `SMB_WATCH_AUTH_USER` | Worker ログインユーザー名 |
| `--auth-pass` | `SMB_WATCH_AUTH_PASS` | Worker ログインパスワード |
| `--auth-url` | `SMB_WATCH_AUTH_URL` | Worker ログイン URL |

3 つとも指定するか、全て省略してください。

**Google OAuth 認証（デフォルト）:**

| オプション | 環境変数 | 説明 |
|---|---|---|
| `--google-client-id` | `GOOGLE_CLIENT_ID` | OAuth 2.0 Client ID |
| `--google-client-secret` | `GOOGLE_CLIENT_SECRET` | OAuth 2.0 Client Secret |
| `--google-auth-worker-url` | `SMB_WATCH_UPLOAD_WORKER_URL` | smb-upload-worker の URL |

認証オプションを省略すると Google OAuth Device Flow で認証します。ブラウザで Google アカウントにログインし、表示されたコードを入力してください。

### 組織選択

Google OAuth 認証時、ユーザーが複数の組織に所属している場合は対話的に組織を選択します。

| オプション | 環境変数 | 説明 |
|---|---|---|
| `--organization-id` | `ORGANIZATION_ID` | 組織 ID を直接指定（保存設定より優先） |

**組織 ID の解決順序:**
1. `--organization-id` / `ORGANIZATION_ID` 環境変数
2. `organization_config.json`（端末に保存された前回の選択）
3. サーバーから組織一覧を取得し、複数あれば対話的に選択
4. JWT 内のデフォルト組織（フォールバック）

選択結果は `organization_config.json` に保存され、次回以降は自動で使用されます。リセットするには `organization_config.json` を削除してください。

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
| `organization_config.json` | 選択した組織の設定（Google OAuth 時） |
| `google_token_cache.json` | Google OAuth トークンキャッシュ |

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
