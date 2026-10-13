#!/usr/bin/env python3
"""K-204: yalnız incelenmiş immutable geçmiş fingerprintlerini doğrular."""
import argparse
import hashlib
from pathlib import Path
import re
import subprocess


def dogrula(path):
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != "# zee-sir-yanlis-pozitif-1":
        raise ValueError("Yanlış pozitif kayıt şeması geçersiz")
    seen = set()
    for line in lines:
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 3:
            raise ValueError("Yanlış pozitif kaydı üç alan ister")
        fingerprint, digest, reason = fields
        match = re.fullmatch(r"([0-9a-f]{40}):((?:docs|adr)/[^:]+\.md):(generic-api-key):([1-9][0-9]*)", fingerprint)
        if (not match or fingerprint in seen or not re.fullmatch(r"[0-9a-f]{64}", digest)
                or len(reason) < 40 or ".." in Path(match[2]).parts):
            raise ValueError("Exact fingerprint/hash/gerekçe geçersiz")
        content = subprocess.check_output(["git", "show", match[1]+":"+match[2]]).decode("utf-8").splitlines()
        number = int(match[4])
        if number > len(content) or hashlib.sha256(content[number-1].encode()).hexdigest() != digest:
            raise ValueError("Yanlış pozitif immutable kaynak satırı hash'i eşleşmiyor")
        seen.add(fingerprint)
    if not seen:
        raise ValueError("Yanlış pozitif kaydı boş")
    return sorted(seen)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kayit", type=Path, default=Path("docs/sir-yanlis-pozitif-v1.tsv"))
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    fingerprints = dogrula(args.kayit)
    args.out.write_text("\n".join(fingerprints)+"\n", encoding="utf-8")


if __name__ == "__main__":
    main()
