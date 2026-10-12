#!/usr/bin/env python3
"""K-198: stock Server Core'da kur/program/LSP/kaldır, Rust kurulmadan."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess
import sys
import tempfile
import uuid

BASE = "mcr.microsoft.com/windows/servercore@sha256:22505496dd4229dba63453ba0c6dc31c06fd3e11810e5b8e429aa6b16dab2457"
PS = r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
BIN = r"C:\Users\ContainerUser\Zee Türkçe\bin"
DATA = r"C:\Users\ContainerUser\Zee Türkçe\veri"


def manifest_dogrula(manifest, sha, hashes):
    if not manifest.startswith("# zee-kurulum-1"):
        raise ValueError("Installed manifest source/schema differs")
    records = [line.split("\t") for line in manifest.splitlines() if line and not line.startswith("#")]
    expected = [[BIN + "\\" + filename, hashes[filename], sha] for filename in ("dil.exe", "dillsp.exe")]
    if sorted(records) != sorted(expected):
        raise ValueError("Installed manifest paths/binary hashes differ")


def container_temizle(name, primary_error):
    try:
        cleanup = subprocess.run(["docker", "rm", "--force", name], capture_output=True, timeout=30)
        if cleanup.returncode and primary_error is None:
            raise ValueError("Container cleanup failed")
    except (OSError, subprocess.SubprocessError):
        if primary_error is None:
            raise


def prova(artifact, sha):
    scripts = Path(__file__).resolve().parent
    api = runpy.run_path(str(scripts / "temiz-windows-prova.py"))
    hashes = api["artefakt_dogrula"](artifact, sha)
    run = api["run"]
    if sys.platform != "win32":
        raise ValueError("Windows Docker host required")
    info = json.loads(run(["docker", "info", "--format", "{{json .}}"], timeout=30))
    if info.get("OSType") != "windows":
        raise ValueError("Windows Docker daemon required")
    run(["docker", "pull", BASE], timeout=600)
    image = json.loads(run(["docker", "image", "inspect", BASE]))[0]
    if image.get("Os") != "windows" or image.get("Architecture") != "amd64":
        raise ValueError("Pinned Windows amd64 image required")
    frames = runpy.run_path(str(scripts / "calisma-ortami-denetle.py"))
    install_hashes = {n: hashlib.sha256((scripts / n).read_bytes()).hexdigest() for n in ("kur.ps1", "kaldir.ps1")}
    name = "zee-k198-" + uuid.uuid4().hex
    with tempfile.TemporaryDirectory(prefix="zee-clean-install-") as directory:
        staged = Path(directory)
        for filename in ["dil.exe", "dillsp.exe", "GIT_SHA", "SHA256SUMS"]:
            shutil.copyfile(artifact / filename, staged / filename)
        api["artefakt_dogrula"](staged, sha)
        for filename in install_hashes:
            shutil.copyfile(scripts / filename, staged / filename)
            if hashlib.sha256((staged / filename).read_bytes()).hexdigest() != install_hashes[filename]:
                raise ValueError("Staged installer bytes differ")
        (staged / "merhaba.dil").write_bytes('"Dünyaya merhaba" yaz\n'.encode("utf-8"))
        run(["icacls", str(staged), "/grant", "*S-1-5-32-545:(OI)(CI)RX", "/T"], timeout=30)
        container = ["docker", "run", "--detach", "--name", name, "--isolation", "process",
                     "--network", "none", "--user", "ContainerUser", "--env", r"PATH=C:\Windows\System32;C:\Windows",
                     "--mount", f"type=bind,src={staged},dst=C:\\zee,readonly"]
        try:
            run(container + [BASE, PS, "-NoProfile", "-NonInteractive", "-Command", "while ($true) { Start-Sleep -Seconds 5 }"])
            execute = ["docker", "exec", name]
            inventory = run(execute + ["cmd.exe", "/d", "/s", "/c",
                r"ver & for %A in (rustc.exe cargo.exe rustup.exe) do @for %P in (C:\Windows\System32 C:\Windows) do @if exist %P\%A exit /b 17"]).decode("utf-8", errors="replace")
            install = run(execute + [PS, "-NoProfile", "-NonInteractive", "-File", r"C:\zee\kur.ps1",
                                    "-Artefakt", r"C:\zee", "-Hedef", BIN, "-Veri", DATA]).decode("utf-8", errors="replace")
            script = ("$ErrorActionPreference='Stop'; [Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false); "
                      "Get-Content -Raw -Encoding UTF8 ([IO.Path]::Combine($env:ZEE_KUR_DATA,'kurulum-v1.tsv'))")
            manifest = run(["docker", "exec", "--env", "ZEE_KUR_DATA=" + DATA, name, PS,
                            "-NoProfile", "-NonInteractive", "-Command", script]).decode("utf-8")
            manifest_dogrula(manifest, sha, hashes)
            output = run(execute + [BIN + r"\dil.exe", "çalıştır", r"C:\zee\merhaba.dil"])
            if output != "Dünyaya merhaba\n".encode("utf-8"):
                raise ValueError("Installed program output differs")
            requests = [{"jsonrpc": "2.0", "id": 1, "method": "initialize",
                         "params": {"processId": None, "rootUri": None, "capabilities": {}}},
                        {"jsonrpc": "2.0", "id": 2, "method": "shutdown"},
                        {"jsonrpc": "2.0", "method": "exit"}]
            replies = frames["lsp_cevaplari"](run(["docker", "exec", "-i", name, BIN + r"\dillsp.exe"],
                                                b"".join(map(frames["lsp_cerceve"], requests))))
            api["lsp_dogrula"](replies)
            uninstall = run(execute + [PS, "-NoProfile", "-NonInteractive", "-File", r"C:\zee\kaldir.ps1", "-Veri", DATA]).decode("utf-8", errors="replace")
            check = ("$ErrorActionPreference='Stop'; "
                     "foreach ($f in @([IO.Path]::Combine($env:ZEE_KUR_BIN,'dil.exe'), "
                     "[IO.Path]::Combine($env:ZEE_KUR_BIN,'dillsp.exe'), "
                     "[IO.Path]::Combine($env:ZEE_KUR_DATA,'kurulum-v1.tsv'))) { "
                     "if (Test-Path -LiteralPath $f) { throw 'Uninstall left an owned file' } }")
            run(["docker", "exec", "--env", "ZEE_KUR_BIN=" + BIN, "--env", "ZEE_KUR_DATA=" + DATA,
                 name, PS, "-NoProfile", "-NonInteractive", "-Command", check])
        finally:
            # Yalnız bu provanın UUID adlı container'ı kaldırılır.
            container_temizle(name, sys.exc_info()[0])
    return {"sema": "zee-temiz-windows-kurulum-1", "sonuc": "gecti", "git_sha": sha,
            "prova_git_sha": os.environ.get("GITHUB_SHA", "-"), "tarih_utc": datetime.now(timezone.utc).isoformat(),
            "prova_araci_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "helper_sha256": hashlib.sha256((scripts / "temiz-windows-prova.py").read_bytes()).hexdigest(),
            "kurulum_betikleri": install_hashes, "base": BASE, "image_id": image["Id"],
            "image_os_version": image.get("OsVersion"), "repo_digests": image.get("RepoDigests"),
            "docker_server_version": info.get("ServerVersion"), "container_arguments": container[1:],
            "inventory": inventory, "rust_araclari": "yok", "kurulum": "gecti", "kurulum_log": install,
            "manifest": manifest, "kaldirma": "gecti", "kaldirma_log": uninstall, "container_temizligi": "gecti",
            "program_exit": 0, "program_ciktisi": output.decode("utf-8"), "ikililer": hashes,
            "lsp_initialize_shutdown_exit": "gecti", "lsp_cevaplari": replies,
            "sinir": "Stock Server Core, ContainerUser, ağsız read-only bind; geçici root ve host kernel paylaşımı. Seçili kur/program/LSP/kaldır; masaüstü/minimum OS veya bütün IO kanıtı değildir."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artefakt", type=Path, required=True)
    parser.add_argument("--beklenen-sha", required=True)
    parser.add_argument("--rapor", type=Path, required=True)
    args = parser.parse_args()
    try:
        report = prova(args.artefakt.resolve(), args.beklenen_sha)
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        report = {"sema": "zee-temiz-windows-kurulum-1", "sonuc": "kaldi", "git_sha": args.beklenen_sha,
                  "base": BASE, "hata": str(error)[:4000]}
    args.rapor.parent.mkdir(parents=True, exist_ok=True)
    args.rapor.write_bytes((json.dumps(report, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print("clean-windows-install: " + report["sonuc"])
    return 0 if report["sonuc"] == "gecti" else 1


if __name__ == "__main__":
    sys.exit(main())
