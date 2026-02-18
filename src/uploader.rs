use anyhow::{Context, Result};
use reqwest::multipart;
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;
use tracing::{info, warn};

#[derive(Deserialize, Debug)]
pub struct UploadResponse {
    pub uuid: String,
    pub message: String,
}

pub fn build_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .context("Building HTTP client")
}

pub async fn upload_file(
    client: &reqwest::Client,
    url: &str,
    path: &Path,
    token: Option<&str>,
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

    let data_part = multipart::Part::bytes(bytes)
        .file_name(filename.clone())
        .mime_str(&mime)
        .with_context(|| format!("Setting MIME type: {}", mime))?;

    let form = multipart::Form::new()
        .part("data", data_part)
        .text("from", "front");

    let mut request = client.post(url).multipart(form);
    if let Some(t) = token {
        request = request.bearer_auth(t);
    }

    let response = request
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
            info!(
                "Uploaded {} -> uuid: {}, message: {}",
                filename, resp.uuid, resp.message
            );
        }
        Err(e) => {
            warn!("Uploaded {} but could not parse response: {}", filename, e);
        }
    }

    Ok(())
}
