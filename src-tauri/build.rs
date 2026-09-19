use std::path::Path;
use std::process::Command;

fn main() {
    build_hook_client();
    tauri_build::build()
}

/// Claude Code runs the hook client by absolute path, resolved beside the app's own
/// executable, so Tauri has to pick it up as a sidecar — and it looks for one during
/// this build script, not at bundle time. Building it here is what makes it exist for
/// every way the app gets compiled: `cargo build`, clippy, `tauri dev`, `tauri build`,
/// a fresh clone.
///
/// rustc rather than cargo because the crate is one file with no dependencies, and a
/// nested cargo would block on the build directory lock this script is already under.
fn build_hook_client() {
    const SOURCE: &str = "../crates/hook-client/src/main.rs";
    const OUT_DIR: &str = "binaries";

    println!("cargo:rerun-if-changed={SOURCE}");

    let target = std::env::var("TARGET").expect("cargo sets TARGET for build scripts");
    let suffix = if target.contains("windows") {
        ".exe"
    } else {
        ""
    };
    let out = Path::new(OUT_DIR).join(format!("adhd-ranch-hook-{target}{suffix}"));
    std::fs::create_dir_all(OUT_DIR).expect("create the sidecar directory");

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let built = Command::new(rustc)
        .args(["--edition", "2021", "-O", "--target", &target, "-o"])
        .arg(&out)
        .arg(SOURCE)
        .status()
        .expect("run rustc for the hook client");
    assert!(built.success(), "hook client did not build for {target}");
}
