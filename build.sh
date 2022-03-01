#!/bin/bash
set -e
wasm-pack build --target web --release --no-typescript
wasm-opt -Os -o pkg/integral_site_bg_2.wasm pkg/integral_site_bg.wasm
mv pkg/integral_site_bg_2.wasm pkg/integral_site_bg.wasm
llvm-strip --strip-all pkg/integral_site_bg.wasm

rm -fr tmp | true #delete tmp folder if exists
mkdir tmp tmp/pkg
cp -r pkg/integral_site_bg.wasm pkg/integral_site.js tmp/
cp www/index.html www/style.css tmp/

echo "Total size: $(du -sb tmp)"
echo "Binary size: $(du -sb tmp/integral_site_bg.wasm)"