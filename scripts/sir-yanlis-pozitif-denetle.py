#!/usr/bin/env python3
"""K-204: yalnız incelenmiş immutable geçmiş fingerprintlerini doğrular."""
import argparse
import hashlib
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
TLS_FIXTURE = "compiler/tests/fixtures/http-post-tls/server.p12"
TLS_PROOF = "compiler/tests/fixtures/http-post-tls/README.md"


def tls_fixture_dogrula(path, tree, root=ROOT):
    """Yalnız açık test kimliğinin exact içeriği directory fingerprint'i alır."""
    lines = path.read_text(encoding="utf-8").splitlines()
    rows = [line.split("\t") for line in lines if line and not line.startswith("#")]
    if not lines or lines[0] != "# zee-sentetik-tls-fixture-1" or len(rows) != 1:
        raise ValueError("TLS fixture kayıt şeması/sayısı geçersiz")
    if len(rows[0]) != 6:
        raise ValueError("TLS fixture kaydı altı alan ister")
    file, rule, digest, commit, proof, reason = rows[0]
    if (file != TLS_FIXTURE or rule != "pkcs12-file" or proof != TLS_PROOF
            or not re.fullmatch(r"[0-9a-f]{64}", digest)
            or not re.fullmatch(r"[0-9a-f]{40}", commit) or len(reason) < 40):
        raise ValueError("TLS fixture exact yol/kural/kaynak kaydı geçersiz")
    source = root / file
    if source.is_symlink() or not source.is_file() or not (root / proof).is_file():
        raise ValueError("TLS fixture veya test kökeni eksik")
    try:
        immutable = subprocess.check_output(["git", "-C", str(root), "show", commit + ":" + file],
                                            stderr=subprocess.PIPE)
    except subprocess.CalledProcessError as error:
        raise ValueError("TLS fixture immutable kaynak eksik") from error
    if any(hashlib.sha256(data).hexdigest() != digest for data in [source.read_bytes(), immutable]):
        raise ValueError("TLS fixture hash'i değişti; istisna verilmedi")
    if tree is None:
        return []
    copied = tree / file
    if copied.is_symlink() or not copied.is_file() or hashlib.sha256(copied.read_bytes()).hexdigest() != digest:
        raise ValueError("TLS fixture tarama kopyasının hash'i değişti")
    return [str(copied.absolute()) + ":pkcs12-file:0"]


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
    parser.add_argument("--tls-kayit", type=Path, default=ROOT / "docs/sentetik-tls-fixture-v1.tsv")
    parser.add_argument("--tree", type=Path)
    args = parser.parse_args()
    fingerprints = dogrula(args.kayit)
    fingerprints += tls_fixture_dogrula(args.tls_kayit, args.tree)
    args.out.write_text("\n".join(fingerprints)+"\n", encoding="utf-8")


if __name__ == "__main__":
    main()
