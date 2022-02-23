#!/bin/bash
set -e

CONFIG=release
mkdir -p www/pkg

rustup target add wasm32-unknown-unknown

if [ -z "$(cargo install --list | grep wasm-pack)" ]
then
	cargo install wasm-pack
fi

wasm-pack build --release --target web

basic-http-server