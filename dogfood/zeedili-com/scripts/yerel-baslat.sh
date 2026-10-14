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

# Python sahibi WNOWAIT ile lider PID'lerini grup temizliği bitene kadar tutar.
# Bash arkaplan çocuklarını wait'ten önce reap edebildiği için sayısal PID trap'i kullanılmaz.
exec python3 -I -B - "$DIL_BIN" "$SITE_KOKU" "$BETIK_KOKU/yerel-onkapi.mjs" "$DEPO_KOKU/scripts/worker-supervisor.py" <<'PYTHON'
import importlib.util
import os
from pathlib import Path
import signal
import sys
import time
from types import SimpleNamespace

if os.name != "posix" or not all(hasattr(os, name) for name in ("waitid", "WNOWAIT", "WEXITED", "P_PID")):
    print("Yönetilen yerel önizleme bu hostta desteklenmiyor.", file=sys.stderr)
    raise SystemExit(2)
spec = importlib.util.spec_from_file_location("zee_yerel_surec_sahibi", sys.argv[4])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

class SessizMetadata:
    def emit(self, *args, **kwargs):
        pass

owner = module.Supervisor(SimpleNamespace(project=Path(sys.argv[2]), stop_grace=2), SessizMetadata())
exit_request = 0

def interrupted(number, frame):
    global exit_request
    exit_request = 128 + number
    owner.stop = True

for number in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
    signal.signal(number, interrupted)

def wait_status(child, timeout=None):
    deadline = time.monotonic() + timeout if timeout is not None else None
    while not owner.stop:
        status = owner.exit_status(child)
        if status is not None:
            return status
        if deadline is not None and time.monotonic() >= deadline:
            return None
        time.sleep(.02)
    return None

code = 1
try:
    # Yerel prototipin mevcut görünür stdout/stderr ve ortam davranışı korunur.
    worker = owner.spawn([sys.argv[1], "çalıştır", "--deneysel-web", sys.argv[2]],
                         environment=os.environ.copy(), visible=True)
    frontend = None
    for _ in range(40):
        if owner.stop:
            break
        status = owner.exit_status(worker)
        if status is not None:
            code = status if status > 0 else 1
            break
        probe = owner.spawn(["curl", "--fail", "--silent", "--max-time", "1",
                             "http://127.0.0.1:8126/canli"], environment=os.environ.copy())
        status = wait_status(probe, 1.2)
        owner.stop_child(probe)
        if status == 0 and not owner.stop and owner.exit_status(worker) is None:
            frontend = owner.spawn(["node", sys.argv[3]], environment=os.environ.copy(), visible=True)
            break
        time.sleep(.1)
    if frontend is None and not owner.stop:
        print("Zee worker 8126 kapısında hazır olmadı.", file=sys.stderr)
    while frontend is not None and not owner.stop:
        status = owner.exit_status(frontend)
        if status is not None:
            code = status if status >= 0 else 128 - status
            break
        if owner.exit_status(worker) is not None:
            code = 1
            break
        time.sleep(.02)
except (module.Reject, OSError):
    print("Yerel önizleme süreçleri güvenle başlatılamadı.", file=sys.stderr)
    code = 2
finally:
    try:
        owner.cleanup()
    except (module.Reject, OSError):
        print("Yerel önizleme süreç temizliği tamamlanamadı.", file=sys.stderr)
        code = 2
raise SystemExit(exit_request or code)
PYTHON
