use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tracing::{info, warn};

#[derive(Serialize)]
struct CreateFileRequest {
    filename: String,
    #[serde(rename = "type")]
    file_type: String,
    content: String, // base64 encoded
}

#[derive(Deserialize, Debug)]
pub struct UploadResponse {
    pub uuid: String,
}

pub fn build_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .context("Building HTTP client")
}

pub async fn upload_file(
    client: &reqwest::Client,
    url: &str,
    path: &Path,
    token: &str,
) -> Result<()> {
    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("Reading file {}", path.display()))?;

    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());

    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    let body = CreateFileRequest {
        filename: filename.clone(),
        file_type: mime,
        content: STANDARD.encode(&bytes),
    };

    let response = client
        .post(url)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .with_context(|| format!("POST to {}", url))?;

    let status = response.status();

    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "(unreadable body)".to_string());
        return Err(anyhow::anyhow!(
            "Upload failed with HTTP {}: {}",
            status,
            body.trim()
        ));
    }

    match response.json::<UploadResponse>().await {
        Ok(resp) => {
            info!("Uploaded {} -> uuid: {}", filename, resp.uuid);
        }
        Err(e) => {
            warn!("Uploaded {} but could not parse response: {}", filename, e);
        }
    }

    Ok(())
}
