#!/usr/bin/env python3
"""Apply the inactive K-206 patch only in a temporary crate and test it offline."""
import hashlib
import json
import shutil
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCE = ROOT / "compiler/third_party/postgres-protocol"
EXPECTED_TREE = "30fe54faeea20f6772380c5902389016cef4a7823c5886d5e8e39821764aabe4"


def tree_hash(root):
    digest = hashlib.sha256()
    for path in sorted(root.rglob("*"), key=lambda p: p.relative_to(root).as_posix()):
        if path.is_symlink():
            raise ValueError("source symlink forbidden")
        if path.is_dir():
            continue
        if not path.is_file():
            raise ValueError("nonregular source forbidden")
        name = path.relative_to(root).as_posix().encode()
        body = path.read_bytes()
        digest.update(len(name).to_bytes(8, "big"))
        digest.update(name)
        digest.update(len(body).to_bytes(8, "big"))
        digest.update(body)
    return digest.hexdigest()


def run(args, cwd):
    subprocess.run(args, cwd=cwd, check=True)


def main():
    before = tree_hash(SOURCE)
    if before != EXPECTED_TREE:
        raise ValueError("pinned upstream source tree differs from reviewed patch base")
    (ROOT / "compiler/target").mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="zee-k206-frame-", dir=ROOT / "compiler/target") as temporary:
        crate = Path(temporary) / "postgres-protocol"
        shutil.copytree(SOURCE, crate)
        # --fuzz=0 and the exact source digest forbid quietly applying to another base.
        run(["patch", "--batch", "--fuzz=0", "-p1", "-i", str(HERE / "backend.patch")], crate)
        shutil.copyfile(HERE / "Cargo.lock", crate / "Cargo.lock")
        cargo = ["rustup", "run", "1.93.1", "cargo"]
        run(cargo + ["test", "--locked", "--offline", "--lib"], crate)
        run(cargo + ["clippy", "--locked", "--offline", "--lib", "--", "-D", "warnings"], crate)
        run(["rustup", "run", "1.93.1", "rustfmt", "--check", str(crate / "src/message/backend.rs")], crate)
        if tree_hash(SOURCE) != before:
            raise ValueError("production source was modified")
        print(json.dumps({"upstream_tree_sha256": before,
                          "patch_sha256": hashlib.sha256((HERE / "backend.patch").read_bytes()).hexdigest(),
                          "patched_backend_sha256": hashlib.sha256((crate / "src/message/backend.rs").read_bytes()).hexdigest(),
                          "production_patch_active": False}, sort_keys=True))


if __name__ == "__main__":
    main()
