#!/bin/bash
set -ex

apt-get update
apt-get install -y curl build-essential libssl-dev pkg-config
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup.sh
chmod ugo+x rustup.sh
./rustup.sh -y
source $HOME/.cargo/env
cargo install cargo-deb
cargo deb
./builddist.sh