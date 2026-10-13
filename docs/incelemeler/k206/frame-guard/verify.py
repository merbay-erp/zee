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
    (ROOT / "compiler/target").mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="zee-k206-frame-", dir=ROOT / "compiler/target") as temporary:
        crate = Path(temporary) / "postgres-protocol"
        crate.mkdir()
        pin = "c4e9d03616fefa3d4aa9edcdef5c3ab8af7a8c75"
        prefix = "compiler/third_party/postgres-protocol/"
        listing = subprocess.check_output(["git", "ls-tree", "-r", "--name-only", pin, "--", prefix], cwd=ROOT).decode().splitlines()
        for name in listing:
            relative = Path(name.removeprefix(prefix))
            if relative.is_absolute() or ".." in relative.parts:
                raise ValueError("invalid Git source path")
            destination = crate / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(subprocess.check_output(["git", "show", f"{pin}:{name}"], cwd=ROOT))
        if tree_hash(crate) != EXPECTED_TREE:
            raise ValueError("exact Git patch base tree differs")
        # --fuzz=0 and the exact source digest forbid quietly applying to another base.
        run(["patch", "--batch", "--fuzz=0", "-p1", "-i", str(HERE / "backend.patch")], crate)
        shutil.copyfile(HERE / "Cargo.lock", crate / "Cargo.lock")
        cargo = ["rustup", "run", "1.93.1", "cargo"]
        run(cargo + ["test", "--locked", "--offline", "--lib"], crate)
        run(cargo + ["clippy", "--locked", "--offline", "--lib", "--", "-D", "warnings"], crate)
        run(["rustup", "run", "1.93.1", "rustfmt", "--check", str(crate / "src/message/backend.rs")], crate)
        if tree_hash(SOURCE) != before:
            raise ValueError("production source was modified")
        print(json.dumps({"upstream_tree_sha256": EXPECTED_TREE,
                          "patch_sha256": hashlib.sha256((HERE / "backend.patch").read_bytes()).hexdigest(),
                          "patched_backend_sha256": hashlib.sha256((crate / "src/message/backend.rs").read_bytes()).hexdigest(),
                          "production_patch_active": False}, sort_keys=True))


if __name__ == "__main__":
    main()
