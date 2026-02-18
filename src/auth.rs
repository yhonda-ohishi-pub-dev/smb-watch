use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Serialize)]
struct LoginRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct GoogleLoginRequest<'a> {
    #[serde(rename = "idToken")]
    id_token: &'a str,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
    #[serde(rename = "expiresAt")]
    expires_at: String,
    #[serde(rename = "organizationId")]
    organization_id: String,
}

pub async fn login(
    client: &reqwest::Client,
    auth_url: &str,
    user: &str,
    pass: &str,
) -> Result<(String, String)> {
    let resp = client
        .post(auth_url)
        .json(&LoginRequest {
            username: user,
            password: pass,
        })
        .send()
        .await
        .context("Auth login request")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Auth login failed (HTTP {}): {}", status, body.trim());
    }

    let login_resp: LoginResponse = resp.json().await.context("Parsing login response")?;
    info!("Authenticated, token expires at {}", login_resp.expires_at);
    Ok((login_resp.token, login_resp.organization_id))
}

pub async fn login_with_google(
    client: &reqwest::Client,
    auth_url: &str,
    id_token: &str,
) -> Result<(String, String)> {
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

    let login_resp: LoginResponse = resp.json().await.context("Parsing Google auth response")?;
    info!("Authenticated via Google, token expires at {}", login_resp.expires_at);
    Ok((login_resp.token, login_resp.organization_id))
}
