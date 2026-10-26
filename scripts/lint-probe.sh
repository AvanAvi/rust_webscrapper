#!/usr/bin/env bash
# Proves every configured lint actually fires: builds a scratch crate with the
# workspace's [workspace.lints], clippy.toml, and toolchain, runs clippy on
# scripts/fixtures/lint_probe.rs, and fails if any expected lint is missing.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

mkdir -p "$SCRATCH/src"
cp "$ROOT/clippy.toml" "$ROOT/rust-toolchain.toml" "$SCRATCH/"
cp "$ROOT/scripts/fixtures/lint_probe.rs" "$SCRATCH/src/lib.rs"
{
  printf '[package]\nname = "lint-probe"\nversion = "0.0.0"\nedition = "2024"\npublish = false\n\n'
  sed -n '/^\[workspace\.lints/,/^$/p' "$ROOT/Cargo.toml" | sed 's/^\[workspace\.lints/[lints/'
} > "$SCRATCH/Cargo.toml"

OUTPUT="$(cd "$SCRATCH" && cargo clippy --quiet -- -D warnings 2>&1 || true)"

EXPECTED=(cognitive_complexity too_many_lines too_many_arguments unwrap_used expect_used panic todo unsafe_code missing_docs)
MISSING=0
for LINT in "${EXPECTED[@]}"; do
  PATTERN="${LINT//_/[-_]}"
  if grep -Eq "(clippy::|#|-[DF] |\()${PATTERN}([^a-z_-]|$)" <<<"$OUTPUT"; then
    echo "fires: $LINT"
  else
    echo "MISSING: $LINT"
    MISSING=1
  fi
done

if [[ "$MISSING" -ne 0 ]]; then
  echo "$OUTPUT"
  exit 1
fi
echo "lint-probe: all ${#EXPECTED[@]} lints fire"
