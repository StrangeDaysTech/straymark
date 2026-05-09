use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Read;
use std::path::Path;

const GITHUB_REPO: &str = "StrangeDaysTech/straymark";
const GITHUB_API_BASE: &str = "https://api.github.com";

/// Information about a GitHub release (distribution ZIP)
#[derive(Debug)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub zip_url: String,
}

/// A single asset in a GitHub release
#[derive(Debug)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
}

/// Full release information including all assets
#[derive(Debug)]
pub struct FullReleaseInfo {
    pub tag_name: String,
    pub assets: Vec<ReleaseAsset>,
}

/// Build an HTTP client with optional GitHub token authentication
fn build_client() -> Result<reqwest::blocking::Client> {
    let mut builder = reqwest::blocking::Client::builder().user_agent("straymark-cli");

    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        use reqwest::header;
        let mut headers = header::HeaderMap::new();
        let value = header::HeaderValue::from_str(&format!("Bearer {}", token))
            .context("Invalid GITHUB_TOKEN value")?;
        headers.insert(header::AUTHORIZATION, value);
        builder = builder.default_headers(headers);
    }

    builder.build().context("Failed to create HTTP client")
}

/// Strip known tag prefixes (fw-, cli-, v) and return the version string
pub fn strip_tag_prefix(tag: &str) -> &str {
    tag.strip_prefix("fw-")
        .or_else(|| tag.strip_prefix("cli-"))
        .or_else(|| tag.strip_prefix("v"))
        .unwrap_or(tag)
}

/// Fetch the latest release whose tag starts with the given prefix
pub fn get_latest_release_by_prefix(prefix: &str) -> Result<FullReleaseInfo> {
    let url = format!("{}/repos/{}/releases", GITHUB_API_BASE, GITHUB_REPO);

    let client = build_client()?;

    let response = client
        .get(&url)
        .send()
        .context("Failed to fetch releases from GitHub")?;

    if response.status() == reqwest::StatusCode::FORBIDDEN {
        bail!(
            "GitHub API rate limit exceeded. Set GITHUB_TOKEN environment variable to increase limits."
        );
    }

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        bail!("No releases found for {}", GITHUB_REPO);
    }

    if !response.status().is_success() {
        bail!("GitHub API error: {}", response.status());
    }

    let body: serde_json::Value = response.json().context("Failed to parse releases JSON")?;

    let releases = body.as_array().context("Expected JSON array of releases")?;

    for release in releases {
        let tag_name = match release["tag_name"].as_str() {
            Some(t) => t,
            None => continue,
        };

        if !tag_name.starts_with(prefix) {
            continue;
        }

        let tag_name = tag_name.to_string();
        let mut assets = Vec::new();

        if let Some(asset_array) = release["assets"].as_array() {
            for asset in asset_array {
                if let (Some(name), Some(url)) = (
                    asset["name"].as_str(),
                    asset["browser_download_url"].as_str(),
                ) {
                    assets.push(ReleaseAsset {
                        name: name.to_string(),
                        download_url: url.to_string(),
                    });
                }
            }
        }

        return Ok(FullReleaseInfo { tag_name, assets });
    }

    bail!("No release found with tag prefix '{}'", prefix)
}

/// Get full release info for CLI releases (cli-* tags)
pub fn get_latest_release_full() -> Result<FullReleaseInfo> {
    get_latest_release_by_prefix("cli-")
}

/// Get the latest framework release info (fw-* tags)
pub fn get_latest_release() -> Result<ReleaseInfo> {
    let full = get_latest_release_by_prefix("fw-")?;

    // Look for the straymark distribution ZIP (straymark-fw-*.zip)
    let zip_url = full
        .assets
        .iter()
        .find(|a| a.name.starts_with("straymark-fw-") && a.name.ends_with(".zip"))
        .map(|a| a.download_url.clone());

    // Fallback to zipball if no distribution asset found
    let zip_url = zip_url.unwrap_or_else(|| {
        format!(
            "https://github.com/{}/archive/refs/tags/{}.zip",
            GITHUB_REPO, full.tag_name
        )
    });

    Ok(ReleaseInfo {
        tag_name: full.tag_name,
        zip_url,
    })
}

/// Download a file from a URL to a destination path with progress bar
pub fn download_file(url: &str, dest: &Path, label: &str) -> Result<()> {
    let client = build_client()?;

    let mut response = client
        .get(url)
        .send()
        .with_context(|| format!("Failed to download {}", label))?;

    if !response.status().is_success() {
        bail!("Download failed with status: {}", response.status());
    }

    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut file = std::fs::File::create(dest).context("Failed to create temporary file")?;
    let mut buffer = [0u8; 8192];
    let mut downloaded: u64 = 0;

    loop {
        let bytes_read = response.read(&mut buffer).context("Failed to read response")?;
        if bytes_read == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buffer[..bytes_read])
            .context("Failed to write to file")?;
        downloaded += bytes_read as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Download complete");
    Ok(())
}

/// Download a ZIP file from a URL to a temporary file
pub fn download_zip(url: &str, dest: &Path) -> Result<()> {
    download_file(url, dest, "release ZIP")
}
