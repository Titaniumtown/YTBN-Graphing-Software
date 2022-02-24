#!/bin/bash
set -e
wasm-pack build --target web --release --no-typescript
wasm-opt -Oz -o pkg/integral_site_bg_2.wasm pkg/integral_site_bg.wasm
mv pkg/integral_site_bg_2.wasm pkg/integral_site_bg.wasm
llvm-strip --strip-all pkg/integral_site_bg.wasm
