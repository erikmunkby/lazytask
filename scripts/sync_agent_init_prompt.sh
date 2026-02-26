#!/usr/bin/env bash
set -euo pipefail

START_TAG="<EXTREMELY_IMPORTANT>"
END_TAG="</EXTREMELY_IMPORTANT>"
COPY_IF_MISMATCH=0
AGENTS_PATH="AGENTS.md"
PROMPT_PATH="src/config/prompts/agent_init.md"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --copy-if-mismatch)
      COPY_IF_MISMATCH=1
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ ! -f "$AGENTS_PATH" ]]; then
  echo "file not found: $AGENTS_PATH" >&2
  exit 2
fi

if [[ ! -f "$PROMPT_PATH" ]]; then
  echo "file not found: $PROMPT_PATH" >&2
  exit 2
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

raw_body_file="$tmp_dir/prompt_raw_body.txt"
body_file="$tmp_dir/prompt_body.txt"
current_file="$tmp_dir/current_body.txt"
expected_norm="$tmp_dir/expected_norm.txt"
current_norm="$tmp_dir/current_norm.txt"
updated_file="$tmp_dir/agents_updated.md"

start_count="$(grep -Ec "^[[:space:]]*${START_TAG}[[:space:]]*$" "$AGENTS_PATH" || true)"
end_count="$(grep -Ec "^[[:space:]]*${END_TAG}[[:space:]]*$" "$AGENTS_PATH" || true)"

if [[ "$start_count" -ne 1 ]]; then
  echo "expected exactly one start tag line (${START_TAG}) in $AGENTS_PATH, found: $start_count" >&2
  exit 2
fi

if [[ "$end_count" -ne 1 ]]; then
  echo "expected exactly one end tag line (${END_TAG}) in $AGENTS_PATH, found: $end_count" >&2
  exit 2
fi

# Mirror src/config/prompts.rs::prompt_body behavior for metadata-comment assets.
awk '
BEGIN { in_comment = 0 }
NR == 1 && ($0 ~ /^<!---/ || $0 ~ /^<!--/) { in_comment = 1; next }
in_comment {
  if ($0 ~ /-->/) { in_comment = 0; next }
  next
}
{ print }
' "$PROMPT_PATH" > "$raw_body_file"

# Mirror fallback split_once("\n---\n") behavior.
if grep -qx -- '---' "$raw_body_file"; then
  awk 'found { print } $0 == "---" && !found { found = 1 }' "$raw_body_file" > "$body_file"
else
  cp "$raw_body_file" "$body_file"
fi

awk -v s="$START_TAG" -v e="$END_TAG" '
BEGIN { in_block = 0; seen_start = 0; seen_end = 0 }
$0 ~ "^[[:space:]]*" s "[[:space:]]*$" {
  if (seen_start == 1) {
    print "multiple start tags found while parsing block" > "/dev/stderr"
    exit 2
  }
  seen_start = 1
  in_block = 1
  next
}
$0 ~ "^[[:space:]]*" e "[[:space:]]*$" {
  if (seen_start == 0) {
    print "end tag appears before start tag" > "/dev/stderr"
    exit 2
  }
  seen_end = 1
  in_block = 0
  exit 0
}
in_block { print }
END {
  if (seen_start == 0 || seen_end == 0) {
    print "missing EXTREMELY_IMPORTANT block boundaries" > "/dev/stderr"
    exit 2
  }
}
' "$AGENTS_PATH" > "$current_file"

normalize_file() {
  local input_file="$1"
  local output_file="$2"
  sed 's/\r$//' "$input_file" | awk '
  {
    lines[NR] = $0
  }
  END {
    first = 1
    while (first <= NR && lines[first] == "") {
      first++
    }
    last = NR
    while (last >= first && lines[last] == "") {
      last--
    }
    for (i = first; i <= last; i++) {
      print lines[i]
    }
  }' > "$output_file"
}

normalize_file "$body_file" "$expected_norm"
normalize_file "$current_file" "$current_norm"

if cmp -s "$expected_norm" "$current_norm"; then
  echo "agent prompt block is up to date"
  exit 0
fi

if [[ "$COPY_IF_MISMATCH" -eq 0 ]]; then
  echo "agent prompt block is out of sync; run with --copy-if-mismatch" >&2
  exit 1
fi

awk -v s="$START_TAG" -v e="$END_TAG" -v body_file="$expected_norm" '
BEGIN {
  body_len = 0
  while ((getline line < body_file) > 0) {
    body[++body_len] = line
  }
  close(body_file)
  in_block = 0
  inserted = 0
}
$0 ~ "^[[:space:]]*" s "[[:space:]]*$" {
  print
  for (i = 1; i <= body_len; i++) {
    print body[i]
  }
  in_block = 1
  inserted = 1
  next
}
$0 ~ "^[[:space:]]*" e "[[:space:]]*$" {
  in_block = 0
  print
  next
}
!in_block { print }
END {
  if (inserted == 0) {
    print "failed to insert updated prompt block" > "/dev/stderr"
    exit 2
  }
}
' "$AGENTS_PATH" > "$updated_file"

mv "$updated_file" "$AGENTS_PATH"
echo "updated $AGENTS_PATH from $PROMPT_PATH"
