#!/bin/bash
sudo rm /usr/bin/radiobrowser
sudo userdel radiobrowser
sudo groupdel radiobrowser

sudo rm /etc/systemd/system/radiobrowser.service
sudo systemctl daemon-reload
sudo rm -rf /usr/share/radiobrowser
sudo rm -rf /var/log/radiobrowser

sudo rm /etc/logrotate.d/radiobrowser