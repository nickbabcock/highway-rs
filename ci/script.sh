#!/bin/bash

set -ex

# Setup some variables for executing cargo commands.
# Things are a little different if we're testing with cross.
if [ ! z "$CROSS_TARGET" ]; then
  rustup target add "$CROSS_TARGET"
  cargo install cross --force
  export CARGO_CMD="cross"
  export TARGET_PARAM="--target $CROSS_TARGET"
else
  export CARGO_CMD="cargo"
  export TARGET_PARAM=""
fi

"$CARGO_CMD" build --verbose $TARGET_PARAM
"$CARGO_CMD" doc --verbose $TARGET_PARAM
"$CARGO_CMD" test --verbose $TARGET_PARAM
"$CARGO_CMD" bench --no-run --verbose $TARGET_PARAM
