#!/usr/bin/env python3
"""Yalnız kullanıcının K-214/K-215/K-326 yeniden yazımlarındaki eski dal uçlarını eşler."""
from pathlib import Path
import re
import subprocess
import sys

OLD_MASTER = "45b66f3574e0be020aeffc95580dd8db822e1379"
FULL_OLD_MASTER = "4c13052abaed4f199aeb5210f4180e1f8cebc420"
OLD_TIPS = {FULL_OLD_MASTER, OLD_MASTER, "0a8d1ba197d01fbcf87927ee7a26b89682807767",
            "179fe78f1c446621b584abb8ddb3a6e3e996d591"}
# K-326: GitHub ana dalı ile Enterprise/Git2 aynalarının yeniden yazım öncesi uçları.
K326_TIPS = {"e196b6ede4109e59a4202e7503bd1a2387eb9498",
             "bbc9ba4b5ca9c7d89f4bcad8c4b8eb4872c9f85d"}
OLD_TIPS |= K326_TIPS
ROOT = Path(__file__).resolve().parents[1]


def resolve(before, root=ROOT):
    if before not in OLD_TIPS:
        return before
    rows = [line.split("\t") for line in
            (root / ("docs/incelemeler/k326/sha-esleme-v1.tsv" if before in K326_TIPS
                     else "docs/incelemeler/k215/sha-esleme-v1.tsv" if before == FULL_OLD_MASTER
                     else "docs/incelemeler/k214/sha-esleme-v1.tsv")).read_text().splitlines()
            if line and not line.startswith("#")]
    selected = [row for row in rows if row[0] == before]
    if len(selected) != 1 or len(selected[0]) != 2 or not re.fullmatch(r"[0-9a-f]{40}", selected[0][1]):
        raise ValueError("Exact eski master eşlemesi eksik/bozuk")
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
