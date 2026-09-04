#!/bin/bash
source ./vars.sh

run_with_retry() {
  local attempt=1
  while [ $attempt -le $MAX_RETRIES ]; do
    log_message "attempt $attempt"
    attempt=$((attempt + 1))
  done
}

run_with_retry
