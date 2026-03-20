fn main() {
    let backend_url = std::env::var("BACKEND_URL")
        .unwrap_or_else(|_| "https://overlay.soulreturns.com".to_string());
    let soul_app_token = std::env::var("SOUL_APP_TOKEN").unwrap_or_default();

    println!("cargo:rustc-env=BACKEND_URL={}", backend_url);
    println!("cargo:rustc-env=SOUL_APP_TOKEN={}", soul_app_token);
    println!("cargo:rerun-if-env-changed=BACKEND_URL");
    println!("cargo:rerun-if-env-changed=SOUL_APP_TOKEN");

    tauri_build::build()
}
