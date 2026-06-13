//! `straymark loom serve` — download-on-demand launcher (Loom plan.md §5).
//!
//! 1. Resolve the project root.
//! 2. Ensure `~/.straymark/bin/straymark-loom` is present and current
//!    (download + extract the latest `loom-*` release asset otherwise).
//! 3. Print the EXPERIMENTAL banner and spawn the server pointed at the
//!    project; optionally open the browser.
//!
//! The CLI deliberately has no axum/tokio dependency — it only downloads and
//! launches (keeps the `opt-level=z` footprint intact).

use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::download::{self, strip_tag_prefix};
use crate::platform;
use crate::self_update::{extract_from_tar_gz, extract_from_zip};
use crate::utils;

const BINARY_NAME: &str = if cfg!(windows) {
    "straymark-loom.exe"
} else {
    "straymark-loom"
};

pub fn run(path: &str, port: u16, no_open: bool) -> Result<()> {
    banner();

    let project = PathBuf::from(path)
        .canonicalize()
        .with_context(|| format!("Cannot resolve project path {path}"))?;

    let binary = ensure_loom_binary()?;

    let url = format!("http://127.0.0.1:{port}");
    let mut child = Command::new(&binary)
        .arg(&project)
        .arg("--port")
        .arg(port.to_string())
        .spawn()
        .with_context(|| format!("Failed to launch {}", binary.display()))?;

    if !no_open {
        // Give the server a moment to bind before pointing a browser at it.
        std::thread::sleep(std::time::Duration::from_millis(600));
        // The server owns the terminal output; browser failure is not fatal.
        let _ = open_browser(&url);
    }

    let status = child.wait().context("Loom server exited abnormally")?;
    if !status.success() {
        bail!("straymark-loom exited with {status}");
    }
    Ok(())
}

fn banner() {
    eprintln!();
    eprintln!(
        "{}",
        "  ⚠  LOOM IS EXPERIMENTAL (v0)".yellow().bold()
    );
    eprintln!(
        "{}",
        "     Unstable: API, CLI surface, and on-disk layout may change or be\n     removed without a deprecation cycle. Loopback-only. Read-only."
            .yellow()
    );
    eprintln!();
}

/// `~/.straymark/bin/` — the per-user cache for downloaded components.
fn cache_bin_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Cannot determine the home directory (HOME/USERPROFILE unset)")?;
    Ok(PathBuf::from(home).join(".straymark").join("bin"))
}

/// Returns the cached Loom binary, downloading the latest `loom-*` release
/// when missing or stale. A version marker file next to the binary tracks
/// what is cached.
fn ensure_loom_binary() -> Result<PathBuf> {
    let bin_dir = cache_bin_dir()?;
    let exe = bin_dir.join(BINARY_NAME);
    let version_marker = bin_dir.join("straymark-loom.version");

    let release = match download::get_latest_release_by_prefix("loom-") {
        Ok(r) => r,
        Err(err) => {
            // Offline with a cached binary: serve what we have, loudly.
            if exe.exists() {
                utils::warn(&format!(
                    "Cannot check for Loom updates ({err}); using the cached binary"
                ));
                return Ok(exe);
            }
            return Err(err).context(
                "Loom is downloaded on demand and no release could be fetched. \
                 Check your network (or GITHUB_TOKEN for rate limits) and retry",
            );
        }
    };
    let version = strip_tag_prefix(&release.tag_name).to_string();

    let cached = std::fs::read_to_string(&version_marker)
        .map(|s| s.trim().to_string())
        .ok();
    if exe.exists() && cached.as_deref() == Some(version.as_str()) {
        return Ok(exe);
    }

    let target = platform::current_target()?;
    let ext = platform::archive_extension();
    let asset_name = format!("straymark-loom-v{version}-{target}.{ext}");
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .with_context(|| {
            format!(
                "Release {} has no asset {asset_name} for this platform",
                release.tag_name
            )
        })?;

    utils::info(&format!(
        "Downloading Loom {version} ({target}) — first use is opt-in by download"
    ));

    let temp = tempfile::tempdir().context("Failed to create temp dir")?;
    let archive_path = temp.path().join(&asset_name);
    download::download_file(&asset.download_url, &archive_path, "straymark-loom")?;

    let extracted = if ext == "zip" {
        extract_from_zip(&archive_path, temp.path(), BINARY_NAME)?
    } else {
        extract_from_tar_gz(&archive_path, temp.path(), BINARY_NAME)?
    };

    std::fs::create_dir_all(&bin_dir)
        .with_context(|| format!("Failed to create {}", bin_dir.display()))?;
    // Replace atomically-ish: copy to a temp name, then rename over.
    let staging = bin_dir.join(format!("{BINARY_NAME}.new"));
    std::fs::copy(&extracted, &staging).context("Failed to stage the Loom binary")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&staging, std::fs::Permissions::from_mode(0o755))?;
    }
    std::fs::rename(&staging, &exe).context("Failed to install the Loom binary")?;
    std::fs::write(&version_marker, &version)?;

    utils::success(&format!("Loom {version} cached at {}", exe.display()));
    Ok(exe)
}

fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = Command::new("open");
        c.arg(url);
        c
    };
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", "", url]);
        c
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let mut cmd = {
        let mut c = Command::new("xdg-open");
        c.arg(url);
        c
    };
    cmd.spawn().context("Failed to open the browser")?;
    Ok(())
}
