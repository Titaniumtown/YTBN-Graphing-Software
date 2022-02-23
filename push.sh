#!/bin/bash
set -e #kill script if error occurs

wasm-pack build --target web --release

rm -fr tmp | true #delete tmp folder if exists
mkdir tmp tmp/pkg
cp -r pkg/integral_site_bg.wasm pkg/integral_site_bg.wasm.d.ts pkg/integral_site.d.ts pkg/integral_site.js tmp/pkg/
cp www/index.html www/style.css tmp/

sed -i 's/\.\.\/pkg/\.\/pkg/g' tmp/index.html

# put the commit at the beginning of index.html
mv tmp/index.html tmp/index.html.bak
commit_comment="<!-- Commit: $(git rev-parse HEAD) -->"
echo $commit_comment > tmp/index.html
cat tmp/index.html.bak >> tmp/index.html
rm -fr tmp/index.html.bak

echo "rsyncing"
rsync -av --delete --info=progress2 tmp/ rpi-public:/mnt/hdd/http_share/integral-demo/
rsync -av --delete --info=progress2 --exclude=".git" tmp/ ../integral-demo/
rm -fr tmp
cd ../integral-demo
git add .
git commit -m "update"
git push