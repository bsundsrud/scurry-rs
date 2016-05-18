#!/bin/bash
source pg_options.sh
if [ -z "$DATA_DIR" ]; then
    echo '$DATA_DIR must be set!'
    exit 1
fi

./stop_postgres.sh
sleep 2
rm -rf "$DATA_DIR"
