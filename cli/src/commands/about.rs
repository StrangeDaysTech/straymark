use anyhow::Result;
use colored::Colorize;

use crate::manifest::DistManifest;
use crate::self_update::{self, InstallMethod};

pub fn run() -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let description = env!("CARGO_PKG_DESCRIPTION");
    let authors = env!("CARGO_PKG_AUTHORS");
    let license = env!("CARGO_PKG_LICENSE");
    let repository = env!("CARGO_PKG_REPOSITORY");
    let homepage = env!("CARGO_PKG_HOMEPAGE");

    println!();
    println!(
        "  {} {}",
        "StrayMark CLI".bold(),
        format!("cli-{version}").dimmed()
    );

    // Show framework version if installed
    if let Ok(cwd) = std::env::current_dir() {
        let manifest_path = cwd.join(".straymark/dist-manifest.yml");
        if let Ok(manifest) = DistManifest::load(&manifest_path) {
            println!(
                "  {} {}",
                "Framework:".dimmed(),
                format!("fw-{}", manifest.version).dimmed()
            );
        }
    }

    // Show install method
    let install_label = match self_update::detect_install_method() {
        InstallMethod::Cargo => "cargo (crates.io)",
        InstallMethod::GitHubBinary => "prebuilt binary (GitHub Releases)",
    };
    println!("  {} {}", "Install:".dimmed(), install_label.dimmed());

    println!("  {}", description.dimmed());
    println!();
    println!("  {}  {}", "Author:".cyan(), authors);
    println!("  {} {}", "License:".cyan(), license);
    println!("  {}    {}", "Repo:".cyan(), repository);
    println!("  {}     {}", "Web:".cyan(), homepage);
    println!();

    Ok(())
}
