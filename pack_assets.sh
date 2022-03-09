#!/bin/bash
rm -fr data.tar.zst | true
cd assets
tar -I 'zstd --ultra -22' -cf ../data.tar.zst *.*