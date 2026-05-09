use anyhow::{bail, Result};

/// Returns the target triple for the current platform
pub fn current_target() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-gnu"),
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("windows", "x86_64") => Ok("x86_64-pc-windows-msvc"),
        (os, arch) => bail!("Unsupported platform: {}/{}", os, arch),
    }
}

/// Returns the archive extension for the current platform
pub fn archive_extension() -> &'static str {
    if cfg!(windows) {
        "zip"
    } else {
        "tar.gz"
    }
}

/// Generates the expected asset filename for a given version
pub fn asset_name(version: &str) -> Result<String> {
    let target = current_target()?;
    let ext = archive_extension();
    Ok(format!("straymark-cli-v{}-{}.{}", version, target, ext))
}
