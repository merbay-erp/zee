#!/usr/bin/env python3
"""K-228: own empty loopback cluster, owner/runtime roles and strict native gate."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import platform
import subprocess
import tempfile
import time
import uuid

ROOT = Path(__file__).resolve().parent.parent
IMAGE = "postgres@sha256:0ea6700a3b4f0ae6ce746519073558aed4d88a79d8d07622a9a644946c7319c4"
TEST = "izole_goc_durum_kuru_checksum_rol_ve_atomik_uygulama_sinirini_korur"


def run(args, **kw):
    return subprocess.run(args, check=True, capture_output=True, timeout=kw.pop("timeout", 60), **kw).stdout.decode()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--engine", choices=("docker", "local"), default="docker")
    p.add_argument("--postgres-bin-dir")
    a = p.parse_args()
    if a.engine == "local" and not a.postgres_bin_dir:
        p.error("local requires --postgres-bin-dir; existing databases cannot be selected")
    name = "zee-goc-ci-" + uuid.uuid4().hex
    created = False
    prefix = ["runuser", "-u", "postgres", "--"] if os.geteuid() == 0 else []
    started = time.monotonic()
    timestamp = datetime.now(timezone.utc).isoformat()
    with tempfile.TemporaryDirectory(prefix="zee-goc-ci-") as temp:
        directory = Path(temp)
        data = directory / "data"
        try:
            if a.engine == "docker":
                created = True
                run(["docker", "run", "--detach", "--rm", "--name", name, "--publish", "127.0.0.1::5432",
                     "--env", "POSTGRES_HOST_AUTH_METHOD=trust", IMAGE], timeout=300)
                for _ in range(120):
                    r = subprocess.run(["docker", "exec", name, "pg_isready", "-h", "127.0.0.1", "-U", "postgres"],
                                       capture_output=True, timeout=10)
                    if r.returncode == 0:
                        break
                    time.sleep(0.25)
                else:
                    raise ValueError("own empty PostgreSQL did not become ready")
                port = int(run(["docker", "port", name, "5432/tcp"]).strip().rsplit(":", 1)[1])
                psql = ["docker", "exec", name, "psql", "-U", "postgres"]
                version = run(["docker", "exec", name, "postgres", "--version"]).strip()
            else:
                import socket
                binaries = Path(a.postgres_bin_dir).resolve()
                if prefix:
                    run(["chown", "-R", "postgres:postgres", temp])
                run(prefix + [str(binaries / "initdb"), "--no-locale", "--encoding=UTF8", "--auth=trust",
                              "--username=postgres", "--pgdata", str(data)])
                with socket.socket() as listener:
                    listener.bind(("127.0.0.1", 0))
                    port = listener.getsockname()[1]
                (data / "postgresql.conf").write_text(f"listen_addresses='127.0.0.1'\nport={port}\n"
                                                       f"unix_socket_directories='{temp}'\nssl=off\n")
                run(prefix + [str(binaries / "pg_ctl"), "--pgdata", str(data), "--wait", "--timeout=30",
                              "--log", str(directory / "server.log"), "start"])
                psql = prefix + [str(binaries / "psql"), "-h", "127.0.0.1", "-p", str(port), "-U", "postgres"]
                version = run([str(binaries / "postgres"), "--version"]).strip()
            # This psql command reaches only the cluster just created above.
            run(psql + ["-v", "ON_ERROR_STOP=1", "--command",
                        "CREATE ROLE zee_owner LOGIN; CREATE ROLE zee_runtime LOGIN;"])
            run(psql + ["-v", "ON_ERROR_STOP=1", "--command", "CREATE DATABASE zee_goc_ci OWNER zee_owner;"])
            run(psql + ["-v", "ON_ERROR_STOP=1", "--dbname", "zee_goc_ci", "--command",
                        "ALTER SCHEMA public OWNER TO zee_owner;"])
            env = {k: v for k, v in os.environ.items() if not k.startswith("ZEE_POSTGRES_GOC_CI_")}
            env["CARGO_TERM_COLOR"] = "never"
            env["ZEE_POSTGRES_GOC_CI_HEDEFI"] = f"postgresql://127.0.0.1:{port}/zee_goc_ci"
            for role in ("OWNER", "RUNTIME"):
                env[f"ZEE_POSTGRES_GOC_CI_{role}_URL"] = f"host=127.0.0.1 port={port} user=zee_{role.lower()} dbname=zee_goc_ci sslmode=disable"
            cmd = ["cargo", "test", "--locked", "--test", "postgresql_goc_plani_testi", TEST, "--",
                   "--ignored", "--exact", "--nocapture"]
            r = subprocess.run(cmd, cwd=ROOT / "compiler", env=env, capture_output=True, timeout=600)
            log = (r.stdout + r.stderr).decode(errors="replace")
            target = ROOT / "compiler/target"
            target.mkdir(exist_ok=True)
            (target / "postgresql-goc-ci.log").write_text(log)
            git = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True)
            clean = subprocess.run(["git", "status", "--porcelain"], cwd=ROOT, capture_output=True, text=True)
            files = [ROOT / "compiler/src/postgresql.rs", ROOT / "compiler/tests/postgresql_goc_plani_testi.rs", Path(__file__)]
            (target / "postgresql-goc-ci.json").write_text(json.dumps({
                "schema": "zee-postgresql-goc-ci-1", "server": version, "engine": a.engine,
                "source_sha": git.stdout.strip() if git.returncode == 0 else None,
                "source_clean": not clean.stdout.strip() if clean.returncode == 0 else None,
                "fixture_hashes": {str(f.relative_to(ROOT)): hashlib.sha256(f.read_bytes()).hexdigest() for f in files},
                "started_utc": timestamp, "host": platform.platform(),
                "rustc": run(["rustc", "--version"], cwd=ROOT / "compiler").strip(),
                "image": IMAGE if a.engine == "docker" else None, "test": TEST, "exit": r.returncode,
                "elapsed_seconds": round(time.monotonic() - started, 3),
                "boundary": "own empty loopback database and synthetic owner/runtime roles; no customer cluster or TLS claim"
            }, indent=2) + "\n")
            if (r.returncode or not re.search(rf"^test {TEST} \.\.\. ok$", log, re.MULTILINE)
                    or re.findall(r"^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", log,
                                  re.MULTILINE) != [("1", "0", "0")]):
                raise ValueError("migration native gate requires 1 passed/0 failed/0 ignored; inspect log")
            print(f"Migration plan gate: 1/1 passed, 0 ignored; {version}")
        finally:
            if a.engine == "docker" and created:
                cleanup = subprocess.run(["docker", "rm", "--force", name], capture_output=True, timeout=60)
                if cleanup.returncode and subprocess.run(["docker", "inspect", name], capture_output=True).returncode == 0:
                    raise ValueError("own migration cluster could not be removed")
            if a.engine == "local" and (data / "postmaster.pid").exists():
                run(prefix + [str(binaries / "pg_ctl"), "--pgdata", str(data), "--mode=immediate", "--wait", "stop"])


if __name__ == "__main__":
    main()
