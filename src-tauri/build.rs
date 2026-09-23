use std::path::Path;
use std::process::Command;

fn main() {
    build_hook_client();
    tauri_build::build()
}

/// Opt-in, for type-checking the host against another OS: no sidecar can be produced
/// there, but `tauri_build::build()` still insists on resolving one.
const SKIP_SIDECAR: &str = "ADHD_RANCH_SKIP_SIDECAR";

/// Ranch copies the bundled client to a stable data-root path for Claude's plugin,
/// so Tauri has to pick it up as a sidecar — and it looks for one during
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
    println!("cargo:rerun-if-env-changed={SKIP_SIDECAR}");

    let target = std::env::var("TARGET").expect("cargo sets TARGET for build scripts");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("cargo sets CARGO_CFG_TARGET_OS");
    let suffix = if target_os == "windows" { ".exe" } else { "" };
    let out = Path::new(OUT_DIR).join(format!("adhd-ranch-hook-{target}{suffix}"));
    std::fs::create_dir_all(OUT_DIR).expect("create the sidecar directory");

    if skip_sidecar(&target_os) {
        write_placeholder(&out, &target);
        return;
    }

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let built = Command::new(rustc)
        .args(["--edition", "2021", "-O", "--target", &target, "-o"])
        .arg(&out)
        .arg(SOURCE)
        .status()
        .expect("run rustc for the hook client");
    assert!(built.success(), "hook client did not build for {target}");
}

/// Cross-OS is the hard gate, not a convenience: every bundle is built on a runner
/// whose OS matches its target, so a distributable always takes the rustc path below
/// no matter what the environment says.
fn skip_sidecar(target_os: &str) -> bool {
    std::env::var(SKIP_SIDECAR).is_ok_and(|v| v == "1") && target_os != std::env::consts::OS
}

/// Text rather than a stub executable: a hook client that silently does nothing is
/// what a working one looks like, so a placeholder has to be unable to run at all.
fn write_placeholder(out: &Path, target: &str) {
    println!("cargo:warning=placeholder sidecar for {target}, not a hook client");
    std::fs::write(
        out,
        format!("{SKIP_SIDECAR}=1 placeholder; not an executable\n"),
    )
    .expect("write the placeholder sidecar");
}
