#!/bin/bash
set -e

# Start munged as user munge
echo "Starting munged as user 'munge'..."
su -s /bin/bash -c "/usr/sbin/munged --foreground --verbose" munge &

# Wait for munged to start
sleep 2

# Check if munged is running
if ! pgrep -u munge munged > /dev/null; then
    echo "Failed to start munged. Exiting."
    exit 1
fi

echo "munged started successfully."

# Start slurmd
echo "Starting slurmd..."
exec /usr/sbin/slurmd -D

