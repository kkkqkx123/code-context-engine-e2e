#!/bin/bash
APP_NAME="demo"
MAX_RETRIES=3
VERBOSE=true

log_message() {
  echo "[$APP_NAME] $1"
}

retry_count=0
log_message "starting with retries=$MAX_RETRIES"
