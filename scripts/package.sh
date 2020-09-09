#!/bin/bash

set -euxo pipefail

LAMBDA_NAME="api"
BUILD_DIR="target/release"
LAMBDA_BIN="$BUILD_DIR/$LAMBDA_NAME"
PKG_DIR="$BUILD_DIR/$LAMBDA_NAME.lambda"
LAMBDA_ZIP="$BUILD_DIR/$LAMBDA_NAME.zip"

mkdir -p "$PKG_DIR"

cd "$PKG_DIR"

cp "$LAMBDA_BIN" bootstrap

zip "$LAMBDA_ZIP" bootstrap 
