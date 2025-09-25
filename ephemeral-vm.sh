#!/bin/bash

# Set library path for krunvm
export DYLD_LIBRARY_PATH="/opt/homebrew/opt/libkrunfw/lib:/opt/homebrew/opt/libkrun/lib:$DYLD_LIBRARY_PATH"

# Run the ephemeral-vm binary with all arguments
exec "$(dirname "$0")/target/debug/ephemeral-vm" "$@"