#!/bin/bash
cargo build --release
sudo mkdir -p /usr/local/bin
sudo mkdir -p /usr/local/share/radiobrowser
sudo mkdir -p /var/log/radiobrowser

sudo cp target/release/radiobrowser-api-rust /usr/local/bin/radiobrowser
sudo cp init/radiobrowser.service /etc/systemd/system
sudo cp static/* /usr/local/share/radiobrowser/
if [ ! -f /etc/radiobrowser.toml ]; then
    sudo cp radiobrowser.toml /etc/radiobrowser.toml
fi

sudo chmod ugo+x /usr/local/bin/radiobrowser
sudo groupadd --system radiobrowser
sudo useradd --system --no-create-home --gid radiobrowser radiobrowser

sudo chown radiobrowser:radiobrowser /var/log/radiobrowser

sudo systemctl daemon-reload

echo "Enable service with:"
echo " - systemctl enable radiobrowser"
echo "Start service with:"
echo " - systemctl start radiobrowser"
echo "Logs:"
echo " - journalctl -fu radiobrowser"
echo "Edit /etc/radiobrowser.toml according to your needs."