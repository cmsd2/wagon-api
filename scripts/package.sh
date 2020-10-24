#!/bin/bash

set -euxo pipefail

LAMBDA_NAME="latest-lambda"
BIN_DIR="target"
DIST_DIR="dist/api"
LAMBDA_BIN="$BIN_DIR/$LAMBDA_NAME"
LAMBDA_ZIP="lambda.zip"

mkdir -p "$DIST_DIR"

cp "$LAMBDA_BIN" "$DIST_DIR/$LAMBDA_ZIP"
