#!/bin/bash
FROST_BRIGHTSKY_ENDPOINT="https://api.brightsky.dev/weather" RUST_LOG=warn target/release/frost 2>/dev/null &
echo $! >pid
exit
