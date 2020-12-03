#!/bin/bash

set -euxo pipefail

aws-build lambda --package openssl-devel --bin api

cp target/latest-lambda target/lambda-api.zip

aws-build lambda --package openssl-devel --bin jwt_authorizer

cp target/latest-lambda target/lambda-jwt_authorizer.zip
