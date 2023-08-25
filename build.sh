#!/bin/bash
set -e

rm -fr tmp | true
rm -fr pkg | true

# cargo test

export RUSTFLAGS="--cfg=web_sys_unstable_apis"

if test "$1" == "" || test "$1" == "release"; then
    time cargo build --release --target wasm32-unknown-unknown -Z build-std=core,compiler_builtins,alloc,std,panic_abort,panic_unwind,proc_macro,unwind -Z build-std-features=panic_immediate_abort --lib --timings
    llvm-strip -s target/wasm32-unknown-unknown/release/ytbn_graphing_software.wasm
    export TYPE="release"
    elif test "$1" == "debug"; then
    time cargo build --target wasm32-unknown-unknown -Z build-std=core,compiler_builtins,alloc,std,panic_abort,panic_unwind,proc_macro,unwind -Z build-std-features=panic-unwind --lib
    export TYPE="debug"
else
    echo "ERROR: build.sh, argument invalid"
    exit 1
fi

pre_size=$(du -sb target/wasm32-unknown-unknown/${TYPE}/ytbn_graphing_software.wasm | awk '{ print $1 }')
echo "compiled size: $pre_size"

wasm-bindgen target/wasm32-unknown-unknown/${TYPE}/ytbn_graphing_software.wasm --out-dir pkg --target web --no-typescript

#if test "$TYPE" == "release"; then
#    echo "running wasm-opt..."
#    time wasm-opt --converge -Oz --code-folding --const-hoisting --coalesce-locals-learning --vacuum --merge-locals --merge-blocks --fast-math --precompute --rse --low-memory-unused --once-reduction --optimize-instructions --licm --intrinsic-lowering \
#    --dce --dae-optimizing --inlining-optimizing --strip-debug \
#    -o pkg/ytbn_graphing_software_bg_2.wasm pkg/ytbn_graphing_software_bg.wasm
#    mv pkg/ytbn_graphing_software_bg_2.wasm pkg/ytbn_graphing_software_bg.wasm
#fi

mkdir tmp
cp -r pkg/ytbn_graphing_software_bg.wasm tmp/

sed -i 's/fatal: true/fatal: false/g' pkg/ytbn_graphing_software.js

sed -i "s/TextEncoder('utf-8')/TextEncoder('utf-8', { ignoreBOM: true, fatal: false })/g" pkg/ytbn_graphing_software.js

#minify pkg/ytbn_graphing_software.js > tmp/ytbn_graphing_software.js

cp www/* tmp/
cp assets/logo.svg tmp/
#minify www/index.html > tmp/index.html
#minify www/sw.js > tmp/sw.js
cp www/index.html www/sw.js pkg/ytbn_graphing_software.js tmp/


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



new_size=$(du -b tmp/${new_wasm_name} | awk '{ print $1 }')
diff=$(echo "scale=5 ; $new_size / $pre_size" | bc)
percent=$(echo "scale=5 ; (1-$diff)*100" | bc)
echo "Total size: $(du -sb tmp)"
echo "Binary size: $new_size reduced: $percent%"
