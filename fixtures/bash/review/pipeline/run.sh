#!/bin/bash
source ./process.sh

main() {
  local target="${1:-data.txt}"
  process_file "$target" | tee output.txt
}

main "$@"
