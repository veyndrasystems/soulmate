#!/bin/sh
set -u

binary=${SOULMATE_BIN:-soulmate}
exec "$binary" benchmark --json "$@"
