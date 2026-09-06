#!/usr/bin/env bash
# K-165/ADR-073: aynı aracın (kanıt özeti) Zee, Rust ve Go gerçeklemelerini
# veri temelli karşılaştırır. Üç çıktı bayt bayt aynı olmak zorundadır; sonra
# kaynak satırı, test sayısı, derleme ve çalışma süreleri ile tepe RSS ölçülür.
# Sonuç docs/dogfood-karsilastirma-v1.tsv'ye exact Git SHA ile eklenir ve
# docs/dogfood-karsilastirma.md içindeki işaretli tablo yeniden yazılır.
set -euo pipefail

depo="$(git rev-parse --show-toplevel)"
cd "$depo"
tur="${1:-10}"
if [[ -n "$(git status --porcelain)" && "${ZEE_KIRLI_AGAC_KABUL:-}" != "1" ]]; then
    echo "Çalışma ağacı temiz değil; tarihçe yalnız temiz ağaçtan exact SHA ile yazılır (ZEE_KIRLI_AGAC_KABUL=1 yerel deneme içindir)." >&2
    exit 2
fi

zee_kaynak="dogfood/kanit-ozeti/kaynak"
rust_kok="dogfood/kanit-ozeti-karsilastirma/rust"
go_kok="dogfood/kanit-ozeti-karsilastirma/go"
cikti_dizini="$(mktemp -d)"
trap 'rm -rf "$cikti_dizini"' EXIT

loc() { # boş ve yorum satırı dışı satır sayısı
    local yorum="$1"; shift
    cat "$@" | awk -v y="$yorum" '{ s=$0; sub(/^[ \t]+/, "", s); if (s != "" && index(s, y) != 1) n++ } END { print n+0 }'
}
sure_ms() { python3 - "$@" <<'PY'
import subprocess, sys, time, statistics
komut = sys.argv[1:]
sureler = []
for _ in range(int(__import__("os").environ.get("ZEE_TUR", "10"))):
    t = time.perf_counter()
    subprocess.run(komut, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    sureler.append((time.perf_counter() - t) * 1000)
print(int(statistics.median(sureler)))
PY
}
tepe_rss_kib() { python3 - "$@" <<'PY'
import resource, subprocess, sys, os
komut = sys.argv[1:]
onceki = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
subprocess.run(komut, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
rss = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
print(rss // 1024 if sys.platform == "darwin" else rss)
PY
}
export ZEE_TUR="$tur"

echo "== derleme"
zee_derleme_ms="$(python3 -c '
import subprocess, time; t=time.perf_counter(); subprocess.run(["cargo","build","--locked","--release","--bin","dil"],cwd="compiler",check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL); print(int((time.perf_counter()-t)*1000))')"
rust_derleme_ms="$(python3 -c '
import subprocess, time, shutil; shutil.rmtree("'"$rust_kok"'/target", ignore_errors=True); t=time.perf_counter(); subprocess.run(["cargo","build","--locked","--release"],cwd="'"$rust_kok"'",check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL); print(int((time.perf_counter()-t)*1000))')"
go_derleme_ms="-"
go_var=0
if command -v go >/dev/null 2>&1; then
    go_var=1
    go_derleme_ms="$(python3 -c '
import subprocess, time; t=time.perf_counter(); subprocess.run(["go","build","-o","'"$cikti_dizini"'/kanit-ozeti-go","."],cwd="'"$go_kok"'",check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL); print(int((time.perf_counter()-t)*1000))')"
fi

echo "== eşdeğerlik"
dil="compiler/target/release/dil"
"$dil" çalıştır "$zee_kaynak/ana.dil" >/dev/null
cp docs/kanit-ozeti.md "$cikti_dizini/zee.md"
"$rust_kok/target/release/kanit_ozeti_rs" . > "$cikti_dizini/rust.md"
cmp "$cikti_dizini/zee.md" "$cikti_dizini/rust.md"
if [[ "$go_var" == 1 ]]; then
    "$cikti_dizini/kanit-ozeti-go" . > "$cikti_dizini/go.md"
    cmp "$cikti_dizini/zee.md" "$cikti_dizini/go.md"
fi
if [[ -n "$(git status --porcelain docs/kanit-ozeti.md)" ]]; then
    echo "docs/kanit-ozeti.md bayattı; yeniden üretildi, commit'le." >&2
    exit 1
fi

echo "== ölçüm"
zee_loc="$(loc '#' "$zee_kaynak"/*.dil)"
rust_loc="$(loc '//' "$rust_kok"/src/main.rs)"
go_loc="$(loc '//' "$go_kok"/main.go "$go_kok"/main_test.go)"
zee_test="$(grep -c '^test "' "$zee_kaynak"/*.dil | awk -F: '{ n += $2 } END { print n+0 }')"
rust_test="$(grep -c '#\[test\]' "$rust_kok"/src/main.rs)"
go_test="$(grep -c '^func Test' "$go_kok"/main_test.go)"
zee_calisma_ms="$(sure_ms "$dil" çalıştır "$zee_kaynak/ana.dil")"
rust_calisma_ms="$(sure_ms "$rust_kok/target/release/kanit_ozeti_rs" .)"
zee_rss="$(tepe_rss_kib "$dil" çalıştır "$zee_kaynak/ana.dil")"
rust_rss="$(tepe_rss_kib "$rust_kok/target/release/kanit_ozeti_rs" .)"
go_calisma_ms="-"; go_rss="-"
if [[ "$go_var" == 1 ]]; then
    go_calisma_ms="$(sure_ms "$cikti_dizini/kanit-ozeti-go" .)"
    go_rss="$(tepe_rss_kib "$cikti_dizini/kanit-ozeti-go" .)"
fi
git checkout -- docs/kanit-ozeti.md 2>/dev/null || true
surtunme="$(grep -c '^f0' dogfood/korpus-v1.tsv || true)"

sha="$(git rev-parse HEAD)"
tarih="$(date -u +%Y-%m-%d)"
platform="$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"
rustc_surumu="$(rustc --version)"
go_surumu="-"
[[ "$go_var" == 1 ]] && go_surumu="$(go version | awk '{print $3}')"
tsv="docs/dogfood-karsilastirma-v1.tsv"
if [[ ! -f "$tsv" ]]; then
    printf '# zee-dogfood-karsilastirma-1\n# git_sha\ttarih\tplatform\trustc\tgo\ttur\tzee_loc\trust_loc\tgo_loc\tzee_test\trust_test\tgo_test\tzee_derleme_ms\trust_derleme_ms\tgo_derleme_ms\tzee_calisma_ms\trust_calisma_ms\tgo_calisma_ms\tzee_rss_kib\trust_rss_kib\tgo_rss_kib\tkorpus_surtunme_vakasi\n' > "$tsv"
fi
printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$sha" "$tarih" "$platform" "$rustc_surumu" "$go_surumu" "$tur" \
    "$zee_loc" "$rust_loc" "$go_loc" "$zee_test" "$rust_test" "$go_test" \
    "$zee_derleme_ms" "$rust_derleme_ms" "$go_derleme_ms" \
    "$zee_calisma_ms" "$rust_calisma_ms" "$go_calisma_ms" \
    "$zee_rss" "$rust_rss" "$go_rss" "$surtunme" >> "$tsv"

md="docs/dogfood-karsilastirma.md"
tablo="$(cat <<TABLO
<!-- ZEE-KARSILASTIRMA:BEGIN -->
Son ölçüm: \`${sha:0:12}\` · $tarih · $platform · $rustc_surumu · go $go_surumu · $tur tur medyanı.

| Ölçü | Zee | Rust | Go |
|---|---:|---:|---:|
| Kaynak satırı (boş/yorum dışı, testler dahil) | $zee_loc | $rust_loc | $go_loc |
| Birim testi | $zee_test | $rust_test | $go_test |
| Derleme süresi (ms; Zee için \`dil\` release derlemesi) | $zee_derleme_ms | $rust_derleme_ms | $go_derleme_ms |
| Çalışma süresi (ms; Zee: derle+yürüt) | $zee_calisma_ms | $rust_calisma_ms | $go_calisma_ms |
| Tepe RSS (KiB) | $zee_rss | $rust_rss | $go_rss |
| Korpusa giren sürtünme vakası | $surtunme | — | — |
<!-- ZEE-KARSILASTIRMA:END -->
TABLO
)"
python3 - "$md" "$tablo" <<'PY'
import re, sys
yol, tablo = sys.argv[1], sys.argv[2]
metin = open(yol, encoding="utf-8").read()
yeni = re.sub(r"<!-- ZEE-KARSILASTIRMA:BEGIN -->.*?<!-- ZEE-KARSILASTIRMA:END -->", lambda _: tablo, metin, flags=re.S)
open(yol, "w", encoding="utf-8").write(yeni)
PY
echo "kayıt: $tsv"; tail -1 "$tsv"
