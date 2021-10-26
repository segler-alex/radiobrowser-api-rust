#!/bin/bash

set -e

sudo mkdir -p /usr/bin
sudo mkdir -p /usr/share/radiobrowser
sudo mkdir -p /var/log/radiobrowser

sudo cp target/release/radiobrowser-api-rust /usr/bin/radiobrowser
sudo cp init/radiobrowser.service /etc/systemd/system
sudo cp static/* /usr/share/radiobrowser/
sudo cp etc/config-example.toml /etc/radiobrowser/config-example.toml
sudo cp etc/language-replace.csv /etc/radiobrowser/language-replace.csv
sudo cp etc/language-to-code.csv /etc/radiobrowser/language-to-code.csv
if [ ! -f /etc/radiobrowser/config.toml ]; then
    sudo cp etc/config-example.toml /etc/radiobrowser/config.toml
fi
sudo cp etc/logrotate /etc/logrotate.d/radiobrowser

sudo chmod ugo+x /usr/bin/radiobrowser
sudo groupadd --system radiobrowser
sudo useradd --system --no-create-home --home-dir /var/lib/radiobrowser --gid radiobrowser radiobrowser

# Create log dir
sudo mkdir -p /var/log/radiobrowser
sudo chown radiobrowser:radiobrowser /var/log/radiobrowser

# Create home dir
sudo mkdir -p /var/lib/radiobrowser
sudo chown radiobrowser:radiobrowser /var/lib/radiobrowser

sudo systemctl daemon-reload

echo "Enable service with:"
echo " - systemctl enable radiobrowser"
echo "Start service with:"
echo " - systemctl start radiobrowser"
echo "Logs:"
echo " - journalctl -fu radiobrowser"
echo "Edit /etc/radiobrowser/config.toml to according to your needs."