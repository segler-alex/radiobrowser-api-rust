#!/bin/bash

rm radio.sql.gz
wget http://www.radio-browser.info/backups/latest.sql.gz -O radio.sql.gz
mkdir -p dbdata
docker stack deploy -c docker-compose.yml radiobrowser
