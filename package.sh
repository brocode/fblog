#!/bin/bash
set -x -e -o pipefail

cargo build --release
name=fblog
version=$(target/release/$name --version | cut -d ' ' -f 2)

rm -f target/release/$name.rpm
docker run -v $(readlink -e ./target/release):/release -it --rm  alanfranz/fpm-within-docker:centos-7     fpm -s dir -t rpm -n $name -p /release/$name.rpm -v $version /release/$name=/usr/bin/$name
rm -f target/release/$name.deb
docker run -v $(readlink -e ./target/release):/release -it --rm  alanfranz/fpm-within-docker:ubuntu-zesty fpm -s dir -t deb -n $name -p /release/$name.deb -v $version /release/$name=/usr/bin/$name

