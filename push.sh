#!/bin/bash
set -e #kill script if error occurs

wasm-pack build --target web --release

rm -fr tmp | true #delete tmp folder if exists
mkdir tmp
cp -r pkg www tmp/
mv tmp/www/index.html tmp/

sed -i 's/style.css/www\/style.css/g' tmp/index.html
sed -i 's/bootstrap.js/www\/bootstrap.js/g' tmp/index.html


echo "rsyncing"
rsync -av --delete --info=progress2 tmp/ rpi-public:/mnt/hdd/http_share/integrals/
rm -fr tmp