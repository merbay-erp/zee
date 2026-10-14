#!/usr/bin/env bash
set -euo pipefail

depo_koku="$(git rev-parse --show-toplevel)"
cd "$depo_koku"

# Kirli caller dahil yalnız başlangıçtaki exact commit arşivlenir. Eşzamanlı
# checkout/commit manifest, zaman, isim veya ZIP kaynaklarını değiştiremez.
sha="$(git rev-parse --verify 'HEAD^{commit}')"
surum="$(git describe --tags --always "$sha" 2>/dev/null)"
cikti="${1:-dist/zee-kaynak-${surum}.zip}"

if [[ "$cikti" != /* ]]; then
    cikti="$depo_koku/$cikti"
fi

# Var olmayan ebeveynleri de çöz, fakat kapsam denetlenmeden mkdir yapma.
# Son dosya/sidecar için symlink veya dizin üzerine yayınlama yoktur.
cikti="$(python3 -I - "$depo_koku" "$cikti" "$sha" <<'PY'
from pathlib import Path
import os
import subprocess
import sys
import tempfile
import unicodedata

try:
    root = Path(sys.argv[1]).resolve(strict=True)
    requested = Path(sys.argv[2])
    if requested.name in ('', '.', '..'):
        raise ValueError
    if any(c in str(requested) for c in '\r\n\t'):
        raise ValueError
    if requested.is_symlink():
        raise ValueError
    output = requested.parent.resolve(strict=False) / requested.name
    if output == root or root not in output.parents:
        raise ValueError
    tracked = set()
    for command in (['ls-files', '-z'], ['ls-tree', '-r', '-z', '--name-only', sys.argv[3]]):
        result = subprocess.run(['git', '-C', str(root), *command], capture_output=True, check=True)
        tracked.update(os.fsdecode(name) for name in result.stdout.split(b'\0') if name)
    git_directory = subprocess.run(['git', '-C', str(root), 'rev-parse', '--absolute-git-dir'],
                                   capture_output=True, check=True)
    directory_bytes = git_directory.stdout
    if directory_bytes.endswith(b'\n'):
        directory_bytes = directory_bytes[:-1]
    metadata = Path(os.fsdecode(directory_bytes)).resolve(strict=True)
    potential = {unicodedata.normalize('NFC', name).casefold() for name in tracked}
    for path in (output, Path(str(output) + '.sha256')):
        if path.is_symlink() or (path.exists() and not path.is_file()):
            raise ValueError
        relative = path.relative_to(root)
        if any(part.casefold() == '.git' for part in relative.parts):
            raise ValueError
        if relative.as_posix() in tracked:
            raise ValueError
        for parent in path.parents:
            if parent.exists() and parent.samefile(metadata):
                raise ValueError
        if path.exists():
            for name in tracked:
                try:
                    if path.samefile(root / name):
                        raise ValueError
                except FileNotFoundError:
                    pass
        # Missing tracked files have no inode to compare. Probe local directory
        # name rules only for a possible alias, inside the already-checked root.
        if unicodedata.normalize('NFC', relative.as_posix()).casefold() in potential:
            parent = path.parent
            while not parent.exists():
                parent = parent.parent
            if not parent.is_dir():
                raise ValueError
            descriptor, name = tempfile.mkstemp(prefix='.zee-kaynak-Aa-é-', dir=parent)
            os.close(descriptor)
            probe = Path(name)
            try:
                case_alias = probe.with_name(probe.name.swapcase())
                norm_alias = probe.with_name(unicodedata.normalize('NFD', probe.name))
                insensitive = case_alias.exists() and case_alias.samefile(probe)
                normalizing = norm_alias.exists() and norm_alias.samefile(probe)
                def comparison_key(value):
                    value = unicodedata.normalize('NFC', value) if normalizing else value
                    return value.casefold() if insensitive else value
                if comparison_key(relative.as_posix()) in {comparison_key(value) for value in tracked}:
                    raise ValueError
            finally:
                probe.unlink()
    print(output)
except (OSError, ValueError, RuntimeError, subprocess.SubprocessError):
    print('Arşiv yalnız depo içindeki normal bir çıktı yoluna yazılabilir.', file=sys.stderr)
    sys.exit(2)
PY
)"

mkdir -p "$(dirname "$cikti")"
gecici_klasor="$(mktemp -d "$(dirname "$cikti")/.zee-kaynak-arsivi.XXXXXX")"
gecici="$gecici_klasor/kaynak.zip"
gecici_ozet="$gecici_klasor/kaynak.sha256"
manifest="$gecici_klasor/KAYNAK-SHA256.txt"
trap 'rm -rf "$gecici_klasor"' EXIT

sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum | awk '{print $1}'
    else
        shasum -a 256 | awk '{print $1}'
    fi
}

while IFS= read -r -d '' dosya; do
    if [[ "$dosya" == *$'\n'* ]]; then
        echo "Satır sonu taşıyan Git yolu kaynak manifestinde desteklenmiyor." >&2
        exit 1
    fi
    ozet="$(git show "$sha:$dosya" | sha256)"
    printf '%s  %s\n' "$ozet" "$dosya" >> "$manifest"
done < <(git ls-tree -r -z --name-only "$sha")

commit_zamani="$(git show -s --format=%ct "$sha")"
git archive --format=zip --mtime="@$commit_zamani" --prefix=zee/ \
    --add-file="$manifest" --output="$gecici" "$sha"

if unzip -Z1 "$gecici" | grep -Eq '(^|/)(__MACOSX|target|artifacts|\.git)(/|$)|\.(profraw|profdata)$'; then
    echo "Arşiv yasaklı build/metadata girdisi içeriyor." >&2
    exit 1
fi
if ! unzip -Z1 "$gecici" | grep -qx 'zee/KAYNAK-SHA256.txt'; then
    echo "Arşiv kaynak SHA-256 manifestini taşımıyor." >&2
    exit 1
fi

# K-243: gerçek ZIP/comment/manifest/kaynak hash eşliğini yayınlamadan önce
# aynı exact Git snapshot'a bağla. Uzak CI kaydı/artefakt indirme yapılmaz.
python3 -I - "$depo_koku" "$sha" "$gecici" <<'PY'
import importlib.util
from pathlib import Path
import sys

sys.dont_write_bytecode = True
try:
    repo = Path(sys.argv[1])
    spec = importlib.util.spec_from_file_location('source_verifier', repo / 'scripts/kaynak-ci-esleme.py')
    verifier = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(verifier)
    source = verifier.archive_hashes(repo, sys.argv[2])
    verifier.zip_hashes(Path(sys.argv[3]).read_bytes(), sys.argv[2], source)
except Exception:
    print('Kaynak ZIP/comment/manifest eşliği doğrulanamadı; arşiv yayımlanmadı.', file=sys.stderr)
    sys.exit(1)
PY

arsiv_ozeti="$(sha256 < "$gecici")"
printf '%s  %s\n' "$arsiv_ozeti" "$(basename "$cikti")" > "$gecici_ozet"
mv "$gecici" "$cikti"
mv "$gecici_ozet" "${cikti}.sha256"
rm -rf "$gecici_klasor"
trap - EXIT
echo "Temiz kaynak arşivi: $cikti"
echo "Arşiv özeti: ${cikti}.sha256"
