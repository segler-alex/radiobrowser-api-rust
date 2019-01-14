#!/bin/bash
cargo build --release
sudo mkdir -p /usr/local/bin
sudo mkdir -p /usr/local/share/radiobrowser

sudo cp target/release/radiobrowser-api-rust /usr/local/bin/radiobrowser.bin
sudo cp init/radiobrowser.service /etc/systemd/system
sudo cp static/* /usr/local/share/radiobrowser/

cat <<EOF | sudo tee /usr/local/bin/radiobrowser
#!/bin/bash
. /etc/radiobrowser.conf
export HOST
export PORT
export SERVER_URL
export THREADS
export UPDATE_CACHES_INTERVAL
export DATABASE_URL
export STATIC_FILES_DIR
export MIRROR_PULL_INTERVAL
radiobrowser.bin $${PARAMS}
EOF

CONFIG_FILE="/etc/radiobrowser.conf"
if [ ! -f "${CONFIG_FILE}" ]; then
    cat <<EOF | sudo tee /etc/radiobrowser.conf
# Listening IP
HOST=127.0.0.1
# Listening port
PORT=8080
# Documentation url to be used if automatic is not working
SERVER_URL=https://api.radio-browser.info
# How many concurrent threads used by socket
THREADS=5
# Update caches at an interval in seconds
UPDATE_CACHES_INTERVAL=30
# database connection string (mysql, mariadb)
DATABASE_URL=mysql://radiouser:password@localhost/radio
# Directory for static and template files
STATIC_FILES_DIR=/usr/local/share/radiobrowser
# Mirror pull interval in seconds
MIRROR_PULL_INTERVAL=600
# Mirror from server
PARAMS="-m http://www.radio-browser.info/webservice"
EOF
fi

sudo chmod ugo+x /usr/local/bin/radiobrowser
sudo groupadd --system radiobrowser
sudo useradd --system --no-create-home --gid radiobrowser radiobrowser

sudo systemctl daemon-reload

echo "Enable service with:"
echo " - systemctl enable radiobrowser"
echo "Start service with:"
echo " - systemctl start radiobrowser"
echo "Logs:"
echo " - journalctl -fu radiobrowser"
echo "Edit /etc/radiobrowser.conf according to your needs."