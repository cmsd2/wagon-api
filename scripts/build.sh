#!/bin/bash

set -euxo pipefail

aws-build lambda --package openssl-devel --bin api

cp target/latest-lambda target/lambda-api.zip

aws-build lambda --package openssl-devel --bin authorizer

cp target/latest-lambda target/lambda-authorizer.zip

