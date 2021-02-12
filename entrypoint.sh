#!/bin/bash
set -e
echo "$@"

exec /app/mongodb-poster "$@"
