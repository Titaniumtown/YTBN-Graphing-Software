#!/bin/bash
rm -fr data.tar.zst | true
tar -I 'zstd --ultra -22' -cf data.tar.zst assets/*.*