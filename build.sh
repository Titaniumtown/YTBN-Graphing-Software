#!/bin/bash
set -e

rm -fr tmp | true
rm -fr pkg | true

# cargo test

export RUSTFLAGS="--cfg=web_sys_unstable_apis"

if test "$1" == "" || test "$1" == "release"; then
    cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --lib
    llvm-strip -s target/wasm32-unknown-unknown/release/ytbn_graphing_software.wasm
    export TYPE="release"
elif test "$1" == "debug"; then
    cargo build --dev --target wasm32-unknown-unknown -Z build-std=std,panic_unwind -Z build-std-features=panic-abort --lib
    export TYPE="debug"
else
    echo "ERROR: build.sh, argument invalid"
    exit 1
fi

wasm-bindgen target/wasm32-unknown-unknown/${TYPE}/ytbn_graphing_software.wasm --out-dir pkg --target web --no-typescript

if test "$TYPE" == "release"; then
    echo "running wasm-opt..."
    time wasm-opt -Oz --flatten --nominal --dae --dce --code-folding --const-hoisting --coalesce-locals-learning --vacuum --merge-locals --merge-blocks --no-exit-runtime --fast-math --traps-never-happen -o pkg/ytbn_graphing_software_bg_2.wasm pkg/ytbn_graphing_software_bg.wasm
    mv pkg/ytbn_graphing_software_bg_2.wasm pkg/ytbn_graphing_software_bg.wasm
fi

mkdir tmp
cp -r pkg/ytbn_graphing_software_bg.wasm tmp/

# sed -i 's/fatal: true/fatal: false/g' pkg/ytbn_graphing_software.js

# sed -i "s/TextEncoder('utf-8')/TextEncoder('utf-8', { ignoreBOM: true, fatal: false })/g" pkg/ytbn_graphing_software.js

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
