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

    /// SMB username
    #[arg(long, env = "SMB_USER")]
    pub smb_user: String,

    /// SMB password
    #[arg(long, env = "SMB_PASS", hide_env_values = true)]
    pub smb_pass: String,

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
}
