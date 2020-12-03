#!/bin/bash

set -euxo pipefail

git clone https://github.com/cmsd2/aws-build

cargo install --git https://github.com/cmsd2/aws-build
