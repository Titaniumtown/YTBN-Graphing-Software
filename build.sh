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
    RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web --dev --no-typescript
else
    echo "ERROR: build.sh, argument invalid"
    exit 1
fi

mkdir tmp
cp -r pkg/ytbn_graphing_software_bg.wasm tmp/

sed -i 's/fatal: true/fatal: false/g' pkg/ytbn_graphing_software.js

sed -i "s/TextEncoder('utf-8')/TextEncoder('utf-8', { ignoreBOM: true, fatal: false })/g" pkg/ytbn_graphing_software.js

minify pkg/ytbn_graphing_software.js > tmp/ytbn_graphing_software.js

cp www/* tmp/
minify www/index.html > tmp/index.html
minify www/sw.js > tmp/sw.js

wasm_sum=($(md5sum tmp/ytbn_graphing_software_bg.wasm))
js_sum=($(md5sum tmp/ytbn_graphing_software.js))
sum=($(echo "$wasm_sum $js_sum" | md5sum))

echo "sum: $sum"

new_wasm_name="${sum}.wasm"
new_js_name="${sum}.js"


mv tmp/ytbn_graphing_software_bg.wasm "tmp/${new_wasm_name}"
mv tmp/ytbn_graphing_software.js "tmp/${new_js_name}"


sed -i "s/ytbn_graphing_software_bg.wasm/${new_wasm_name}/g" tmp/*.*

sed -i "s/ytbn_graphing_software.js/${new_js_name}/g" tmp/*.*



echo "Total size: $(du -sb tmp)"
echo "Binary size: $(du -sb tmp/${new_wasm_name})"
