use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

const GOOGLE_DEVICE_CODE_URL: &str = "https://oauth2.googleapis.com/device/code";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const DEVICE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";
const REFRESH_GRANT_TYPE: &str = "refresh_token";
const TOKEN_CACHE_FILE: &str = "google_token_cache.json";

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_url: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Serialize)]
struct DeviceCodeRequest<'a> {
    client_id: &'a str,
    scope: &'a str,
}

#[derive(Deserialize)]
struct TokenResponse {
    id_token: Option<String>,
    refresh_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Serialize)]
struct TokenPollRequest<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    device_code: &'a str,
    grant_type: &'a str,
}

#[derive(Serialize)]
struct RefreshRequest<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    refresh_token: &'a str,
    grant_type: &'a str,
}

#[derive(Serialize, Deserialize)]
struct TokenCache {
    id_token: String,
    refresh_token: String,
    /// Unix timestamp (seconds) when id_token expires
    expires_at: i64,
}

impl TokenCache {
    fn load() -> Option<Self> {
        let data = std::fs::read_to_string(TOKEN_CACHE_FILE).ok()?;
        serde_json::from_str(&data).ok()
    }

    fn save(&self) {
        if let Ok(data) = serde_json::to_string(self) {
            let _ = std::fs::write(TOKEN_CACHE_FILE, data);
        }
    }

    fn is_valid(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        // 5分の余裕を持たせる
        self.expires_at > now + 300
    }
}

fn extract_exp(id_token: &str) -> Option<i64> {
    let payload = id_token.split('.').nth(1)?;
    // base64url decode (no padding)
    let padded = match payload.len() % 4 {
        2 => format!("{}==", payload),
        3 => format!("{}=", payload),
        _ => payload.to_string(),
    };
    let decoded = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE,
        padded,
    )
    .ok()?;
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    json["exp"].as_i64()
}

pub async fn device_flow_get_id_token(
    client: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
) -> Result<String> {
    // キャッシュ確認
    if let Some(cache) = TokenCache::load() {
        if cache.is_valid() {
            info!("キャッシュされた Google トークンを使用します");
            return Ok(cache.id_token);
        }
        // id_token 期限切れ → refresh_token で更新
        info!("Google トークンをリフレッシュします...");
        match refresh_id_token(client, client_id, client_secret, &cache.refresh_token).await {
            Ok(new_id_token) => {
                let expires_at = extract_exp(&new_id_token).unwrap_or(0);
                TokenCache {
                    id_token: new_id_token.clone(),
                    refresh_token: cache.refresh_token,
                    expires_at,
                }
                .save();
                info!("Google トークンのリフレッシュが完了しました");
                return Ok(new_id_token);
            }
            Err(e) => {
                info!("リフレッシュ失敗、再認証します: {:#}", e);
            }
        }
    }

    // Device Flow で新規認証
    let id_token = do_device_flow(client, client_id, client_secret).await?;
    Ok(id_token)
}

async fn refresh_id_token(
    client: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<String> {
    let resp: TokenResponse = client
        .post(GOOGLE_TOKEN_URL)
        .form(&RefreshRequest {
            client_id,
            client_secret,
            refresh_token,
            grant_type: REFRESH_GRANT_TYPE,
        })
        .send()
        .await
        .context("Refresh token request")?
        .json()
        .await
        .context("Parsing refresh token response")?;

    if let Some(err) = resp.error {
        anyhow::bail!("リフレッシュエラー: {} - {}", err, resp.error_description.as_deref().unwrap_or(""));
    }

    resp.id_token.ok_or_else(|| anyhow::anyhow!("リフレッシュレスポンスに id_token がありません"))
}

async fn do_device_flow(
    client: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
) -> Result<String> {
    // Step 1: デバイスコードを取得
    let raw = client
        .post(GOOGLE_DEVICE_CODE_URL)
        .form(&DeviceCodeRequest {
            client_id,
            scope: "openid email profile",
        })
        .send()
        .await
        .context("Device code request")?
        .text()
        .await
        .context("Reading device code response")?;
    tracing::debug!("Device code response: {}", raw);
    let resp: DeviceCodeResponse =
        serde_json::from_str(&raw).with_context(|| format!("Parsing device code response: {}", raw))?;

    // Step 2: ユーザーに認証を促す
    println!();
    println!("=== Google 認証が必要です ===");
    println!("ブラウザで以下の URL を開いてください:");
    println!("  {}", resp.verification_url);
    println!("コードを入力してください: {}", resp.user_code);
    println!("============================");
    println!();
    info!("Google 認証を待っています...");

    // Step 3: トークンをポーリング
    let interval = Duration::from_secs(resp.interval.max(5));
    let deadline = std::time::Instant::now() + Duration::from_secs(resp.expires_in);

    loop {
        if std::time::Instant::now() > deadline {
            anyhow::bail!("Google 認証がタイムアウトしました。smb-watch を再起動してください。");
        }

        sleep(interval).await;

        let token_resp: TokenResponse = client
            .post(GOOGLE_TOKEN_URL)
            .form(&TokenPollRequest {
                client_id,
                client_secret,
                device_code: &resp.device_code,
                grant_type: DEVICE_GRANT_TYPE,
            })
            .send()
            .await
            .context("Token poll request")?
            .json()
            .await
            .context("Parsing token poll response")?;

        match token_resp.error.as_deref() {
            None => {
                info!("Google 認証が完了しました");
                let id_token = token_resp
                    .id_token
                    .ok_or_else(|| anyhow::anyhow!("レスポンスに id_token がありません"))?;
                let refresh_token = token_resp.refresh_token.unwrap_or_default();
                let expires_at = extract_exp(&id_token).unwrap_or(0);
                TokenCache {
                    id_token: id_token.clone(),
                    refresh_token,
                    expires_at,
                }
                .save();
                return Ok(id_token);
            }
            Some("authorization_pending") => {
                // まだ待機中 - ループ継続
            }
            Some("slow_down") => {
                sleep(Duration::from_secs(5)).await;
            }
            Some(err) => {
                anyhow::bail!(
                    "Device flow エラー: {} - {}",
                    err,
                    token_resp.error_description.as_deref().unwrap_or("")
                );
            }
        }
    }
}
