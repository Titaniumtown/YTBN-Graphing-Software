#!/bin/bash
rm -fr assets.tar.zst | true
tar -I 'zstd --ultra -22' --strip-components=9999 -cf ./assets.tar.zst assets/text.json $1
