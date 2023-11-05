#!/bin/bash

set -euxo pipefail

cargo lambda build --output-format zip

