#!/bin/bash -x

cargo install --path .
npm i
npm run build
pushd maplibre-gl-js
npm i
npm run build-dist
popd
pushd electron
npm i
popd