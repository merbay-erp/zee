#!/usr/bin/env python3
"""Yalnız kullanıcının K-214 yeniden yazımındaki üç eski dal ucunu eşler."""
from pathlib import Path
import re
import subprocess
import sys

OLD_MASTER = "45b66f3574e0be020aeffc95580dd8db822e1379"
OLD_TIPS = {OLD_MASTER, "0a8d1ba197d01fbcf87927ee7a26b89682807767",
            "179fe78f1c446621b584abb8ddb3a6e3e996d591"}
ROOT = Path(__file__).resolve().parents[1]


def resolve(before, root=ROOT):
    if before not in OLD_TIPS:
        return before
    rows = [line.split("\t") for line in
            (root / "docs/incelemeler/k214/sha-esleme-v1.tsv").read_text().splitlines()
            if line and not line.startswith("#")]
    selected = [row for row in rows if row[0] == before]
    if len(selected) != 1 or len(selected[0]) != 2 or not re.fullmatch(r"[0-9a-f]{40}", selected[0][1]):
        raise ValueError("K-214 exact master eşlemesi eksik/bozuk")
    target = selected[0][1]
    subprocess.run(["git", "-C", str(root), "merge-base", "--is-ancestor", target, "HEAD"],
                   check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return target


if __name__ == "__main__":
    try:
        print(resolve(sys.argv[1]))
    except (IndexError, ValueError, OSError, subprocess.CalledProcessError):
        print("K-214 taban kimliği doğrulanamadı.", file=sys.stderr)
        sys.exit(2)
