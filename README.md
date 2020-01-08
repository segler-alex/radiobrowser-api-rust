# radiobrowser-api-rust

This is the radio browser server part providing the API on the second level servers for http://www.radio-browser.info

The server addresses are:
* https://de1.api.radio-browser.info
* https://fr1.api.radio-browser.info
* https://nl1.api.radio-browser.info

The main server on http://www.radio-browser.info still has an older version running which will not be upgraded anymore.

Send me feature requests, bug reports or extend it yourself. I license it freely, you could also start your own server if you wish.

You can find the API documentation on http://www.radio-browser.info/gui/#!/api

## Setup
You can do a native setup or a docker setup

### easy all-in-one docker setup with automatic TLS from Let's Encrypt
* This has been tested on Ubuntu 18.04
* Automatic redirect from HTTP to HTTPS
* Automatic generation and update of Let's Encrypt certificates
* Automatic start on reboot
* Automatic fetch of station changes and check information from main server at http://api.radio-browser.info

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
# set email and domain
export SOURCE="my.domain.org"
export EMAIL="mymail@mail.com"
# OPTIONAL: enable checking of stations, this does check all stations once every 24 hours
export ENABLE_CHECK="true"
# deploy app stack
docker stack deploy -c docker-compose-traefik.yml rb
```

### native setup
Requirements:
* apache
* rust, cargo (http://www.rustup.sh)
* mariadb or mysql

```bash
# install packages (ubuntu 18.04)
curl https://sh.rustup.rs -sSf | sh
sudo apt install apache2 libssl-dev pkg-config gcc
sudo apt install default-mysql-server

# enable apache modules
sudo a2enmod proxy_http
sudo systemctl restart apache2
```

Apache config file example
```
<VirtualHost *:80>
    ServerName www.radio-browser.info

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

```bash
# create database and user
cat /var/www/radio/init.sql | mysql

# clone repository
git clone https://github.com/segler-alex/radiobrowser-api-rust ~/radio

# compile, install
cd ~/radio
bash install.sh

# test it
xdg-open http://localhost/webservice/xml/countries
# or just open the link with your favourite browser
```

If you want to enable SSL, just follow this guide:
https://certbot.eff.org/

### docker setup
```bash
# start db and api server
docker-compose up --abort-on-container-exit
```

### debian package
```bash
cargo install cargo-deb
cargo deb # run this in your Cargo project directory
```
