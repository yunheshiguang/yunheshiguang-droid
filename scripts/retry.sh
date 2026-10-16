#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 3 ]; then
  echo "usage: $0 <attempts> <first wait in seconds> <command> [argument...]" >&2
  exit 2
fi
case "$1:$2" in
  *[!0-9:]* | :* | *:)
    echo "$0: the attempts and the wait must be whole numbers" >&2
    exit 2
    ;;
esac
attempts=$((10#$1))
wait=$((10#$2))
shift 2

attempt=1
until "$@"; do
  status=$?
  if [ "$attempt" -ge "$attempts" ]; then
    echo "$1 failed with exit code $status in all $attempts attempts." >&2
    exit "$status"
  fi
  echo "$1 failed with exit code $status (attempt $attempt of $attempts); retrying in $wait s." >&2
  sleep "$wait"
  attempt=$((attempt + 1))
  wait=$((wait * 2))
done
