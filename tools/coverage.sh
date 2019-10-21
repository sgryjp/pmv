#!/bin/sh
SCRIPT=`readlink -f $0`
SCRIPT_PATH=`dirname $SCRIPT`

cd "$SCRIPT_PATH/.."

rm -rf target/debug/

cargo test --no-run
if [ $? -ne 0 ]; then
    echo "Unit test failed."
    exit 1
fi

kcov target/cov `find target/debug -regex 'target/debug/pmv-[^\.]+$'`
if [ $? -ne 0 ]; then
    echo "Failed to generate HTML coverage report."
    exit 1
fi

echo "Generated report should be written as: target/cov/index.html"
