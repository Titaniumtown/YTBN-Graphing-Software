#!/bin/bash
set -e

rm -fr tmp | true
rm -fr pkg | true

# cargo test

#apply optimizations via wasm-opt
wasm_opt() {
	wasm-opt -Oz -o pkg/ytbn_graphing_software_bg_2.wasm pkg/ytbn_graphing_software_bg.wasm
	mv pkg/ytbn_graphing_software_bg_2.wasm pkg/ytbn_graphing_software_bg.wasm
}

if test "$1" == "" || test "$1" == "release"; then
	RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web --release --no-typescript
	echo "Binary size (pre-wasm_opt): $(du -sb pkg/ytbn_graphing_software_bg.wasm)"
	wasm_opt #apply wasm optimizations
	echo "Binary size (pre-strip): $(du -sb pkg/ytbn_graphing_software_bg.wasm)"
	llvm-strip --strip-all pkg/ytbn_graphing_software_bg.wasm
elif test "$1" == "debug"; then
	RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web --debug --no-typescript
else
	echo "ERROR: build.sh, argument invalid"
	exit 1
fi

mkdir tmp
cp -r pkg/ytbn_graphing_software_bg.wasm pkg/ytbn_graphing_software.js tmp/

sed -i 's/fatal: true/fatal: false/g' tmp/ytbn_graphing_software.js

sed -i "s/TextEncoder('utf-8')/TextEncoder('utf-8', { ignoreBOM: true, fatal: false })/g" tmp/ytbn_graphing_software.js


cp www/* tmp/

echo "Total size: $(du -sb tmp)"
echo "Binary size: $(du -sb tmp/ytbn_graphing_software_bg.wasm)"
