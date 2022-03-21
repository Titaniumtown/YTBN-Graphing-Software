#!/bin/bash
rm -fr assets.tar.zst | true
cd assets
tar -I 'zstd --ultra -22' -cf ../assets.tar.zst *.*
