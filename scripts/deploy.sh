#!/bin/bash

set -euxo pipefail

cd infra && npm run cdk deploy -- --require-approval never '*'

