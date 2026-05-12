#!/bin/sh
# StrayMark CLI installer — https://github.com/StrangeDaysTech/straymark
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.sh | sh
#   curl -fsSL ... | sh -s -- --tag cli-1.0.0 --to ~/.local/bin
#
# Compatible with bash, zsh, dash, and POSIX sh.

set -eu

REPO="StrangeDaysTech/straymark"
BINARY="straymark"

# ── Helpers ──────────────────────────────────────────────────────────────

say() {
    printf 'straymark-install: %s\n' "$*"
}

err() {
    say "ERROR: $*" >&2
    exit 1
}

need() {
    if ! command -v "$1" >/dev/null 2>&1; then
        err "$1 is required but not found in PATH"
    fi
}

download() {
    _url="$1"
    _out="$2"
    _auth_header=""

    if [ -n "${GITHUB_TOKEN:-}" ]; then
        _auth_header="Authorization: token ${GITHUB_TOKEN}"
    fi

    if command -v curl >/dev/null 2>&1; then
        if [ -n "$_auth_header" ]; then
            curl -fsSL -H "$_auth_header" -o "$_out" "$_url"
        else
            curl -fsSL -o "$_out" "$_url"
        fi
    elif command -v wget >/dev/null 2>&1; then
        if [ -n "$_auth_header" ]; then
            wget -q --header="$_auth_header" -O "$_out" "$_url"
        else
            wget -q -O "$_out" "$_url"
        fi
    else
        err "curl or wget is required to download files"
    fi
}

usage() {
    cat <<EOF
StrayMark CLI installer

USAGE:
    install.sh [OPTIONS]

OPTIONS:
    --tag <TAG>    Install a specific version (e.g. cli-1.0.0). Default: latest
    --to  <DIR>    Installation directory. Default: ~/.local/bin (or /usr/local/bin with sudo)
    --help         Show this help message

ENVIRONMENT:
    GITHUB_TOKEN   GitHub API token to avoid rate limits

EXAMPLES:
    curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.sh | sh
    curl -fsSL ... | sh -s -- --tag cli-1.0.0
    curl -fsSL ... | sh -s -- --to /usr/local/bin
EOF
}

# ── Parse arguments ──────────────────────────────────────────────────────

TAG=""
INSTALL_DIR=""

while [ $# -gt 0 ]; do
    case "$1" in
        --tag)
            [ $# -ge 2 ] || err "--tag requires a value"
            TAG="$2"
            shift 2
            ;;
        --to)
            [ $# -ge 2 ] || err "--to requires a value"
            INSTALL_DIR="$2"
            shift 2
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            err "unknown option: $1 (use --help for usage)"
            ;;
    esac
done

# ── Detect platform ─────────────────────────────────────────────────────

detect_target() {
    _os="$(uname -s)"
    _arch="$(uname -m)"

    case "$_os" in
        Linux|linux)    _os_part="unknown-linux-gnu" ;;
        Darwin|darwin)  _os_part="apple-darwin" ;;
        *)              err "unsupported OS: $_os" ;;
    esac

    case "$_arch" in
        x86_64|amd64)   _arch_part="x86_64" ;;
        aarch64|arm64)  _arch_part="aarch64" ;;
        *)              err "unsupported architecture: $_arch" ;;
    esac

    # Only supported combinations
    case "${_arch_part}-${_os_part}" in
        x86_64-unknown-linux-gnu)   ;;
        x86_64-apple-darwin)        ;;
        aarch64-apple-darwin)       ;;
        *)  err "unsupported platform: ${_os}/${_arch} (target: ${_arch_part}-${_os_part})" ;;
    esac

    TARGET="${_arch_part}-${_os_part}"
}

# ── Resolve install directory ────────────────────────────────────────────

resolve_install_dir() {
    if [ -n "$INSTALL_DIR" ]; then
        return
    fi

    if [ "$(id -u)" -eq 0 ]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="${HOME}/.local/bin"
    fi
}

# ── Get latest tag ───────────────────────────────────────────────────────

get_latest_tag() {
    _api_url="https://api.github.com/repos/${REPO}/releases"
    _tmp_json="${TMPDIR_CLEANUP}/releases.json"

    say "fetching latest CLI release info..."
    if ! download "$_api_url" "$_tmp_json" 2>/dev/null; then
        echo ""
        echo "  Failed to fetch release info from GitHub API." >&2
        echo "  This may be due to rate limiting." >&2
        echo "  Set GITHUB_TOKEN to authenticate, or use --tag to specify a version." >&2
        echo "" >&2
        err "could not determine latest version"
    fi

    # Extract the first tag_name starting with "cli-" (POSIX-compatible)
    TAG=$(sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\(cli-[^"]*\)".*/\1/p' "$_tmp_json" | head -1)

    if [ -z "$TAG" ]; then
        err "could not find a CLI release (cli-* tag) from GitHub API response"
    fi

    say "latest version: ${TAG}"
}

# ── Main ─────────────────────────────────────────────────────────────────

main() {
    detect_target
    resolve_install_dir

    # Check dependencies
    if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
        err "curl or wget is required"
    fi
    need tar

    # Create temp directory with cleanup trap
    TMPDIR_CLEANUP="$(mktemp -d)"
    trap 'rm -rf "$TMPDIR_CLEANUP"' EXIT

    # Get version
    if [ -z "$TAG" ]; then
        get_latest_tag
    else
        say "using specified version: ${TAG}"
    fi

    # Strip leading 'cli-' prefix for the version in asset name
    VERSION_NUM="${TAG#cli-}"

    # Build asset name and URL
    ASSET="straymark-cli-v${VERSION_NUM}-${TARGET}.tar.gz"
    URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"

    say "downloading ${ASSET}..."
    ARCHIVE="${TMPDIR_CLEANUP}/${ASSET}"
    if ! download "$URL" "$ARCHIVE"; then
        echo "" >&2
        echo "  Download failed. Possible causes:" >&2
        echo "  - Version ${TAG} does not exist" >&2
        echo "  - No binary available for ${TARGET}" >&2
        echo "  - Network connectivity issue" >&2
        echo "" >&2
        err "failed to download ${URL}"
    fi

    # Extract binary
    say "extracting ${BINARY}..."
    tar xzf "$ARCHIVE" -C "$TMPDIR_CLEANUP"

    if [ ! -f "${TMPDIR_CLEANUP}/${BINARY}" ]; then
        err "binary '${BINARY}' not found in archive"
    fi

    # Install
    mkdir -p "$INSTALL_DIR"

    if ! cp "${TMPDIR_CLEANUP}/${BINARY}" "${INSTALL_DIR}/${BINARY}" 2>/dev/null; then
        echo "" >&2
        echo "  Permission denied writing to ${INSTALL_DIR}." >&2
        echo "  Try one of:" >&2
        echo "    sudo sh install.sh --to /usr/local/bin" >&2
        echo "    sh install.sh --to ~/.local/bin" >&2
        echo "" >&2
        err "failed to install binary to ${INSTALL_DIR}"
    fi

    chmod +x "${INSTALL_DIR}/${BINARY}"
    say "installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"

    # Verify
    if "${INSTALL_DIR}/${BINARY}" --version >/dev/null 2>&1; then
        _version=$("${INSTALL_DIR}/${BINARY}" --version)
        say "verified: ${_version}"
    else
        say "warning: could not verify installation (binary may not run on this platform)"
    fi

    # PATH check
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*)
            ;;
        *)
            echo ""
            echo "  ${INSTALL_DIR} is not in your PATH."
            echo "  Add it by running:"
            echo ""
            echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
            echo ""
            echo "  To make it permanent, add that line to your shell profile:"
            echo "    ~/.bashrc, ~/.zshrc, or ~/.profile"
            echo ""
            ;;
    esac

    say "done!"
}

main
