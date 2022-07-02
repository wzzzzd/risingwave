#!/bin/bash

# Exits as soon as any line fails.
set -euo pipefail

source ci/scripts/common.env.sh

cargo build

echo "--- Generate RiseDev CI config"
cp risedev-components.ci.env risedev-components.user.env

echo "--- Prepare RiseDev playground"
cargo make pre-start-playground
cargo make link-all-in-one-binaries

#echo "--- e2e, ci-3cn-1fe, streaming"
#cargo make ci-start ci-3cn-1fe
#sqllogictest -p 4566 -d dev  './e2e_test/streaming/**/*.slt' -j 16
#
#echo "--- Kill cluster"
#cargo make ci-kill
#
#echo "--- e2e, ci-3cn-1fe, delta join"
#cargo make ci-start ci-3cn-1fe
#sqllogictest -p 4566 -d dev  './e2e_test/streaming_delta_join/**/*.slt'
#
#echo "--- Kill cluster"
#cargo make ci-kill

echo "--- e2e, ci-3cn-1fe, batch distributed"
cargo make ci-start ci-3cn-1fe
sqllogictest -p 4566 -d dev  './e2e_test/ddl/**/*.slt'
sqllogictest -p 4566 -d dev  './e2e_test/batch/**/*.slt' -j 16

echo "--- Kill cluster"
cargo make ci-kill
