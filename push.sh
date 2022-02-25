#!/bin/bash
set -e #kill script if error occurs

bash build.sh

echo "rsyncing"
rsync -av --delete --info=progress2 tmp/ rpi-public:/mnt/hdd/http_share/integral-demo/
rsync -av --delete --info=progress2 --exclude=".git" tmp/ ../integral-demo/
rm -fr tmp
cd ../integral-demo
git add .
git commit -m "update"
git push