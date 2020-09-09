#!/bin/bash

set -euxo pipefail

LAMBDA_NAME="api"
BIN_DIR="target/release"
LAMBDA_BIN="$BIN_DIR/$LAMBDA_NAME"
BUILD_DIR="dist"
PKG_DIR="$BUILD_DIR/$LAMBDA_NAME"
LAMBDA_ZIP="lambda.zip"

mkdir -p "$PKG_DIR"

cp "$LAMBDA_BIN" "$PKG_DIR/bootstrap"

cd "$PKG_DIR"

zip "$LAMBDA_ZIP" bootstrap
