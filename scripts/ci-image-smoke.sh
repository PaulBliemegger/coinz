#!/usr/bin/env bash
set -euo pipefail

# Run only against disposable images/data on a synthetic CI Docker daemon.
image=${1:?Usage: ci-image-smoke.sh IMAGE}
container="coinz-ci-${RANDOM}-${RANDOM}"

cleanup() {
  docker rm --force --volumes "$container" >/dev/null 2>&1 || true
}
trap cleanup EXIT

docker run --detach --name "$container" \
  --network none \
  --read-only --tmpfs /tmp \
  --cap-drop ALL --security-opt no-new-privileges:true \
  --mount type=volume,destination=/data \
  "$image" >/dev/null

wait_for_health() {
  for ((attempt = 0; attempt < 30; attempt++)); do
    state=$(docker inspect --format '{{.State.Status}}' "$container")
    if [[ "$state" != running ]]; then
      echo "Synthetic container stopped during startup." >&2
      docker logs "$container" >&2
      return 1
    fi
    if docker exec "$container" curl --fail --silent \
      http://127.0.0.1:3000/api/health >/dev/null; then
      return 0
    fi
    sleep 2
  done
  echo "Synthetic container did not become ready." >&2
  docker logs "$container" >&2
  return 1
}

wait_for_health
docker exec "$container" curl --fail --silent \
  http://127.0.0.1:3000/ >/dev/null
printf '%s\n' 'synthetic-ci-password' \
  | docker exec --interactive "$container" /app/coinz \
    create-user synthetic-ci --password-stdin
docker restart --time 5 "$container" >/dev/null
wait_for_health
