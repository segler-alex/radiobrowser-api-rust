#!/bin/bash

DISTDIR=$(pwd)/dist
mkdir -p ${DISTDIR}
mkdir -p ${DISTDIR}/etc
mkdir -p ${DISTDIR}/bin
cargo build --release

cp target/release/radiobrowser-api-rust ${DISTDIR}/bin/radiobrowser
cp -R init ${DISTDIR}/
cp -R static ${DISTDIR}/
cp radiobrowser.toml ${DISTDIR}/etc/config-example.toml
cp install_from_dist.sh ${DISTDIR}/install.sh

tar -czf $(pwd)/radiobrowser-dist.tar.gz -C ${DISTDIR} bin init static etc install.sh