#!/bin/bash

set -euxo pipefail

cd infra && npm run cdk synth -- 'WagonApi'

