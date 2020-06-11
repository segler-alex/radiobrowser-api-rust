# radiobrowser-api-rust

This is the radio browser server part providing the API on the second level servers for <http://www.radio-browser.info>

The server addresses are:

* <https://de1.api.radio-browser.info>
* <https://fr1.api.radio-browser.info>
* <https://nl1.api.radio-browser.info>

The main server on <http://www.radio-browser.info> still has an older version running which will not be upgraded anymore.

Send me feature requests, bug reports or extend it yourself. I license it freely, you could also start your own server if you wish.

You can find the API documentation on <https://api.radio-browser.info/>

## What is radiobrowser-api-rust?
In short it is an API for an index of web streams (audio and video). Streams can be added and searched by any user of the API.

## Features
* Open source
* Freely licensed
* Well documented API
* Automatic regular online checking of streams
* Highliy configurable
* Easy setup for multiple configurations (native, deb-packages, docker, ansible)
* Implemented in Rust-lang
* Optional: multi-server setup with automatic mirroring
* Optional: response caching in internal or external cache (redis, memcached)

## Setup

You can do a native setup or a docker setup

### easy all-in-one docker setup with automatic TLS from Let's Encrypt

#### install

* This has been tested on Ubuntu 18.04
* Automatic redirect from HTTP to HTTPS
* Automatic generation and update of Let's Encrypt certificates
* Automatic start on reboot
* Automatic fetch of station changes and check information from main server

```bash
# checkout this project
git clone https://github.com/segler-alex/radiobrowser-api-rust.git
cd radiobrowser-api-rust
git checkout stable
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
```

#### upgrade

```bash
# checkout this project
cd radiobrowser-api-rust
git checkout stable
git pull
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
wget https://github.com/segler-alex/radiobrowser-api-rust/releases/download/0.6.16/radiobrowser-dist.tar.gz
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
wget https://github.com/segler-alex/radiobrowser-api-rust/releases/download/0.6.16/radiobrowser-api-rust_0.6.16_amd64.deb
sudo apt install default-mysql-server
sudo dpkg -i radiobrowser-api-rust_0.6.16_amd64.deb
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
ansible-playbook -e "email=test@example.com" -e "version=0.6.16" -e "ansible_python_interpreter=auto" -i "test.example.com,test2.example.com" ansible/playbook.yml
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