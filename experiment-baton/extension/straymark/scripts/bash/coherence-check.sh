#!/usr/bin/env bash
# straymark coherence-check — invoked by SpecKit hooks (normally before_implement).
#
# Read-only: runs the Baton coherence engine scoped to the active feature and
# surfaces findings before the agent implements. Never mutates the repository,
# and never breaks the SpecKit flow (graceful degradation throughout).
set -uo pipefail

EVENT="${1:-before_implement}"
ROOT="$(pwd)"
CONFIG="$ROOT/.specify/extensions/straymark/straymark-config.yml"

# cfg <key> <default> — read a top-level scalar from the YAML config.
cfg() {
  local v=""
  if [ -f "$CONFIG" ]; then
    v="$(grep -E "^$1:" "$CONFIG" | head -1 \
      | sed -E "s/^$1:[[:space:]]*//; s/[[:space:]]*#.*$//; s/[\"']//g; s/[[:space:]]*$//")"
  fi
  printf '%s' "${v:-$2}"
}

GATE="$(cfg gate advisory)"
MIN_CONF="$(cfg min_confidence medium)"
BIN="$(cfg binary "")"

# Resolve the binary: config path → PATH → cargo (dev, needs $BATON_REPO).
run_baton() {
  if [ -n "$BIN" ] && [ -x "$BIN" ]; then
    "$BIN" "$@"
  elif command -v straymark-baton >/dev/null 2>&1; then
    straymark-baton "$@"
  elif command -v cargo >/dev/null 2>&1 && [ -n "${BATON_REPO:-}" ]; then
    ( cd "$BATON_REPO" && cargo run -q -p straymark-baton -- "$@" )
  else
    return 127
  fi
}

# Resolve the active feature directory name from .specify/feature.json.
FEATURE=""
FJSON="$ROOT/.specify/feature.json"
if [ -f "$FJSON" ]; then
  FEATURE="$(grep -oE '"feature_directory"[[:space:]]*:[[:space:]]*"[^"]+"' "$FJSON" \
    | sed -E 's/.*"([^"]+)"$/\1/')"
  FEATURE="$(basename "$FEATURE" 2>/dev/null)"
fi

ARGS=(coherence "$ROOT" --min-confidence "$MIN_CONF")
[ -n "$FEATURE" ] && ARGS+=(--spec "$FEATURE")

echo "[straymark] coherence check ($EVENT)${FEATURE:+ for feature $FEATURE}…"
OUT="$(run_baton "${ARGS[@]}" 2>&1)"
CODE=$?

if [ "$CODE" -eq 127 ]; then
  echo "[straymark] straymark-baton not found — skipping coherence check."
  echo "[straymark] set 'binary:' in straymark-config.yml, add it to PATH, or export BATON_REPO. Flow continues."
  exit 0
fi

echo "$OUT"

# baton exit codes: 0 clean, 1 blocking findings present, 2 usage error.
if [ "$CODE" -eq 1 ] && [ "$GATE" = "block" ]; then
  echo "[straymark] BLOCKING coherence findings and gate=block — resolve before implementing."
  exit 1
fi
exit 0
