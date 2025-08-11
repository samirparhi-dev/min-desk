#!/bin/bash
#
# open-vnc-client.sh - Opens VNC connection to min-desk container
#
# This script opens the macOS VNC client and connects to the min-desk
# container running a VNC server on port 5900.
#

echo "Opening VNC connection to min-desk container..."

# Check if Docker container is running
if ! docker ps | grep -q min-desk; then
  echo "Error: min-desk container doesn't appear to be running."
  echo "Please start the container first with: docker-compose up -d"
  exit 1
fi

# Open VNC client with the correct URL
open vnc://localhost:5900

echo "VNC client launched. If you're prompted for a password, leave it blank as authentication is disabled."
echo "If the VNC client didn't open automatically, please run: open vnc://localhost:5900"
