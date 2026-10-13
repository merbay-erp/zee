#!/usr/bin/env bash
set -euo pipefail

BETIK_KOKU="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SITE_KOKU="$(cd -- "$BETIK_KOKU/.." && pwd)"
DEPO_KOKU="$(cd -- "$SITE_KOKU/../.." && pwd)"
DIL_BIN="${DIL_BIN:-$DEPO_KOKU/compiler/target/release/dil}"

if [[ ! -x "$DIL_BIN" ]]; then
  echo "Zee CLI bulunamadı: $DIL_BIN" >&2
  echo "Önce: cd compiler && cargo build --release --locked --bin dil" >&2
  exit 1
fi

"$DIL_BIN" denetle "$SITE_KOKU"
"$DIL_BIN" dene "$SITE_KOKU"

"$DIL_BIN" çalıştır --deneysel-web "$SITE_KOKU" &
ISCI_PID=$!
temizle() {
  kill "$ISCI_PID" 2>/dev/null || true
}
trap temizle EXIT INT TERM

for _ in {1..40}; do
  if curl --fail --silent --max-time 1 http://127.0.0.1:8126/canli >/dev/null; then
    exec node "$BETIK_KOKU/yerel-onkapi.mjs"
  fi
  sleep 0.1
done

echo "Zee worker 8126 kapısında hazır olmadı." >&2
exit 1
