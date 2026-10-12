#!/usr/bin/env bash
# K-169/ADR-071, K-194: ikililerin tamamı geçici alanda doğrulanır;
# yayınlama hatasında bu koşunun oluşturduğu dosyalar geri alınır.
#   scripts/kur.sh --artefakt KLASÖR [--hedef ~/.local/bin] [--veri ~/.local/share/zee]
set -euo pipefail

artefakt=""
hedef="${HOME}/.local/bin"
veri="${HOME}/.local/share/zee"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --artefakt) artefakt="$2"; shift 2 ;;
        --hedef) hedef="$2"; shift 2 ;;
        --veri) veri="$2"; shift 2 ;;
        *) echo "Kullanım: scripts/kur.sh --artefakt KLASÖR [--hedef BIN] [--veri VERİ]" >&2; exit 2 ;;
    esac
done
[[ -n "$artefakt" && -d "$artefakt" ]] || { echo "Sürüm artefakt klasörü gerekli (--artefakt)." >&2; exit 2; }
[[ -f "$artefakt/SHA256SUMS" ]] || { echo "Artefaktta SHA256SUMS yok; doğrulanamayan ikili kurulmaz." >&2; exit 1; }

ozet_hesapla() {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'; else shasum -a 256 "$1" | awk '{print $1}'; fi
}

manifest_yolu_denetle() {
    local yol
    for yol in "$@"; do
        case "$yol" in
            *$'\t'* | *$'\r'* | *$'\n'*) echo "Kurulum yolu TSV kontrol karakteri içeremez." >&2; return 1 ;;
        esac
    done
}
manifest_yolu_denetle "$hedef" "$veri"
git_sha="-"
if [[ -f "$artefakt/GIT_SHA" ]]; then
    git_sha="$(cat "$artefakt/GIT_SHA")"
    [[ "$git_sha" =~ ^[0-9a-f]{40}$ ]] || { echo "GIT_SHA exact 40 küçük hex olmalı." >&2; exit 1; }
fi

manifest="$veri/kurulum-v1.tsv"
if [[ -e "$manifest" || -L "$manifest" ]]; then
    echo "Önce var olan kurulumu kaldır: scripts/kaldir.sh --veri $veri" >&2
    exit 1
fi
mkdir -p "$hedef" "$veri"
# Manifestte yollar çalışma dizininden bağımsızdır.
hedef="$(cd -P "$hedef" && printf '%s/' "$PWD")"
hedef="${hedef%/}"
veri="$(cd -P "$veri" && printf '%s/' "$PWD")"
veri="${veri%/}"
manifest_yolu_denetle "$hedef" "$veri"
manifest="$veri/kurulum-v1.tsv"
hazirlik=""
gecici_manifest=""
kurulan_yollar=()
kurulacak_adlar=()
tamamlandi=0

temizle() {
    local sonuc="$?" dosya
    trap - EXIT
    if [[ "$tamamlandi" -ne 1 && ${#kurulan_yollar[@]} -gt 0 ]]; then
        for dosya in "${kurulan_yollar[@]}"; do rm -f "$dosya"; done
    fi
    [[ -z "$hazirlik" ]] || rm -rf "$hazirlik"
    [[ -z "$gecici_manifest" ]] || rm -f "$gecici_manifest"
    exit "$sonuc"
}
trap temizle EXIT
hazirlik="$(mktemp -d "$hedef/.zee-kurulum.XXXXXX")"
gecici_manifest="$(mktemp "$veri/.zee-kurulum-manifest.XXXXXX")"
printf '# zee-kurulum-1\n# dosya\tsha256\tkaynak_git_sha\n' > "$gecici_manifest"

while IFS= read -r satir || [[ -n "$satir" ]]; do
    [[ -z "$satir" ]] && continue
    beklenen="${satir%%  *}"
    ad="${satir##*  }"
    case "$ad" in dil | dil.exe | dillsp | dillsp.exe) ;; *) continue ;; esac
    [[ "$beklenen" =~ ^[0-9a-f]{64}$ ]] || { echo "SHA256SUMS özeti geçersiz: $ad" >&2; exit 1; }
    if [[ -e "$hazirlik/$ad" ]]; then
        echo "SHA256SUMS içinde yinelenen ad: $ad" >&2; exit 1
    fi
    kaynak="$artefakt/$ad"
    [[ -f "$kaynak" ]] || { echo "Artefaktta $ad yok." >&2; exit 1; }
    if [[ -e "$hedef/$ad" || -L "$hedef/$ad" ]]; then
        echo "$hedef/$ad zaten var; kaldırmadan üzerine yazılmaz." >&2; exit 1
    fi
    install -m 0755 "$kaynak" "$hazirlik/$ad"
    bulunan="$(ozet_hesapla "$hazirlik/$ad")"
    if [[ "$bulunan" != "$beklenen" ]]; then
        echo "SHA-256 uyuşmuyor: $ad (beklenen $beklenen, bulunan $bulunan); kurulum iptal." >&2
        exit 1
    fi
    kurulacak_adlar+=("$ad")
    printf '%s\t%s\t%s\n' "$hedef/$ad" "$bulunan" "$git_sha" >> "$gecici_manifest"
done < "$artefakt/SHA256SUMS"
if [[ ${#kurulacak_adlar[@]} -eq 0 ]]; then
    echo "SHA256SUMS içinde dil/dillsp girdisi yok." >&2; exit 1
fi

for ad in "${kurulacak_adlar[@]}"; do
    # Hard link aynı hedef dosya sisteminde oluşturulur; var olan dosya ezilmez.
    [[ ! -e "$hedef/$ad" && ! -L "$hedef/$ad" ]] || { echo "$hedef/$ad zaten var; kaldırmadan üzerine yazılmaz." >&2; exit 1; }
    ln "$hazirlik/$ad" "$hedef/$ad"
    kurulan_yollar+=("$hedef/$ad")
done
# Manifestin yayınlanması da no-clobber'dır ve kendi veri dosya sistemindedir.
[[ ! -e "$manifest" && ! -L "$manifest" ]] || { echo "Kurulum manifesti zaten var." >&2; exit 1; }
ln "$gecici_manifest" "$manifest"
tamamlandi=1
echo "Kuruldu (${#kurulan_yollar[@]} ikili) → $hedef; manifest: $manifest"
case ":$PATH:" in *":$hedef:"*) ;; *) echo "Not: $hedef PATH içinde değil; kabuğuna ekle." ;; esac
