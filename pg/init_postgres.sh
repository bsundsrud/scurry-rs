#!/bin/bash
source pg_options.sh
if [ -z "$DATA_DIR" ]; then
    echo '$DATA_DIR must be set!'
    exit 1
fi
pg_ctl init -D "$DATA_DIR"
./start_postgres.sh
sleep 1
createdb $(whoami)
createdb scurry_test
