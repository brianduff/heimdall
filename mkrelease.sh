#!/bin/sh

version=$1

if [ -z "$version" ]
then
  echo "Must specify a version"
  exit 1
else
  rm -rf release/$version
  mkdir -p release/$version
  cargo build --release
  cp target/release/heimdall release/$version/
  strip release/$version/heimdall
  cp install.sh release/$version/
  cp -r static release/$version/
fi

tar -cvf release/heimdall-$version.tar.gz -C release/$version .
shasum -a 256 release/heimdall-$version.tar.gz
