#!/bin/bash
set -e

# Define variables
DOCKERHUB_USERNAME="jsfillman"  # Replace with your Docker Hub username
PLATFORMS="linux/386,linux/amd64,linux/arm64/v8,linux/riscv64,linux/ppc64le,linux/s390x"
VERSION="24.05"  # Update this to match the Slurm version you're building
DOCKERFILES=("slurmctld" "slurmd" "munged")  # Add more Dockerfile names if needed

# Iterate over each Dockerfile
for TARGET in "${DOCKERFILES[@]}"; do
    IMAGE_NAME="${DOCKERHUB_USERNAME}/${TARGET}"

    echo "Building and pushing ${IMAGE_NAME} for platforms: ${PLATFORMS}"

    # Build and push the image using Docker Buildx
    docker buildx build \
        --platform ${PLATFORMS} \
        --file "Dockerfile.${TARGET}" \
        --push \
        --tag "${IMAGE_NAME}:${VERSION}" \
        --tag "${IMAGE_NAME}:latest" \
        --tag "${IMAGE_NAME}:kluge" .

    echo "Successfully built and pushed ${IMAGE_NAME}"
done

echo "All images built and pushed successfully!"
