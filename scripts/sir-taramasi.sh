#!/usr/bin/env bash
# K-184: Yerelde ve CI'da aynı sır taraması; değerler çıktıda maskelenir.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
command -v gitleaks >/dev/null || { echo 'Gitleaks 8.30.1 PATH içinde gerekli.' >&2; exit 2; }
[[ "$(gitleaks version)" == '8.30.1' ]] || { echo 'Beklenen Gitleaks sürümü: 8.30.1' >&2; exit 2; }
umask 077
zee_scan_tmp="$(mktemp -d)"
trap 'rm -rf "$zee_scan_tmp"' EXIT
printf '[extend]\nuseDefault = true\n' > "$zee_scan_tmp/default.toml"
zee_scan_flags=(--redact --no-banner --ignore-gitleaks-allow --gitleaks-ignore-path "$zee_scan_tmp/ignore" --config "$zee_scan_tmp/default.toml")
touch "$zee_scan_tmp/ignore"
if [[ "${1:-}" == '--staged' ]]; then
    git diff --cached --no-ext-diff --binary | gitleaks stdin "${zee_scan_flags[@]}"
elif [[ $# == 0 ]]; then
    gitleaks git "${zee_scan_flags[@]}" --log-opts=--all .
    # Çalışma ağacındaki izlenen dosyalar ve yeni, ignore edilmeyen dosyalar.
    python3 - "$zee_scan_tmp/tree" <<'PY'
from pathlib import Path
import subprocess, sys, shutil
out = Path(sys.argv[1]); out.mkdir()
paths = subprocess.check_output(['git', 'ls-files', '-z', '--cached', '--others', '--exclude-standard'])
for name in paths.split(b'\0'):
    if not name: continue
    src = Path(name.decode())
    if src.is_file() and not src.is_symlink():
        dst = out / src; dst.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(src, dst)
PY
    gitleaks dir "${zee_scan_flags[@]}" "$zee_scan_tmp/tree"
else
    echo 'Kullanım: bash scripts/sir-taramasi.sh [--staged]' >&2; exit 2
fi
