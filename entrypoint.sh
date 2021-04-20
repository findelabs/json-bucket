#!/bin/bash
set -e
echo "$@"

exec /app/json-bucket "$@"
