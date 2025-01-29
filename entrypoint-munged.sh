#!/bin/bash
set -e

# Ensure proper ownership of /run/munge
chown -R munge:munge /run/munge
chmod 0755 /run/munge

# Start munged
su -s /bin/bash -c "/usr/sbin/munged --foreground --verbose" munge
