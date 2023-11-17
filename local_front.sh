#!/bin/sh

(cd ../korekto-frontend && npm install && npm run build)
rm -rf static
mkdir static
cp -a ../korekto-frontend/build/. static/
cp -a welcome/. static/
