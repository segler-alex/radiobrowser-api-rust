#!/bin/bash
sudo rm /usr/local/bin/radiobrowser
sudo userdel radiobrowser
sudo groupdel radiobrowser

sudo rm /etc/systemd/system/radiobrowser.service
sudo systemctl daemon-reload
sudo rm -rf /usr/local/share/radiobrowser