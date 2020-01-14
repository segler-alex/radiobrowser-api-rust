#!/bin/bash
sudo rm /usr/bin/radiobrowser
sudo userdel radiobrowser
sudo groupdel radiobrowser

sudo rm /etc/systemd/system/radiobrowser.service
sudo systemctl daemon-reload
sudo rm -rf /usr/share/radiobrowser