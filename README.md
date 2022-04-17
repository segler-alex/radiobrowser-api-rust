[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](code_of_conduct.md) 
# radiobrowser-api-rust

## What is radiobrowser-api-rust?
In short it is an API for an index of web streams (audio and video). Streams can be added and searched by any user of the API.

There is an official deployment of this software that is also freely usable at https://api.radio-browser.info

## Features
* Open source
* Freely licensed
* Well documented API
* Automatic regular online checking of streams
* Highliy configurable
* Easy setup for multiple configurations (native, deb-packages, docker, ansible)
* Implemented in Rust-lang
* Multiple request types: query, json, x-www-form-urlencoded, form-data
* Multiple output types: xml, json, m3u, pls, xspf, ttl, csv
* Optional: multi-server setup with automatic mirroring
* Optional: response caching in internal or external cache (redis, memcached)

## Setup

You can do a native setup or a docker setup

### easy all-in-one docker setup with automatic TLS from Let's Encrypt

#### install

* This has been tested on Ubuntu 20.04 LTS
* Automatic redirect from HTTP to HTTPS
* Automatic generation and update of Let's Encrypt certificates
* Automatic start on reboot
* Automatic fetch of station changes and check information from main server
* A+ rating on https://www.ssllabs.com/ssltest/

```bash
# create radiobrowser directory
mkdir -p /srv/radiobrowser
cd /srv/radiobrowser
# download docker-compose file
wget https://raw.githubusercontent.com/segler-alex/radiobrowser-api-rust/stable/docker-compose-traefik.yml -O docker-compose-traefik.yml
wget https://raw.githubusercontent.com/segler-alex/radiobrowser-api-rust/stable/traefik-dyn-config.toml -O traefik-dyn-config.toml
# create database save directory
mkdir -p dbdata
# create ssl certificate cache file
touch acme.json
chmod 0600 acme.json
# install docker (ubuntu)
apt install -qy docker.io
docker swarm init
# set email and domain, they are needed for automatic certificate generation and for the reverse proxy that is included in the package
export SOURCE="my.domain.org"
export EMAIL="mymail@mail.com"
# OPTIONAL: enable checking of stations, this does check all stations once every 24 hours
export ENABLE_CHECK="true"
# deploy app stack
docker stack deploy -c docker-compose-traefik.yml rb

# For security reasons you should also install fail2ban to secure SSH and enable a firewall
apt-get install fail2ban -y
ufw allow ssh
ufw allow http
ufw allow https
ufw enable
```

#### upgrade

```bash
# update compose file
cd /srv/radiobrowser
wget https://raw.githubusercontent.com/segler-alex/radiobrowser-api-rust/stable/docker-compose-traefik.yml -O docker-compose-traefik.yml
wget https://raw.githubusercontent.com/segler-alex/radiobrowser-api-rust/stable/traefik-dyn-config.toml -O traefik-dyn-config.toml
# set email and domain, they are needed for automatic certificate generation and for the reverse proxy that is included in the package
export SOURCE="my.domain.org"
export EMAIL="mymail@mail.com"
# OPTIONAL: enable checking of stations, this does check all stations once every 24 hours
export ENABLE_CHECK="true"
# deploy app stack, old versions will automatically be upgraded
docker stack deploy -c docker-compose-traefik.yml rb
```

### install from distribution package

* download latest tar.gz package (<https://github.com/segler-alex/radiobrowser-api-rust/releases>)
* untar
* configure mysql/mariadb on your machine
* create database and database user
* call install script
* change /etc/radiobrowser/config.toml if needed
* start systemd service

```bash
# download distribution
mkdir -p radiobrowser
cd radiobrowser
wget https://github.com/segler-alex/radiobrowser-api-rust/releases/download/0.7.23/radiobrowser-dist.tar.gz
tar -zxf radiobrowser-dist.tar.gz

# config database
sudo apt install default-mysql-server
cat init.sql | mysql

# install
./install.sh
sudo systemctl enable radiobrowser
sudo systemctl start radiobrowser
```

### debian/ubuntu package

* download latest deb package (<https://github.com/segler-alex/radiobrowser-api-rust/releases>)
* install it
* configure mysql/mariadb
* create database and database user

```bash
wget https://github.com/segler-alex/radiobrowser-api-rust/releases/download/0.7.23/radiobrowser-api-rust_0.7.23_amd64.deb
sudo apt install default-mysql-server
sudo dpkg -i radiobrowser-api-rust_0.7.23_amd64.deb
cat /usr/share/radiobrowser/init.sql | mysql
```

### native setup from source

Requirements:

* rust, cargo (<http://www.rustup.sh)>
* mariadb or mysql

```bash
# install packages (ubuntu 18.04)
curl https://sh.rustup.rs -sSf | sh
sudo apt install libssl-dev pkg-config gcc
sudo apt install default-mysql-server
```

```bash
# clone repository
git clone https://github.com/segler-alex/radiobrowser-api-rust ~/radio

# setup database, compile, install
cd ~/radio
cat init.sql | mysql
bash install_from_source.sh

# test it
xdg-open http://localhost/webservice/xml/countries
# or just open the link with your favourite browser
```

### docker setup

```bash
# start db and api server
docker-compose up --abort-on-container-exit
```

### docker from registry
The repository is at https://hub.docker.com/repository/docker/segleralex/radiobrowser-api-rust
```bash
# !!! DO NOT USE THE FOLLOWING FOR PRODUCTION !!!
# It is just for a quickstart and is a minimal setup.

# create virtual network for communication between database and backend
docker network create rbnet
# start database container
docker run \
    --name dbserver \
    --detach \
    --network rbnet \
    --rm \
    -e MYSQL_DATABASE=radio \
    -e MYSQL_USER=radiouser \
    -e MYSQL_PASSWORD=password \
    -e MYSQL_RANDOM_ROOT_PASSWORD=true \
    -p 3306:3306 \
    mariadb --character-set-server=utf8mb4 --collation-server=utf8mb4_unicode_ci
# start radiobrowser container
docker pull segleralex/radiobrowser-api-rust:0.7.23
docker run \
    --name radiobrowserapi \
    --detach \
    --network rbnet \
    --rm \
    -e DATABASE_URL=mysql://radiouser:password@dbserver/radio \
    -e HOST=0.0.0.0 \
    -p 8080:8080 \
    segleralex/radiobrowser-api-rust:0.7.23 radiobrowser-api-rust -vvv
# show logs
docker logs -f radiobrowserapi
# access api with the following link
# http://localhost:8080
# stop radiobrowser container
docker rm -f radiobrowserapi
# stop database container
docker rm -f dbserver
# remove the virtual network
docker network rm rbnet
```

### SSL

Radiobrowser does not yet support connecting with https to it directly. You have to add a reverse proxy like Apache or Nginx.

#### Apache config

```bash
# install packages (ubuntu 18.04)
sudo apt install apache2

# enable apache modules
sudo a2enmod proxy_http
sudo systemctl restart apache2
sudo mkdir -p  /var/www/radio
```

Apache config file example

```apache
<VirtualHost *:80>
    ServerName my.servername.com

    ServerAdmin webmaster@programmierecke.net
    DocumentRoot /var/www/radio

    ErrorLog ${APACHE_LOG_DIR}/error.radio.log
    CustomLog ${APACHE_LOG_DIR}/access.radio.log combined

    ProxyPass "/"  "http://localhost:8080/"
    ProxyPassReverse "/"  "http://localhost:8080/"

    <Directory /var/www/radio/>
        AllowOverride All
        Order allow,deny
        allow from all
    </Directory>
</VirtualHost>
```

Follow this guide to get a free certificate
<https://certbot.eff.org/>

### Ansible role

```bash
# clone this project
git clone https://github.com/segler-alex/radiobrowser-api-rust.git
cd radiobrowser-api-rust
# checkout stable
git checkout stable
# deploy, change email adress, for ssl with certbot
ansible-playbook -e "email=test@example.com" -e "version=0.7.23" -e "ansible_python_interpreter=auto" -i "test.example.com,test2.example.com" ansible/playbook.yml
```

## Building

### Distribution tar.gz

```bash
./builddist.sh
```

### Debian/Ubuntu package

```bash
cargo install cargo-deb
cargo deb # run this in your Cargo project directory
```

### With docker
Generate deb and tar.gz distribution with the help of docker. This has the following upsides:
* platform independent builds
* clean builds

```bash
docker run -w /root -v $(pwd):/root ubuntu:bionic bash build_with_docker.sh
```

### Kubernetes helm chart
There is a helm chart at https://gitlab.com/radiobrowser/helm-charts

## Development

### Run a test environment in multiple shells

```bash
# 1.Shell: start db
docker run -e MYSQL_DATABASE=radio -e MYSQL_USER=radiouser -e MYSQL_PASSWORD=password -e MYSQL_RANDOM_ROOT_PASSWORD=true -p 3306:3306 --rm --name dbserver mariadb --character-set-server=utf8mb4 --collation-server=utf8mb4_unicode_ci
# 2.Shell: start radiobrowser with local config
cargo run -- -f radiobrowser-dev.toml

# 3.Shell IF NEEDED: check content of database directly
docker exec -it dbserver bash
mysql -D radio -u radiouser -ppassword
```

## Environment vars / parameters
A print of the help page of the main executable. You can see parameters here. Many of them may also be set by environment variable (env: ..).
```
$ radiobrowser-api-rust --help
radiobrowser-api-rust 0.7.23
segler_alex@web.de
HTTP Rest API for radiobrowser

USAGE:
    radiobrowser-api-rust [FLAGS] [OPTIONS]

FLAGS:
        --help       Prints help information
    -v, --verbose    increases the log level. can be specified mutliple times 0..3
    -V, --version    Prints version information

OPTIONS:
    -a, --allow-database-downgrade <ALLOW_DATABASE_DOWNGRADE>
            allows downgrade of database if tables were created with newer software version

        --broken_stations_never_working_timeout <BROKEN_STATIONS_NEVER_WORKING_TIMEOUT>
            Broken streams are removed after this timespan, if they have never worked. [env:
            BROKEN_STATIONS_NEVER_WORKING_TIMEOUT=]
        --broken_stations_timeout <BROKEN_STATIONS_TIMEOUT>
            Broken streams are removed after this timespan. [env: BROKEN_STATIONS_TIMEOUT=]

        --cache-ttl <DURATION>
            time to life for cache items [env: CACHETTL=]

        --cache-type <CACHETYPE>
            one of none,builtin,redis,memcached [env: CACHETYPE=]

        --cache-url <URL>
            url to access cache server [env: CACHEURL=]

        --checks_timeout <CHECKS_TIMEOUT>
            Checks are removed after this timespan. [env: CHECKS_TIMEOUT=]

        --chunk-size-changes <CHUNK_SIZE_CHANGES>
            chunk size for downloading changes [env: CHUNK_SIZE_CHANGES=]

        --chunk-size-checks <CHUNK_SIZE_CHECKS>
            chunk size for downloading checks [env: CHUNK_SIZE_CHECKS=]

        --click_valid_timeout <CLICK_VALID_TIMEOUT>
            Possible clicks from the same IP. IPs are removed after this timespan. [env: CLICK_VALID_TIMEOUT=]

        --clicks_timeout <CLICKS_TIMEOUT>
            Clicks are removed after this timespan. [env: CLICKS_TIMEOUT=]

    -c, --concurrency <CONCURRENCY>
            streams checked in parallel [env: CONCURRENCY=]

    -f, --config-file <CONFIG-FILE>
            Path to config file [env: CONFIG_FILE=]  [default: /etc/radiobrowser.toml]

    -d, --database <DATABASE_URL>
            Database connection url [env: DATABASE_URL=]

    -x, --delete <DELETE>
            delete broken stations according to rules [env: DELETE=]

        --enable-check <ENABLE_CHECK>
            enable station checks [env: ENABLE_CHECK=]

        --favicon <FAVICON>
            check favicons and try to repair them [env: FAVICON=]

    -i, --ignore-migration-errors <IGNORE_MIGRATION_ERRORS>                                ignore errors in migrations
    -h, --host <HOST>
            listening host ip [env: HOST=]

    -p, --port <PORT>                                                                      listening port [env: PORT=]
    -l, --log-dir <LOG-DIR>
            Path to log dir [env: LOG_DIR=]

    -j, --log-json <LOG_JSON>                                                              Log in JSON format
        --max_depth <MAX_DEPTH>
            max recursive link check depth [env: MAX_DEPTH=]

    -m, --mirror <MIRROR>...
            address of other radiobrowser server to pull updates from

    -q, --mirror-pull-interval <MIRROR_PULL_INTERVAL>
            pull from mirrors at an interval [env: MIRROR_PULL_INTERVAL=]

        --pause <PAUSE>
            database check pauses [env: PAUSE=]

    -e, --prometheus-exporter <PROMETHEUS_EXPORTER>
            export statistics through a prometheus compatible exporter

        --prometheus-exporter-prefix <PROMETHEUS_EXPORTER_PREFIX>
            prefix for all exported values on /metrics

    -r, --retries <RETRIES>
            Max number of retries for station checks [env: RETRIES=]

        --server-country-code <server-country-code>
            2 letter country code for server location [env: SERVERCOUNTRYCODE=]

        --server-location <server-location>
            freeform location server string [env: SERVERLOCATION=]

    -s, --server-url <SERVER_URL>
            full server url that should be used in docs [env: SERVER_URL=]

        --source <SOURCE>
            Source string for database check entries [env: SOURCE=]

    -g, --static-files-dir <STATIC_FILES_DIR>
            directory that contains the static files [env: STATIC_FILES_DIR=]

    -n, --stations <STATIONS>
            batch size for station checks [env: STATIONS=]

        --tcp_timeout <TCP_TIMEOUT>
            tcp connect/read timeout [env: TCP_TIMEOUT=]

    -t, --threads <THREADS>
            concurrent threads used by socket [env: THREADS=]

    -u, --update-caches-interval <UPDATE_CACHES_INTERVAL>
            update caches at an interval [env: UPDATE_CACHES_INTERVAL=]

        --useragent <USERAGENT>
            user agent value for http requests [env: USERAGENT=]
```