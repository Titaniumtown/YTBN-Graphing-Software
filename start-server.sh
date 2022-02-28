#!/bin/bash
set -e

if test "$1" == "" || test "$1" == "release"; then
    bash build.sh
elif test "$1" == "debug"; then
    wasm-pack build --target web --debug --no-typescript

    rm -fr tmp | true #delete tmp folder if exists
    mkdir tmp tmp/pkg
    cp -r pkg/integral_site_bg.wasm pkg/integral_site.js tmp/pkg/
    cp www/index.html www/style.css tmp/

    sed -i 's/\.\.\/pkg/\.\/pkg/g' tmp/index.html
fi

basic-http-server tmp/