docker buildx build \
  --platform linux/amd64,linux/arm64,linux/ppc64le,linux/s390x,linux/riscv64 \
  --push \
  --tag jsfillman/slurmd:kluge .

