fn main() {
    // .env.build から読み込む（Git管理外）
    if let Ok(content) = std::fs::read_to_string(".env.build") {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                // 環境変数が未設定の場合のみ .env.build の値を使う
                if std::env::var(key).is_err() {
                    println!("cargo:rustc-env={}={}", key.trim(), value.trim());
                }
            }
        }
    }

    // 環境変数から（CI/CD用、.env.build より優先）
    for key in &["DEFAULT_GOOGLE_CLIENT_ID", "DEFAULT_GOOGLE_CLIENT_SECRET", "DEFAULT_WORKER_URL"] {
        if let Ok(val) = std::env::var(key) {
            println!("cargo:rustc-env={}={}", key, val);
        }
        println!("cargo:rerun-if-env-changed={}", key);
    }
    println!("cargo:rerun-if-changed=.env.build");
}
