#!/bin/bash

# Create necessary directories for runtime files
mkdir -p /tmp/slurmctld /tmp/slurm /tmp/run
chown -R slurm:slurm /tmp/slurmctld /tmp/slurm /tmp/run
chmod 700 /tmp/slurmctld /tmp/slurm /tmp/run

# Start slurmctld as the slurm user
/usr/sbin/slurmctld -D
