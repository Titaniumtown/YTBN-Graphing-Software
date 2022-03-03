#!/bin/bash
set -e

rm -fr tmp pkg | true #delete tmp folder if exists

#apply optimizations via wasm-opt
wasm_opt() {
    wasm-opt -Os -o pkg/integral_site_bg_2.wasm pkg/integral_site_bg.wasm
    mv pkg/integral_site_bg_2.wasm pkg/integral_site_bg.wasm
}

if test "$1" == "" || test "$1" == "release"; then
    wasm-pack build --target web --release --no-typescript
    wasm_opt #apply wasm optimizations

    llvm-strip --strip-all pkg/integral_site_bg.wasm
elif test "$1" == "debug"; then
    wasm-pack build --target web --debug --no-typescript
else
    echo "ERROR: build.sh, argument invalid"
    exit 1
fi

mkdir tmp
cp -r pkg/integral_site_bg.wasm pkg/integral_site.js tmp/
cp www/index.html www/style.css tmp/

echo "Total size: $(du -sb tmp)"
echo "Binary size: $(du -sb tmp/integral_site_bg.wasm)"