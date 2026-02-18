# rust-logi JWT 認証追加計画

rust-logi に AuthService を追加し、JWT ベースの認証を実装する。

---

## 依存追加 (`Cargo.toml`)

- [ ] `jsonwebtoken = "9"`
- [ ] `argon2 = "0.5"`

---

## Proto 定義 (`packages/logi-proto/proto/auth.proto`)

- [ ] `auth.proto` 新規作成
  ```protobuf
  syntax = "proto3";
  package logi.auth;

  service AuthService {
    rpc Login(LoginRequest) returns (LoginResponse);
    rpc ValidateToken(ValidateTokenRequest) returns (ValidateTokenResponse);
  }

  message LoginRequest {
    string username = 1;
    string password = 2;
  }

  message LoginResponse {
    string token = 1;          // JWT
    string expires_at = 2;     // ISO 8601
  }

  message ValidateTokenRequest {
    string token = 1;
  }

  message ValidateTokenResponse {
    bool valid = 1;
    string organization_id = 2;
    string username = 3;
  }
  ```

---

## DB マイグレーション (`migrations/00020_create_api_users_and_tokens.sql`)

- [ ] `api_users` テーブル作成
  - `id`, `organization_id`, `username`, `password_hash`, `created_at`, `enabled`
- [ ] `api_tokens` テーブル作成
  - `id`, `user_id`, `token_hash` (SHA256), `expires_at`, `revoked`, `created_at`
- [ ] RLS 設定 (FORCE ROW LEVEL SECURITY)
- [ ] マイグレーション実行

---

## 新規ファイル

- [ ] `src/services/auth_service.rs` — AuthService 実装
  - `Login`: username/password 検証 → JWT 発行 → token_hash を DB 保存
  - `ValidateToken`: JWT 検証 + DB で revoke チェック
- [ ] `src/models/api_user.rs` — ApiUser, ApiToken モデル

---

## 既存ファイル変更

- [ ] `build.rs` — `auth.proto` を compile_protos に追加
- [ ] `src/lib.rs` — `proto::auth` モジュール追加
- [ ] `src/services/mod.rs` — `auth_service` モジュール・re-export 追加
- [ ] `src/models/mod.rs` — `api_user` モジュール追加
- [ ] `src/config.rs` — `jwt_secret: String` を Config に追加 (env: `JWT_SECRET`)
- [ ] `src/main.rs` — `AuthServiceServer` をサーバーに追加

---

## JWT 仕様

- Algorithm: HS256
- Claims: `{ sub: user_id, org: organization_id, username, exp, iat }`
- Secret: 環境変数 `JWT_SECRET`
- 有効期限: 24 時間
- サーバー側で `token_hash` を `api_tokens` に保存 → revoke 可能

---

## デプロイ・検証

- [ ] Cloud Run にデプロイ (`./deploy.sh`)
- [ ] `JWT_SECRET` 環境変数を設定
- [ ] テスト用 api_user を DB に INSERT
- [ ] `grpcurl` で `AuthService/Login` テスト
- [ ] `grpcurl` で `AuthService/ValidateToken` テスト
