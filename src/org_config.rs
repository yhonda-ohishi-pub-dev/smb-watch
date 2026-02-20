use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use tracing::info;

const ORG_CONFIG_FILE: &str = "organization_config.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrgInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub role: String,
}

#[derive(Serialize, Deserialize)]
pub struct OrgConfig {
    pub selected_organization_id: String,
    pub selected_organization_name: String,
    pub organizations: Vec<OrgInfo>,
    pub updated_at: String,
}

impl OrgConfig {
    pub fn load() -> Option<Self> {
        let data = std::fs::read_to_string(ORG_CONFIG_FILE).ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn save(&self) {
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(ORG_CONFIG_FILE, data);
        }
    }
}

/// Worker の /organizations エンドポイントからユーザーの組織一覧を取得
pub async fn fetch_organizations(
    client: &reqwest::Client,
    worker_url: &str,
    token: &str,
) -> Result<Vec<OrgInfo>> {
    let url = format!("{}/organizations", worker_url.trim_end_matches('/'));

    let resp = client
        .post(&url)
        .bearer_auth(token)
        .send()
        .await
        .context("Failed to fetch organizations")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Failed to fetch organizations (HTTP {}): {}", status, body.trim());
    }

    #[derive(Deserialize)]
    struct OrgsResponse {
        organizations: Vec<OrgInfo>,
    }

    let orgs_resp: OrgsResponse = resp
        .json()
        .await
        .context("Parsing organizations response")?;

    Ok(orgs_resp.organizations)
}

/// 対話的に組織を選択
fn prompt_select_organization(orgs: &[OrgInfo]) -> Result<usize> {
    println!();
    println!("=== 組織を選択してください ===");
    for (i, org) in orgs.iter().enumerate() {
        println!("  [{}] {} ({}) - role: {}", i + 1, org.name, org.slug, org.role);
    }
    println!("==============================");
    print!("番号を入力: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let index: usize = input
        .trim()
        .parse::<usize>()
        .context("数値を入力してください")?
        .checked_sub(1)
        .context("1以上の番号を入力してください")?;

    if index >= orgs.len() {
        anyhow::bail!("無効な番号です（1〜{}）", orgs.len());
    }

    Ok(index)
}

/// 組織IDを解決する
///
/// 優先順位:
/// 1. --organization-id CLI / ORGANIZATION_ID env var
/// 2. organization_config.json（端末保存）
/// 3. サーバーから組織一覧取得 → 複数なら対話的選択
/// 4. JWT内のデフォルト組織
pub async fn resolve_organization(
    client: &reqwest::Client,
    cli_org_id: Option<&str>,
    worker_url: &str,
    token: &str,
    jwt_org_id: &str,
) -> Result<String> {
    // 1. CLI override
    if let Some(org_id) = cli_org_id {
        info!("CLI/env 指定の組織を使用: {}", org_id);
        return Ok(org_id.to_string());
    }

    // 2. 端末保存の設定
    if let Some(saved) = OrgConfig::load() {
        info!(
            "保存済みの組織を使用: {} ({})",
            saved.selected_organization_name, saved.selected_organization_id
        );
        return Ok(saved.selected_organization_id);
    }

    // 3. サーバーから取得して選択
    let orgs = match fetch_organizations(client, worker_url, token).await {
        Ok(orgs) => orgs,
        Err(e) => {
            info!("組織一覧の取得に失敗、JWTデフォルトを使用: {:#}", e);
            return Ok(jwt_org_id.to_string());
        }
    };

    let (selected_id, selected_name) = if orgs.is_empty() {
        // 組織なし → JWTデフォルト
        (jwt_org_id.to_string(), String::new())
    } else if orgs.len() == 1 {
        // 単一組織 → 自動選択
        let org = &orgs[0];
        info!("組織が1つのみ: {} ({})", org.name, org.id);
        (org.id.clone(), org.name.clone())
    } else {
        // 複数組織 → 対話的選択
        let idx = prompt_select_organization(&orgs)?;
        let org = &orgs[idx];
        info!("組織を選択しました: {} ({})", org.name, org.id);
        (org.id.clone(), org.name.clone())
    };

    // 選択結果を保存
    let now = chrono::Utc::now().to_rfc3339();
    OrgConfig {
        selected_organization_id: selected_id.clone(),
        selected_organization_name: selected_name,
        organizations: orgs,
        updated_at: now,
    }
    .save();

    Ok(selected_id)
}
