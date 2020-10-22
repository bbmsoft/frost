#!/bin/bash
git pull &&
    cargo make build -p production &&
    ./stop.sh &&
    ./start.sh
