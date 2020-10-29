#!/bin/bash
DIR=$(dirname "$0")
PID=$(cat "$DIR/pid")
kill $PID
