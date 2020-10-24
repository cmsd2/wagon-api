#!/bin/bash

set -euxo pipefail

aws-build lambda --bin api

cp target/latest-lambda target/lambda.zip
