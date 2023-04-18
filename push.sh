#!/bin/bash
set -e #kill script if error occurs

cargo test

bash build.sh release

echo "rsyncing"
# rsync -av --delete --info=progress2 tmp/ rpi-public:/mnt/hdd/http_share/ytbn/
rsync -av --delete --info=progress2 --exclude=".git" tmp/ ../titaniumtown.github.io/
rm -fr tmp
cd ../titaniumtown.github.io
git add .
git commit -m "update" | true
git push
