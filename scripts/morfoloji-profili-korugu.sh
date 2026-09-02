#!/usr/bin/env bash
set -euo pipefail

betik_dizini="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
exec "$betik_dizini/conformance-korugu.sh" "$@"
