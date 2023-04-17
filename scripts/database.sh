#!/usr/bin/env bash

set -x
set -eo pipefail

if ! [ -x "$(command -v mycli)" ]; then
    echo "ERROR: mycli command missing";
    exit 1;
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo "ERROR: sqlx command missing";
    exit 1;
fi

DB_HOST="${POSTGRES_HOST:=localhost}"
DB_PORT="${POSTGRES_PORT:=3306}"
DB_USER="${MYSQL_ROOT_USER:=root}"
DB_PASSWORD="${MYSQL_ROOT_PASSWORD:=admin}"
DB_NAME="${MYSQL_DATABASE:=DAIKOKU}"

sudo docker run \
    --name daikoku \
    -e MYSQL_ROOT_PASSWORD=${DB_PASSWORD} \
    -e MYSQL_DATABASE=${DB_NAME} \
    -p "${DB_PORT}":3306 \
    -d mysql:8 || true;

until mycli -h "${DB_HOST}" -u "${DB_USER}" -P "${DB_PORT}" -p "${DB_PASSWORD}" -e "SHOW DATABASES;"; do
    >&2 echo "MySQL is still unavailable"
    sleep 5
done

>&2 echo "MySQL is ready"

export DATABASE_URL=mysql://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}

sqlx database create
sqlx migrate run
