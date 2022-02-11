#!/bin/bash
set -e #kill script if error occurs

wasm-pack build --target web --release

rm -fr tmp | true #delete tmp folder if exists
mkdir tmp tmp/pkg tmp/www
cp -r pkg/integral_site_bg.wasm pkg/integral_site_bg.wasm.d.ts pkg/integral_site.d.ts pkg/integral_site.js tmp/pkg/
cp www/bootstrap.js www/index.html www/index.js www/style.css tmp/www/
mv tmp/www/index.html tmp/

sed -i 's/style.css/www\/style.css/g' tmp/index.html
sed -i 's/bootstrap.js/www\/bootstrap.js/g' tmp/index.html


echo "rsyncing"
rsync -av --delete --info=progress2 tmp/ rpi-public:/mnt/hdd/http_share/integrals/
rm -fr tmp