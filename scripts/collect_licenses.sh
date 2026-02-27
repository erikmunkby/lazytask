#!/usr/bin/env bash
set -euo pipefail

# Collect third-party licenses for the shipped binary into THIRD_PARTY_LICENSES.
# Uses only cargo metadata + jq — no extra tools needed.
# Dev-dependencies are excluded since they don't ship in the binary.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT="$ROOT_DIR/THIRD_PARTY_LICENSES"

if ! command -v jq &>/dev/null; then
  echo "jq not found. Install with: brew install jq (macOS) or apt install jq (Linux)"
  exit 1
fi

echo "Generating third-party license file..."
META=$(cargo metadata --format-version 1 --manifest-path "$ROOT_DIR/Cargo.toml")

ROOT_ID=$(echo "$META" | jq -r '.resolve.root')
# Walk the resolve graph from the root, collecting only normal (non-dev, non-build) dep IDs.
# jq outputs all reachable package IDs via BFS-like expansion.
RUNTIME_IDS=$(echo "$META" | jq -r --arg root "$ROOT_ID" '
  .resolve.nodes as $nodes |
  # Build lookup: id -> [normal dep pkg ids]
  (reduce $nodes[] as $n ({}; . + {
    ($n.id): [$n.deps[] | select(.dep_kinds | any(.kind == null)) | .pkg]
  })) as $graph |
  # BFS from root
  {visited: [], queue: [$root]} |
  until(.queue | length == 0;
    .queue[0] as $cur |
    .queue[1:] as $rest |
    if (.visited | index($cur)) then .queue = $rest
    else
      .visited += [$cur] |
      .queue = ($rest + ($graph[$cur] // []))
    end
  ) |
  .visited[] | select(. != $root)
')

{
  echo "# Third-Party Licenses"
  echo ""
  echo "Licenses for all third-party dependencies."
  echo ""

  echo "$META" | jq -r --argjson ids "$(echo "$RUNTIME_IDS" | jq -R . | jq -s .)" '
    [.packages[] | select(.id as $id | $ids | index($id))]
    | sort_by(.name | ascii_downcase)[]
    | [.name, .version, (.license // "Unknown"), (.repository // ""), .manifest_path] | @tsv
  ' | while IFS=$'\t' read -r name version license_id repo manifest; do
    echo "## ${name} ${version} — ${license_id}"
    echo ""
    if [ -n "$repo" ]; then
      echo "Repository: ${repo}"
      echo ""
    fi

    # Try to find a LICENSE file in the package source
    pkg_dir=$(dirname "$manifest")
    found_license=""
    for candidate in LICENSE LICENSE-MIT LICENSE-APACHE LICENSE.md COPYING; do
      if [ -f "$pkg_dir/$candidate" ]; then
        cat "$pkg_dir/$candidate"
        found_license="yes"
        break
      fi
    done

    if [ -z "$found_license" ]; then
      echo "Licensed under ${license_id}. See package repository for full text."
    fi
    echo ""
  done
} > "$OUTPUT"

echo "Written to $OUTPUT ($(wc -l < "$OUTPUT" | tr -d ' ') lines)"
