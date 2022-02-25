#!/bin/bash
set -e
wasm-pack build --target web --release --no-typescript
wasm-opt -Oz -o pkg/integral_site_bg_2.wasm pkg/integral_site_bg.wasm
mv pkg/integral_site_bg_2.wasm pkg/integral_site_bg.wasm
llvm-strip --strip-all pkg/integral_site_bg.wasm

rm -fr tmp | true #delete tmp folder if exists
mkdir tmp tmp/pkg
cp -r pkg/integral_site_bg.wasm pkg/integral_site.js tmp/pkg/
cp www/index.html www/style.css tmp/

sed -i 's/\.\.\/pkg/\.\/pkg/g' tmp/index.html

git ls-files --exclude-standard --others . >/dev/null 2>&1; ec=$?
if test "$ec" = 0; then
    echo some untracked files
elif test "$ec" = 1; then
    commit=$(git rev-parse HEAD)
    sed -i "s/let commit = ''/let commit = '${commit}'/g" tmp/index.html
fi
