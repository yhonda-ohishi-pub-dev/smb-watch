mod auth;
mod cli;
mod google_auth;
mod scanner;
mod smb;
mod state;
mod uploader;

use anyhow::Result;
use clap::Parser;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    let config = cli::Config::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new(&config.log_level)
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let scan_start = SystemTime::now();

    if let Some(local_path) = &config.local_path {
        info!("Local mode: monitoring {}", local_path.display());
        run(&config, local_path, scan_start).await
    } else {
        if config.smb_user.is_none() || config.smb_pass.is_none() {
            anyhow::bail!(
                "--smb-user and --smb-pass (or SMB_USER/SMB_PASS env vars) are required for SMB mode. \
                 Use --local-path for local mode."
            );
        }
        let mount = smb::SmbMount::mount(&config)?;
        let scan_path = PathBuf::from(format!("{}\\{}", mount.drive_letter, config.smb_path));
        let result = run(&config, &scan_path, scan_start).await;

        if let Err(e) = mount.unmount() {
            warn!("Failed to unmount SMB share: {:#}", e);
        }

        result
    }
}

async fn run(config: &cli::Config, scan_root: &std::path::Path, scan_start: SystemTime) -> Result<()> {
    let failed_list_path = state::failed_list_path(&config.state_file);

    // 1. Load previously failed files (retry candidates)
    let mut retry_candidates = state::load_failed_list(&failed_list_path)?;
    if !retry_candidates.is_empty() {
        info!("{} file(s) pending retry from previous run", retry_candidates.len());
        // Remove retry candidates that no longer exist on the share
        retry_candidates.retain(|p| p.exists());
    }

    // 2. Resolve "since" threshold: --since flag takes precedence over last_run.txt
    let since: SystemTime = if let Some(dt) = config.since {
        info!("Using --since override: {}", dt.to_rfc3339());
        SystemTime::from(dt)
    } else {
        state::read_last_run(&config.state_file)?
    };
    info!("Scanning: {}", scan_root.display());

    let changed_files = scanner::find_changed_files(scan_root, since)?;

    // 3. Merge: changed files + retries, deduplicated
    let retry_set: HashSet<PathBuf> = retry_candidates.into_iter().collect();
    let mut all_files: Vec<PathBuf> = changed_files;
    for p in &retry_set {
        if !all_files.contains(p) {
            info!("Adding retry: {}", p.display());
            all_files.push(p.clone());
        }
    }

    let files_found = all_files.len();
    info!("Found {} file(s) to process ({} new/changed + {} retries)",
        files_found,
        files_found - retry_set.len().min(files_found),
        retry_set.len().min(files_found),
    );

    let mut uploaded = 0usize;
    let mut new_failed: Vec<PathBuf> = Vec::new();

    if files_found == 0 {
        info!("No files to process");
    } else if config.dry_run {
        info!("Dry run mode: skipping uploads");
        for path in &all_files {
            info!("  Would upload: {}", path.display());
        }
    } else {
        let client = uploader::build_client()?;

        // Authenticate if auth options are provided
        let (token, org_id) = match (&config.auth_user, &config.auth_pass, &config.auth_url) {
            // 既存のパスワード認証モード
            (Some(user), Some(pass), Some(url)) => {
                let (t, id) = auth::login(&client, url, user, pass).await?;
                (Some(t), Some(id))
            }
            // 認証なし → Google OAuth
            (None, None, None) => {
                let id_token = google_auth::device_flow_get_id_token(&client, &config.google_client_id, &config.google_client_secret).await?;
                let google_auth_url = format!("{}/auth/google", config.google_auth_worker_url.trim_end_matches('/'));
                let (t, id) = auth::login_with_google(&client, &google_auth_url, &id_token).await?;
                (Some(t), Some(id))
            }
            _ => anyhow::bail!(
                "--auth-user, --auth-pass, --auth-url must all be specified together"
            ),
        };

        let upload_url = match (&token, &config.auth_url) {
            (Some(_), None) => {
                // Google OAuth モード: worker の /upload へ
                format!("{}/upload", config.google_auth_worker_url.trim_end_matches('/'))
            }
            (Some(_), Some(_)) => {
                // パスワード認証モード: upload_url の /upload へ
                format!("{}/upload", config.upload_url.trim_end_matches('/'))
            }
            _ => {
                format!("{}/api/recieve", config.upload_url.trim_end_matches('/'))
            }
        };

        for (i, path) in all_files.iter().enumerate() {
            info!("Uploading {}/{}: {}", i + 1, files_found, path.display());
            match uploader::upload_file(&client, &upload_url, path, token.as_deref(), org_id.as_deref()).await {
                Ok(()) => uploaded += 1,
                Err(e) => {
                    warn!("Failed: {}: {:#}", path.display(), e);
                    new_failed.push(path.clone());
                }
            }
        }

        if !new_failed.is_empty() {
            warn!("{} file(s) failed; will retry next run", new_failed.len());
        }
    }

    let failed_count = new_failed.len();

    // 4. Save updated failed list (empty = delete the file)
    state::save_failed_list(&failed_list_path, &new_failed)?;

    // 5. Always advance last_run to scan_start so new files aren't missed.
    //    Failed files are tracked separately in failed_files.txt.
    state::append_run_record(
        &config.state_file,
        &state::RunRecord {
            start: scan_start,
            end: SystemTime::now(),
            files_found,
            uploaded,
            failed: failed_count,
            dry_run: config.dry_run,
        },
    )?;

    Ok(())
}
