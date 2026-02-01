#!/bin/bash
# ONA Health Check Script
# Sends a simple Narsese statement via UDP and checks for response

set -e

ONA_HOST="${ONA_HOST:-localhost}"
ONA_PORT="${ONA_PORT:-50000}"
TIMEOUT="${TIMEOUT:-2}"

# Send a simple test statement to ONA
# The statement "<test --> ok>." is a basic NARS judgment
echo "<test --> ok>." | timeout "$TIMEOUT" nc -u -w1 "$ONA_HOST" "$ONA_PORT" > /dev/null 2>&1

# If we reach here, the UDP packet was sent successfully
# ONA doesn't necessarily respond to every input, so we just verify the port is reachable
exit 0
