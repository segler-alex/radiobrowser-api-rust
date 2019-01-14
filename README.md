# radiobrowser-api-rust
This is the radio browser server part, that provides the api on http://www.radio-browser.info

Send me feature requests, bug reports or extend it yourself. I licenced it freely, you could also start your own server if you wish.

You can find the api documentation on http://api.radio-browser.info

## Setup
You can do a native setup or a docker setup

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
sudo a2enmod rewrite headers
```

Apache config file example
```
<VirtualHost *:80>
	ServerName www.radio-browser.info

	ServerAdmin webmaster@programmierecke.net
	DocumentRoot /var/www/radio

	ErrorLog ${APACHE_LOG_DIR}/error.radio.log
	CustomLog ${APACHE_LOG_DIR}/access.radio.log combined

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
git clone https://github.com/segler-alex/radiobrowser-api ~/radio

# compile, install
cd ~/radio
bash install.sh

# test it
xdg-open http://localhost/webservice/xml/countries
# or just open the link with your favourite browser
```

### docker setup
TODO