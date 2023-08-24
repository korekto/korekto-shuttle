#!/bin/sh

rm -rf target/front_git
mkdir -p target/front_git
git clone git@github.com:korekto/korekto-frontend.git target/front_git
(cd target/front_git && npm install && npm run build)
rm -rf static
mkdir static
cp -a target/front_git/build/. static/
cp -a welcome/. static/
