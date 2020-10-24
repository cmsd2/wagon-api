#!/bin/bash

set -euxo pipefail

aws-build lambda --bin api
