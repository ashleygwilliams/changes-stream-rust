#!/bin/bash

set -o errexit -o nounset

rev=$(git rev-parse --short HEAD)

cargo doc

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
