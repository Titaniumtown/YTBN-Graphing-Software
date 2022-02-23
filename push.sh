#!/bin/bash
set -e #kill script if error occurs

wasm-pack build --target web --release

rm -fr tmp | true #delete tmp folder if exists
mkdir tmp tmp/pkg
cp -r pkg/integral_site_bg.wasm pkg/integral_site_bg.wasm.d.ts pkg/integral_site.d.ts pkg/integral_site.js tmp/pkg/
cp www/bootstrap.js www/index.html www/index.js www/style.css tmp/
# mv tmp/www/index.html tmp/

sed -i 's/\.\.\/pkg/\.\/pkg/g' tmp/bootstrap.js

# put the commit at the beginning of index.html
sed -i '1i <!-- Commit: $(git rev-parse HEAD) -->' tmp/index.html

echo "rsyncing"
rsync -av --delete --info=progress2 tmp/ rpi-public:/mnt/hdd/http_share/integral-demo/
rsync -av --delete --info=progress2 --exclude=".git" tmp/ ../integral-demo/
rm -fr tmp
cd ../integral-demo
git add .
git commit -m "update"
git push