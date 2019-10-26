#!/bin/sh
cargo kcov -- --include-path src
echo "Generated report should be written as: target/cov/index.html"
