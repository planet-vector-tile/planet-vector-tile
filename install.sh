#!/bin/bash -x

cargo install --path .
npm i
npm run build
pushd maplibre-gl-js
npm i
npm run build-dev
popd
pushd desktop
npm i
popd
