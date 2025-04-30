#!/bin/bash

# Parse the DB URL
DB_URL="postgres://user:password@localhost:5432/db_name"

# Extract components
USER=$(echo $DB_URL | sed -E 's|.*://([^:]+):.*|\1|')
PASSWORD=$(echo $DB_URL | sed -E 's|.*://[^:]+:([^@]+)@.*|\1|')
HOST=$(echo $DB_URL | sed -E 's|.*@([^:/]+):.*|\1|')
PORT=$(echo $DB_URL | sed -E 's|.*:([0-9]+)/.*|\1|')
DB_NAME=$(echo $DB_URL | sed -E 's|.*/([^?]+)|\1|')

# The name of your Docker container running Postgres
CONTAINER_NAME="my-postgres"

echo "Resetting database '$DB_NAME' on container '$CONTAINER_NAME'..."

# Drop and recreate the DB using psql inside the container
docker exec -e PGPASSWORD=$PASSWORD -i $CONTAINER_NAME psql -U $USER -h $HOST -p $PORT -d postgres <<EOF
DROP DATABASE IF EXISTS "$DB_NAME";
CREATE DATABASE "$DB_NAME";
EOF

echo "Database '$DB_NAME' has been reset."

