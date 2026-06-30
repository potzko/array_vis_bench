#!/usr/bin/env bash
# Workspace tests with a single aggregate summary, via cargo-nextest.
# nextest doesn't run doc-tests, so do those in a second pass.
# Args pass through:  ./test_summary.sh -p spec_catalog
cargo nextest run "$@"; n=$?
cargo test --doc "$@"; d=$?
exit $(( n || d ))
