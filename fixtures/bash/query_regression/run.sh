#!/bin/bash
source ./lib.sh
. ./env.sh
export APP_PORT=8080
start_server --port "$APP_PORT"
