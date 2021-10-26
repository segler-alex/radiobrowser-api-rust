#!/bin/bash
set -e

DISTDIR=$(pwd)/dist
mkdir -p ${DISTDIR}
mkdir -p ${DISTDIR}/bin
cargo build --release

cp target/release/radiobrowser-api-rust ${DISTDIR}/bin/radiobrowser
mkdir -p ${DISTDIR}/init
cp debian/radiobrowser.service ${DISTDIR}/init/
cp -R static ${DISTDIR}/
cp -R etc ${DISTDIR}/
cp install_from_dist.sh ${DISTDIR}/install.sh

tar -czf $(pwd)/radiobrowser-dist.tar.gz -C ${DISTDIR} bin init static etc install.sh