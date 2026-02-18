use chrono::{DateTime, Utc};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "smb-watch", about = "Monitor SMB share and upload changed files via HTTP")]
pub struct Config {
    /// SMB server hostname or IP
    #[arg(long, default_value = "172.18.21.102")]
    pub smb_host: String,

    /// SMB share name
    #[arg(long, default_value = "共有")]
    pub smb_share: String,

    /// Subdirectory within the SMB share
    #[arg(long, default_value = "新車検証")]
    pub smb_path: String,

    /// SMB username (required for SMB mode, ignored in local mode)
    #[arg(long, env = "SMB_USER")]
    pub smb_user: Option<String>,

    /// SMB password (required for SMB mode, ignored in local mode)
    #[arg(long, env = "SMB_PASS", hide_env_values = true)]
    pub smb_pass: Option<String>,

    /// SMB domain (optional)
    #[arg(long, env = "SMB_DOMAIN", default_value = "")]
    pub smb_domain: String,

    /// HTTP upload base URL
    #[arg(long, env = "UPLOAD_URL", default_value = "https://nuxt-pwa-carins.mtamaramu.com")]
    pub upload_url: String,

    /// Path to state file storing last run timestamp
    #[arg(long, default_value = "last_run.txt")]
    pub state_file: std::path::PathBuf,

    /// Windows drive letter to use for net use mount
    #[arg(long, default_value = "Z:")]
    pub drive_letter: String,

    /// Scan files but do not upload (dry run)
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Log level: error, warn, info, debug, trace
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// スキャン基準時刻を上書き。RFC3339形式 (例: 2026-02-10T00:00:00Z)。
    /// 指定すると last_run.txt より優先される。
    #[arg(long, value_name = "DATETIME", value_parser = parse_since)]
    pub since: Option<DateTime<Utc>>,

    /// Auth username (Worker login)
    #[arg(long, env = "SMB_WATCH_AUTH_USER")]
    pub auth_user: Option<String>,

    /// Auth password (Worker login)
    #[arg(long, env = "SMB_WATCH_AUTH_PASS", hide_env_values = true)]
    pub auth_pass: Option<String>,

    /// Auth login URL (e.g. https://smb-upload-worker.xxx.workers.dev/auth/login)
    #[arg(long, env = "SMB_WATCH_AUTH_URL")]
    pub auth_url: Option<String>,

    /// Local directory path to monitor (enables local mode, skips SMB mount)
    #[arg(long, value_name = "PATH")]
    pub local_path: Option<std::path::PathBuf>,
}

fn parse_since(s: &str) -> std::result::Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| format!("Invalid RFC3339 datetime '{}': {}", s, e))
}
