#!/bin/bash
wasm-pack build --target web --dev
cp -r static/* pkg
echo "import('/frost.js').then(async wasm => await wasm.default());" >pkg/index.js
