/// rust-embed's `#[folder = "web/dist/"]` requires the directory to exist at
/// compile time, but it is gitignored (built by Vite, in CI for releases).
/// Ensure it exists so a fresh clone can `cargo build` without npm.
fn main() {
    let dist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("web/dist");
    std::fs::create_dir_all(&dist).expect("failed to create web/dist for rust-embed");
}
