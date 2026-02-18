use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

const GOOGLE_DEVICE_CODE_URL: &str = "https://oauth2.googleapis.com/device/code";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const DEVICE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

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
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Serialize)]
struct TokenPollRequest<'a> {
    client_id: &'a str,
    device_code: &'a str,
    grant_type: &'a str,
}

pub async fn device_flow_get_id_token(
    client: &reqwest::Client,
    client_id: &str,
) -> Result<String> {
    // Step 1: デバイスコードを取得
    let resp: DeviceCodeResponse = client
        .post(GOOGLE_DEVICE_CODE_URL)
        .form(&DeviceCodeRequest {
            client_id,
            scope: "openid email profile",
        })
        .send()
        .await
        .context("Device code request")?
        .json()
        .await
        .context("Parsing device code response")?;

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
                return token_resp
                    .id_token
                    .ok_or_else(|| anyhow::anyhow!("レスポンスに id_token がありません"));
            }
            Some("authorization_pending") => {
                // まだ待機中 - ループ継続
            }
            Some("slow_down") => {
                // バックオフが必要
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
