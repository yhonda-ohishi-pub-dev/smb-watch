use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

#[derive(Serialize)]
struct GoogleLoginRequest<'a> {
    id_token: &'a str,
}

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
    expires_in: i64,
    user: UserResponse,
}

#[derive(Deserialize)]
struct UserResponse {
    tenant_id: Uuid,
}

/// rust-alc-api の POST /api/auth/google で認証
pub async fn login_with_google(
    client: &reqwest::Client,
    auth_url: &str,
    id_token: &str,
) -> Result<(String, Uuid)> {
    let resp = client
        .post(auth_url)
        .json(&GoogleLoginRequest { id_token })
        .send()
        .await
        .context("Google auth request")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Google auth failed (HTTP {}): {}", status, body.trim());
    }

    let auth_resp: AuthResponse = resp.json().await.context("Parsing auth response")?;
    info!(
        "Authenticated via Google, tenant_id={}, expires_in={}s",
        auth_resp.user.tenant_id, auth_resp.expires_in
    );
    Ok((auth_resp.access_token, auth_resp.user.tenant_id))
}
