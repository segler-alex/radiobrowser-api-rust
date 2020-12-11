#!/bin/bash
set -ex

cargo install cargo-deb
cargo deb
./builddist.sh