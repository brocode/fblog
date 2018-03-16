#!/bin/bash
set -x -e -o pipefail

rm -rf target
if [ "$TRAVIS" = true ]; then
    chmod -R 777 .
fi
docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder:1.24.0 cargo build --release
name=fblog
target_dir="target/x86_64-unknown-linux-musl/release"

version=$($target_dir/$name --version | cut -d ' ' -f 2)

docker run -v $(readlink -e ./$target_dir):/release -it --rm  alanfranz/fpm-within-docker:centos-7     fpm -s dir -t rpm -n $name -p /release/$name.rpm -v $version /release/$name=/usr/bin/$name
docker run -v $(readlink -e ./$target_dir):/release -it --rm  alanfranz/fpm-within-docker:ubuntu-zesty fpm -s dir -t deb -n $name -p /release/$name.deb -v $version /release/$name=/usr/bin/$name

