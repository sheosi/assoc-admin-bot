#!/bin/sh
set -e

# Run database migrations using Atlas CLI
# The binary contains embedded migrations and will extract/run them

/assoc-admin-bot --migrate

# Start the application
exec /assoc-admin-bot
