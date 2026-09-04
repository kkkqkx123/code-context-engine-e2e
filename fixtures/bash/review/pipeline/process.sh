#!/bin/bash
set -euo pipefail

fetch_lines() {
  grep -v "^#" "$1" | tr '[:upper:]' '[:lower:]' | sort | uniq -c | sort -nr
}

process_file() {
  local input="$1"
  fetch_lines "$input" | head -n 20 | while read -r count word; do
    echo "$word: $count"
  done
}
