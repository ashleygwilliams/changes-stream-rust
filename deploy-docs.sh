#!/bin/bash

set -o errexit -o nounset

rev=$(git rev-parse --short HEAD)

cargo doc
cd target/doc
echo "<meta http-equiv=refresh content=0;url=`echo $TRAVIS_REPO_SLUG | cut -d '/' -f 2`/index.html>" > target/doc/index.html

git init
git config user.name "Ashley Williams"
git config user.email "ashley666ashley@gmail.com"

git remote add upstream "https://$GH_TOKEN@github.com/ashleygwilliams/changes-stream-rust.git"
git fetch upstream
git reset upstream/gh-pages

touch .

git add -A .
git commit -m "rebuild pages at ${rev}"
git push -q upstream HEAD:gh-pages
