#!/usr/bin/env python3
"""K-214/K-334: müşteri/altyapı sözlüğünü yayımlamadan Git içeriğini denetler.

Politika yalnız gizli anahtarlı HMAC-SHA256 değerleri taşır (ADR-087). Anahtar
depo dışındadır: `ZEE_ACIK_DEPO_ANAHTARI` ortam değişkeni ya da
`~/.config/zee/acik-depo-anahtari`. Düz terim listesi de depo dışındadır ve
yalnız `--politika-uret` ile HMAC politikasına çevrilir.
"""
import argparse
from collections import Counter
from functools import lru_cache
import hashlib
import hmac
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import unicodedata


ROOT = Path(__file__).resolve().parents[1]
POLICY = ROOT / "docs/acik-depo-politikasi-v2.json"
SCHEMA = "zee-acik-depo-politikasi-2"
KEY_ENV = "ZEE_ACIK_DEPO_ANAHTARI"
KEY_FILE = Path("~/.config/zee/acik-depo-anahtari")
KEY_ID_LABEL = b"zee-acik-depo-anahtar-kimligi-v2"


class MissingKey(Exception):
    """Anahtar yok: kapı geçti sayılmaz, açıkça atlandı ya da hata olur."""


def normalize(text):
    text = unicodedata.normalize("NFKD", text.casefold().replace("ı", "i"))
    return "".join(c for c in text if c.isalnum() and not unicodedata.combining(c))


def digest(text):
    """Anahtarsız SHA-256: yalnız maskeli çıktı ve satır fingerprint'i için."""
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


@lru_cache(maxsize=4)
def _pads(key):
    # HMAC-SHA256 (RFC 2104) iç/dış durumu bir kez hazırlanır; tarihçe taraması
    # milyonlarca kısa alt dize hash'ler ve hmac.new her çağrıda bunu yeniden yapar.
    block = key.ljust(64, b"\0")
    return (hashlib.sha256(bytes(b ^ 0x36 for b in block)),
            hashlib.sha256(bytes(b ^ 0x5C for b in block)))


@lru_cache(maxsize=1 << 18)
def keyed(key, text):
    # Aynı aday dize (sözcük birleşimleri, alt dizeler) tarihçede çok kez tekrar eder.
    inner, outer = _pads(key)
    inner = inner.copy()
    inner.update(text.encode("utf-8"))
    outer = outer.copy()
    outer.update(inner.digest())
    return outer.hexdigest()


def key_id(key):
    return hmac.new(key, KEY_ID_LABEL, hashlib.sha256).hexdigest()


def parse_key(raw):
    raw = raw.strip()
    if not re.fullmatch(r"[0-9a-f]{64}", raw):
        raise ValueError("Anahtar 64 küçük harf onaltılık karakter (32 bayt) olmalı")
    return bytes.fromhex(raw)


def load_key(path=None):
    raw = os.environ.get(KEY_ENV, "")
    if raw.strip():
        return parse_key(raw)
    path = (path or KEY_FILE).expanduser()
    if path.is_file():
        return parse_key(path.read_text(encoding="utf-8"))
    raise MissingKey()


def plain_text(text):
    text = unicodedata.normalize("NFKC", text)
    return "".join(c for c in text if c != "\0" and unicodedata.category(c) != "Cf")


def tokens(text):
    return [normalize(t) for t in re.findall(r"[^\W_]+", plain_text(text), re.UNICODE)]


def exact_words(text, rules):
    if not rules.get("words"):
        return set()
    text = plain_text(text)
    parts = list(re.finditer(r"[^\W_]+", text, re.UNICODE))
    found = set()
    limit = max(rules.get("word_lengths", set()), default=0)
    for start in range(len(parts)):
        candidate = ""
        for end in range(start, min(start + 6, len(parts))):
            if end > start and not re.fullmatch(r"[\s._-]*", text[parts[end-1].end():parts[end].start()]):
                break
            candidate += normalize(parts[end].group())
            if len(candidate) > limit:
                break
            key = keyed(rules["key"], candidate)
            if key in rules.get("words", set()):
                found.add(key)
    return found


@lru_cache(maxsize=1 << 15)
def word_hashes(word, lengths, terms, secret):
    if len(word) >= 32 and re.fullmatch(r"[0-9a-f]+", word):
        return frozenset()
    # Cache only matches, not every candidate digest in long build logs. The
    # policy belongs in the key so independent policies cannot share results.
    terms = frozenset(terms)
    found = set()
    for size in lengths:
        for start in range(len(word) - size + 1):
            key = keyed(secret, word[start:start + size])
            if key in terms:
                found.add(key)
    return frozenset(found)


def matches(text, rules):
    words = tokens(text)
    candidates = set()
    lengths = tuple(sorted(rules["lengths"]))
    terms = tuple(sorted(rules["terms"]))
    for word in words:
        candidates.update(word_hashes(word, lengths, terms, rules["key"]))
    max_length = max(rules["lengths"])
    for start in range(len(words)):
        candidate = ""
        for width in range(1, 7):
            if start + width > len(words):
                break
            candidate += words[start + width - 1]
            if len(candidate) > max_length:
                break
            if len(candidate) in rules["lengths"]:
                key = keyed(rules["key"], candidate)
                if key in rules["terms"]:
                    candidates.add(key)
    found = candidates & rules["terms"]
    found.update(exact_words(text, rules))
    # Server roots have a separate exact structural rule; normal words such as
    # DNS SRV cannot trigger this rule.
    for path in re.findall(r"/(?:[^/\s`\"'<>]+/){1,5}", text):
        components = path.split("/")
        for length in range(2, len(components) + 1):
            candidate = "/".join(components[:length]) + "/"
            candidate_hash = keyed(rules["key"], candidate.casefold())
            if candidate_hash in rules["paths"]:
                found.add(candidate_hash)
    return found


def fingerprint(path, line, rule):
    return digest(path + "\0" + line + "\0" + rule)


def load_policy(path, key):
    data = json.loads(path.read_text(encoding="utf-8"))
    if data.get("schema") != SCHEMA:
        raise ValueError("Politika şeması geçersiz")
    if not re.fullmatch(r"[0-9a-f]{64}", data.get("key_id", "")):
        raise ValueError("Anahtar kimliği eksik")
    # Yanlış anahtar hiçbir şeyle eşleşmez; sessiz geçişi burada durdur.
    if not hmac.compare_digest(data["key_id"], key_id(key)):
        raise ValueError("Anahtar politika ile eşleşmiyor")
    if not re.fullmatch(r"[0-9a-f]{40}", data.get("history_base", "")):
        raise ValueError("Tarihçe tabanı exact SHA olmalı")
    entries = data.get("rules", [])
    if not entries or any(set(row) != {"kind", "length", "hmac"}
                          or row["kind"] not in {"term", "word", "path"}
                          or not isinstance(row["length"], int)
                          or isinstance(row["length"], bool)
                          or row["length"] < 1
                          or not re.fullmatch(r"[0-9a-f]{64}", row["hmac"])
                          for row in entries):
        raise ValueError("Hash kuralı geçersiz")
    if len({(r["kind"], r["hmac"]) for r in entries}) != len(entries):
        raise ValueError("Hash kuralı yineleniyor")
    if data.get("history_scope") != "all-reachable":
        raise ValueError("Tüm erişilebilir tarihçe zorunlu")
    baseline = data.get("legacy_fingerprints", {})
    if baseline:
        raise ValueError("Eski müşteri kaydı istisnası kabul edilmez")
    if not isinstance(baseline, dict) or any(
            not re.fullmatch(r"[0-9a-f]{64}", key)
            or not isinstance(count, int) or isinstance(count, bool) or count < 1
            for key, count in baseline.items()):
        raise ValueError("Eski kayıt fingerprint'i geçersiz")
    return data, {
        "key": key,
        "terms": {r["hmac"] for r in entries if r["kind"] == "term"},
        "paths": {r["hmac"] for r in entries if r["kind"] == "path"},
        "lengths": {r["length"] for r in entries if r["kind"] == "term"},
        "words": {r["hmac"] for r in entries if r["kind"] == "word"},
        "word_lengths": {r["length"] for r in entries if r["kind"] == "word"},
    }


def private_value(kind, value):
    if kind == "path":
        if not re.fullmatch(r"/(?:[^/\s]+/){1,5}", value):
            raise ValueError("Yol kuralı /a/ … biçiminde olmalı")
        return value.casefold()
    value = normalize(value)
    if not value:
        raise ValueError("Boş terim")
    return value


def build_policy(private_path, key, history_base):
    """Depo dışındaki düz terim listesini yalnız HMAC değerlerine çevirir."""
    data = json.loads(private_path.read_text(encoding="utf-8"))
    if data.get("schema") != "zee-acik-depo-ozel-terimler-1":
        raise ValueError("Özel terim listesi şeması geçersiz")
    rules = []
    for row in data.get("rules", []):
        if row.get("kind") not in {"term", "word", "path"}:
            raise ValueError("Özel kural türü geçersiz")
        value = private_value(row["kind"], row.get("value", ""))
        rules.append({"kind": row["kind"], "length": len(value),
                      "hmac": keyed(key, value)})
    # Sıralama düz metinden değil HMAC değerinden gelir; alfabetik sıra sızmaz.
    rules.sort(key=lambda r: (r["kind"], r["hmac"]))
    return {"schema": SCHEMA, "history_base": history_base, "key_id": key_id(key),
            "rules": rules, "legacy_fingerprints": {}, "history_scope": "all-reachable"}


def skipped(reason):
    message = f"Açık depo kapısı ATLANDI: {reason}. Bu sonuç geçti sayılmaz."
    print(message)
    if os.environ.get("GITHUB_ACTIONS") == "true":
        print(f"::warning title=acik-depo atlandı::{message}")
        summary = os.environ.get("GITHUB_STEP_SUMMARY")
        if summary:
            with open(summary, "a", encoding="utf-8") as handle:
                handle.write(f"### ⚠️ {message}\n")


def inspect(path, contents, rules, allowances):
    text = contents.decode("utf-8-sig", "replace")
    if contents.startswith((b"\xff\xfe", b"\xfe\xff")):
        text = contents.decode("utf-16", "replace")
    elif b"\0" in contents:
        # NUL baytlı ikili dosya (font, görsel, wasm): rastgele bayt dizisi kısa sözcük
        # kurallarına yanlış eşleşir; içerik metin sayılmaz, yalnız dosya yolu taranır.
        text = ""
    used = Counter()
    findings = []
    for number, line in [(0, path)] + list(enumerate(text.splitlines(), 1)):
        for rule in matches(line, rules):
            key = fingerprint(path, line, rule)
            used[key] += 1
            if used[key] > allowances.get(key, 0):
                findings.append((digest(path), number))
    return sorted(set(findings))


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args], stderr=subprocess.PIPE)


def tree_files(root, revision):
    for record in git(root, "ls-tree", "-rz", revision).split(b"\0"):
        if not record:
            continue
        meta, name = record.split(b"\t", 1)
        _, kind, oid = meta.split()
        if kind == b"blob":
            yield name.decode("utf-8", "replace"), oid.decode()


class BlobReader:
    def __init__(self, root):
        self.process = subprocess.Popen(["git", "-C", str(root), "cat-file", "--batch"],
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.PIPE)

    def read(self, oid):
        self.process.stdin.write(oid.encode() + b"\n")
        self.process.stdin.flush()
        fields = self.process.stdout.readline().split()
        if len(fields) != 3 or fields[1] != b"blob":
            raise ValueError("Git blob okunamadı")
        contents = self.process.stdout.read(int(fields[2]))
        if self.process.stdout.read(1) != b"\n":
            raise ValueError("Git blob sınırı geçersiz")
        return contents

    def close(self):
        self.process.stdin.close()
        self.process.stdout.close()
        self.process.stderr.close()
        if self.process.wait() != 0:
            raise ValueError("Git blob okuyucu başarısız")


def scan(root, policy, rules, revision="HEAD", history=True, worktree=True):
    allowances = policy["legacy_fingerprints"]
    base = policy["history_base"]
    revisions = [revision]
    if history:
        # Never silently skip history because a shallow checkout lacks the base.
        git(root, "cat-file", "-e", base + "^{commit}")
        revisions += git(root, "rev-list", revision).decode().splitlines()
    seen = set()
    lines = {}
    findings = set()
    reader = BlobReader(root)
    try:
        for rev in dict.fromkeys(revisions):
            for path, oid in tree_files(root, rev):
                if (path, oid) in seen:
                    continue
                seen.add((path, oid))
                if allowances:
                    findings.update(inspect(path, reader.read(oid), rules, allowances))
                    continue
                # İstisna yokken içerik bulgusu yoldan bağımsızdır: taşınan/kopyalanan
                # aynı blob bir kez okunur, yol (satır 0) her seferinde ayrıca taranır.
                if oid not in lines:
                    lines[oid] = {n for _, n in inspect("", reader.read(oid), rules, {}) if n}
                path_hash = digest(path)
                findings.update((path_hash, n) for n in lines[oid])
                if matches(path, rules):
                    findings.add((path_hash, 0))
            if history:
                message = git(root, "show", "-s", "--format=%B", rev)
                findings.update(inspect("git-message", message, rules, {}))
    finally:
        reader.close()
    if worktree:
        paths = git(root, "ls-files", "-z", "--cached", "--others", "--exclude-standard")
        for name in paths.split(b"\0"):
            if not name:
                continue
            path = name.decode("utf-8", "replace")
            file = root / path
            if file.is_symlink():
                findings.update(inspect(path, str(file.readlink()).encode(), rules, allowances))
            elif file.is_file():
                findings.update(inspect(path, file.read_bytes(), rules, allowances))
    return sorted(findings)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--politika", type=Path, default=POLICY)
    parser.add_argument("--depo", type=Path, default=ROOT)
    parser.add_argument("--rev", default="HEAD")
    parser.add_argument("--yalniz-agac", action="store_true")
    parser.add_argument("--calisma-agaci-yok", action="store_true")
    parser.add_argument("--anahtar-dosyasi", type=Path)
    parser.add_argument("--anahtar-yoksa", choices=["hata", "atla"], default="hata",
                        help="atla yalnız gizli anahtarın verilmediği fork PR'ı içindir")
    parser.add_argument("--politika-uret", type=Path, metavar="OZEL_TERIMLER",
                        help="depo dışındaki düz terim listesinden HMAC politikası yazar")
    args = parser.parse_args()
    try:
        key = load_key(args.anahtar_dosyasi)
    except MissingKey:
        if args.anahtar_yoksa == "atla":
            skipped(f"gizli anahtar ({KEY_ENV}) yok; tipik olarak fork PR")
            return 0
        print(f"Açık depo kapısı çalışamadı: gizli anahtar yok ({KEY_ENV} ya da {KEY_FILE}).",
              file=sys.stderr)
        return 3
    except (ValueError, OSError):
        print("Açık depo kapısı çalışamadı: anahtar biçimi geçersiz.", file=sys.stderr)
        return 2
    if args.politika_uret:
        try:
            base = json.loads(args.politika.read_text(encoding="utf-8"))["history_base"]
            policy = build_policy(args.politika_uret, key, base)
        except (ValueError, OSError, KeyError):
            print("Politika üretilemedi: özel terim listesi ya da mevcut politika geçersiz.",
                  file=sys.stderr)
            return 2
        args.politika.write_text(json.dumps(policy, indent=2) + "\n", encoding="utf-8")
        print(f"HMAC politikası yazıldı: {len(policy['rules'])} kural.")
        return 0
    try:
        policy, rules = load_policy(args.politika, key)
        findings = scan(args.depo, policy, rules, args.rev,
                        not args.yalniz_agac, not args.calisma_agaci_yok)
    except (ValueError, OSError, subprocess.CalledProcessError, KeyError):
        print("Açık depo kapısı çalışamadı: politika, anahtar eşleşmesi veya tam Git geçmişi "
              "eksik/geçersiz.", file=sys.stderr)
        return 2
    if findings:
        for path_hash, number in findings[:100]:
            print(f"MASKELİ BULGU dosya_sha256={path_hash} satır={number}", file=sys.stderr)
        print(f"Açık depo kapısı reddetti: {len(findings)} konum.", file=sys.stderr)
        return 1
    print("Açık depo kapısı geçti; erişilebilir tarihçede müşteri/altyapı eşleşmesi yok.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
