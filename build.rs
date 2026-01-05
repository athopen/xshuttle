use std::process::Command;

fn main() {
    set_version();
    set_build_hash();
}

fn set_version() {
    println!("cargo:rerun-if-env-changed=XSHUTTLE_VERSION");

    let version = std::env::var("XSHUTTLE_VERSION").unwrap_or_else(|_| "dev".to_string());
    println!("cargo:rustc-env=XSHUTTLE_VERSION={version}");
}

fn set_build_hash() {
    let hash = Command::new("git")
        .args(["rev-parse", "--short=10", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=XSHUTTLE_BUILD_HASH={hash}");
}
