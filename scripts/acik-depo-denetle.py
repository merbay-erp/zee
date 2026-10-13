#!/usr/bin/env python3
"""K-214: müşteri/altyapı sözlüğünü yayımlamadan Git içeriğini denetler."""
import argparse
from collections import Counter
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import unicodedata


ROOT = Path(__file__).resolve().parents[1]
POLICY = ROOT / "docs/acik-depo-politikasi-v1.json"


def normalize(text):
    text = unicodedata.normalize("NFKD", text.casefold().replace("ı", "i"))
    return "".join(c for c in text if c.isalnum() and not unicodedata.combining(c))


def digest(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def tokens(text):
    text = unicodedata.normalize("NFKC", text)
    text = "".join(c for c in text if c != "\0" and unicodedata.category(c) != "Cf")
    return [normalize(t) for t in re.findall(r"[^\W_]+", text, re.UNICODE)]


@lru_cache(maxsize=8192)
def word_hashes(word, lengths):
    if len(word) >= 32 and re.fullmatch(r"[0-9a-f]+", word):
        return frozenset()
    return frozenset(digest(word[start:start + size])
                     for size in lengths if size <= len(word)
                     for start in range(len(word) - size + 1))


def matches(text, rules):
    words = tokens(text)
    candidates = set()
    lengths = tuple(sorted(rules["lengths"]))
    for word in words:
        candidates.update(word_hashes(word, lengths))
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
                candidates.add(digest(candidate))
    found = candidates & rules["terms"]
    # Server roots have a separate exact structural rule; normal words such as
    # DNS SRV cannot trigger this rule.
    for path in re.findall(r"/(?:[^/\s`\"'<>]+/){1,5}", text):
        components = path.split("/")
        for length in range(2, len(components) + 1):
            candidate = "/".join(components[:length]) + "/"
            candidate_hash = digest(candidate.casefold())
            if candidate_hash in rules["paths"]:
                found.add(candidate_hash)
    return found


def fingerprint(path, line, rule):
    return digest(path + "\0" + line + "\0" + rule)


def load_policy(path):
    data = json.loads(path.read_text(encoding="utf-8"))
    if data.get("schema") != "zee-acik-depo-politikasi-1":
        raise ValueError("Politika şeması geçersiz")
    if not re.fullmatch(r"[0-9a-f]{40}", data.get("history_base", "")):
        raise ValueError("Tarihçe tabanı exact SHA olmalı")
    entries = data.get("rules", [])
    if not entries or any(set(row) != {"kind", "length", "sha256"}
                          or row["kind"] not in {"term", "path"}
                          or not isinstance(row["length"], int)
                          or row["length"] < 1
                          or not re.fullmatch(r"[0-9a-f]{64}", row["sha256"])
                          for row in entries):
        raise ValueError("Hash kuralı geçersiz")
    if len({(r["kind"], r["sha256"]) for r in entries}) != len(entries):
        raise ValueError("Hash kuralı yineleniyor")
    baseline = data.get("legacy_fingerprints", {})
    if not isinstance(baseline, dict) or any(
            not re.fullmatch(r"[0-9a-f]{64}", key)
            or not isinstance(count, int) or isinstance(count, bool) or count < 1
            for key, count in baseline.items()):
        raise ValueError("Eski kayıt fingerprint'i geçersiz")
    return data, {
        "terms": {r["sha256"] for r in entries if r["kind"] == "term"},
        "paths": {r["sha256"] for r in entries if r["kind"] == "path"},
        "lengths": {r["length"] for r in entries if r["kind"] == "term"},
    }


def inspect(path, contents, rules, allowances):
    text = contents.decode("utf-8-sig", "replace")
    if contents.startswith((b"\xff\xfe", b"\xfe\xff")):
        text = contents.decode("utf-16", "replace")
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
        revisions += git(root, "rev-list", base + ".." + revision).decode().splitlines()
    seen = set()
    findings = set()
    reader = BlobReader(root)
    try:
        for rev in dict.fromkeys(revisions):
            for path, oid in tree_files(root, rev):
                if (path, oid) in seen:
                    continue
                seen.add((path, oid))
                findings.update(inspect(path, reader.read(oid), rules, allowances))
            if history and rev != base:
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
    args = parser.parse_args()
    try:
        policy, rules = load_policy(args.politika)
        findings = scan(args.depo, policy, rules, args.rev,
                        not args.yalniz_agac, not args.calisma_agaci_yok)
    except (ValueError, OSError, subprocess.CalledProcessError, KeyError):
        print("Açık depo kapısı çalışamadı: politika veya tam Git geçmişi eksik/geçersiz.", file=sys.stderr)
        return 2
    if findings:
        for path_hash, number in findings[:100]:
            print(f"MASKELİ BULGU dosya_sha256={path_hash} satır={number}", file=sys.stderr)
        print(f"Açık depo kapısı reddetti: {len(findings)} konum.", file=sys.stderr)
        return 1
    print("Açık depo kapısı geçti; yeni müşteri/altyapı eşleşmesi yok.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
