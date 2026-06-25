use std::process::Command;
use std::time::SystemTime;

fn main() {
    // Get short Git commit hash
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let git_hash = git_hash.trim();
    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_hash);

    // Get current build timestamp in ISO 8601 / RFC 3339 format
    let build_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_time);
}
