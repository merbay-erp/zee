#!/usr/bin/env python3
"""K-197: exact release ikililerini Rust içermeyen Nano Server'da sına.

Host yalnız Docker/Python araçlarını sağlar; container'a iki ikili ve kaynak
read-only bağlanır. Masaüstü/minimum Windows OS veya bütün IO kanıtı değildir.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import runpy
import shutil
import subprocess
import sys
import tempfile

BASE = "mcr.microsoft.com/windows/nanoserver@sha256:4fe6f59e0aadb628a933345eae1f5ab43a2af802969ac0e2ef996c66a3188438"


def artefakt_dogrula(root, sha):
    if not re.fullmatch(r"[0-9a-f]{40}", sha):
        raise ValueError("Expected source SHA must be exact")
    if (root / "GIT_SHA").read_text(encoding="ascii").strip() != sha:
        raise ValueError("Artifact source differs")
    hashes = {}
    for line in (root / "SHA256SUMS").read_text(encoding="ascii").splitlines():
        value, name = line.split("  ", 1)
        if name not in ("dil.exe", "dillsp.exe") or name in hashes or not re.fullmatch(r"[0-9a-f]{64}", value):
            raise ValueError("Artifact hash/name/uniqueness invalid")
        if hashlib.sha256((root / name).read_bytes()).hexdigest() != value:
            raise ValueError("Artifact binary hash differs")
        hashes[name] = value
    if set(hashes) != {"dil.exe", "dillsp.exe"}:
        raise ValueError("Two release binaries required")
    return hashes


def run(command, data=None, timeout=60):
    result = subprocess.run(command, input=data, capture_output=True, timeout=timeout)
    if result.returncode:
        raise ValueError(f"Process exit={result.returncode}: " + result.stderr.decode("utf-8", errors="replace")[:4000])
    return result.stdout


def lsp_dogrula(replies):
    identified = [r for r in replies if "id" in r]
    by_id = {r["id"]: r for r in identified}
    if (len(identified) != 2 or set(by_id) != {1, 2}
            or not isinstance(by_id[1].get("result"), dict)
            or not isinstance(by_id[1]["result"].get("capabilities"), dict)
            or "error" in by_id[1] or "error" in by_id[2]
            or "result" not in by_id[2] or by_id[2]["result"] is not None):
        raise ValueError("Container LSP initialize/shutdown reply invalid")


def prova(artifact, sha):
    hashes = artefakt_dogrula(artifact, sha)
    if sys.platform != "win32":
        raise ValueError("Windows Docker host required")
    info = json.loads(run(["docker", "info", "--format", "{{json .}}"], timeout=30))
    if info.get("OSType") != "windows":
        raise ValueError("Windows Docker daemon required")
    run(["docker", "pull", BASE], timeout=300)
    image = json.loads(run(["docker", "image", "inspect", BASE]))[0]
    if image.get("Os") != "windows" or image.get("Architecture") != "amd64":
        raise ValueError("Pinned Windows amd64 image required")
    helper = runpy.run_path(str(Path(__file__).with_name("calisma-ortami-denetle.py")))
    with tempfile.TemporaryDirectory(prefix="zee-clean-windows-") as directory:
        staged = Path(directory)
        for name, value in hashes.items():
            shutil.copyfile(artifact / name, staged / name)
            if hashlib.sha256((staged / name).read_bytes()).hexdigest() != value:
                raise ValueError("Staged binary hash differs")
        (staged / "merhaba.dil").write_bytes('"Dünyaya merhaba" yaz\n'.encode("utf-8"))
        # Yalnız bu geçici, kamuya açık release byte'larına read/execute ACL.
        run(["icacls", str(staged), "/grant", "*S-1-5-32-545:(OI)(CI)RX", "/T"], timeout=30)
        container = ["docker", "run", "--rm", "--isolation", "process", "--network", "none",
                     "--user", "ContainerUser", "--env", r"PATH=C:\Windows\System32;C:\Windows",
                     "--mount", f"type=bind,src={staged},dst=C:\\zee,readonly"]
        inventory_command = (r"ver & for %A in (rustc.exe cargo.exe rustup.exe) do @for %P in "
                             r"(C:\Windows\System32 C:\Windows) do @if exist %P\%A exit /b 17")
        inventory = run(container + [BASE, "cmd.exe", "/d", "/s", "/c", inventory_command]).decode("utf-8", errors="replace")
        output = run(container + [BASE, r"C:\zee\dil.exe", "çalıştır", r"C:\zee\merhaba.dil"])
        if output != "Dünyaya merhaba\n".encode("utf-8"):
            raise ValueError("Container program output differs")
        requests = [{"jsonrpc": "2.0", "id": 1, "method": "initialize",
                     "params": {"processId": None, "rootUri": None, "capabilities": {}}},
                    {"jsonrpc": "2.0", "id": 2, "method": "shutdown"},
                    {"jsonrpc": "2.0", "method": "exit"}]
        replies = helper["lsp_cevaplari"](run(container + ["-i", BASE, r"C:\zee\dillsp.exe"],
                                            b"".join(map(helper["lsp_cerceve"], requests))))
        lsp_dogrula(replies)
    return {"sema": "zee-temiz-windows-runtime-1", "sonuc": "gecti", "git_sha": sha,
            "prova_git_sha": os.environ.get("GITHUB_SHA", "-"),
            "prova_araci_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "tarih_utc": datetime.now(timezone.utc).isoformat(), "base": BASE,
            "image_id": image["Id"], "image_os_version": image.get("OsVersion"),
            "repo_digests": image.get("RepoDigests"), "docker_server_version": info.get("ServerVersion"),
            "inventory": inventory, "container_arguments": container[1:], "rust_araclari": "yok",
            "program_exit": 0, "program_ciktisi": output.decode("utf-8"),
            "lsp_initialize_shutdown_exit": "gecti", "lsp_cevaplari": replies, "ikililer": hashes,
            "sinir": "Nano Server process isolation paylaşılmış host kernel; yalnız program/LSP. Read-only bind, geçici yazılabilir root. Masaüstü/minimum OS veya bütün IO kanıtı değildir."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artefakt", type=Path, required=True)
    parser.add_argument("--beklenen-sha", required=True)
    parser.add_argument("--rapor", type=Path, required=True)
    args = parser.parse_args()
    try:
        report = prova(args.artefakt.resolve(), args.beklenen_sha)
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        report = {"sema": "zee-temiz-windows-runtime-1", "sonuc": "kaldi", "git_sha": args.beklenen_sha,
                  "base": BASE, "hata": str(error)[:4000]}
    args.rapor.parent.mkdir(parents=True, exist_ok=True)
    args.rapor.write_bytes((json.dumps(report, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print("clean-windows-runtime: " + report["sonuc"])
    return 0 if report["sonuc"] == "gecti" else 1


if __name__ == "__main__":
    sys.exit(main())
