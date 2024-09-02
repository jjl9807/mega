#!/bin/sh

export MEGA_BASE_DIR="/opt/mega"
CONFIG_FILE="$MEGA_BASE_DIR/etc/config.toml"

if [ -f "$CONFIG_FILE" ]; then
    echo "Using config file: $CONFIG_FILE"
    exec /usr/local/bin/aries -c "$CONFIG_FILE"
else
    exec /usr/local/bin/aries 
fi