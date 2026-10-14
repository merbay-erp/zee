#!/usr/bin/env python3
"""K-222/GB-022: own ephemeral PostgreSQL, CA/key and strict TLS libtest gate."""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import tempfile
import time
import uuid

ROOT = Path(__file__).resolve().parent.parent
# Official postgres 16.15-bookworm multiarch index; amd64 and arm64 are available.
IMAGE = "postgres@sha256:0ea6700a3b4f0ae6ce746519073558aed4d88a79d8d07622a9a644946c7319c4"
CA_ENV_PREFIX = "ZEE_POSTGRES_CI_"
EXPECTED_TESTS = (
    "izole_tls_dogru_ca_hostname_ve_tls12_ile_baglanir",
    "izole_tls_baska_ca_gercek_sunucuyu_reddeder",
    "izole_tls_san_disindaki_ip_hostnameini_reddeder",
    "izole_tls_null_ve_metin_disini_reddeder_transaction_sagligi_korunur",
    "izole_tls_sorgu_parametre_sonuc_ve_goc_sinirlarini_bir_fazlasinda_reddeder",
)


def run(args, **kwargs):
    return subprocess.run(args, check=True, capture_output=True,
                          timeout=kwargs.pop("timeout", 60), **kwargs).stdout.decode()


def check_results(transcript, returncode):
    if returncode:
        raise ValueError(f"TLS libtest failed with exit {returncode}")
    for test in EXPECTED_TESTS:
        if not re.search(rf"^test {re.escape(test)} \.\.\. ok$", transcript, re.MULTILINE):
            raise ValueError(f"required TLS test did not pass: {test}")
    summaries = re.findall(
        r"^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;",
        transcript,
        re.MULTILINE,
    )
    if summaries != [(str(len(EXPECTED_TESTS)), "0", "0")]:
        raise ValueError(f"TLS gate requires exactly 5 passed/0 failed/0 ignored: {summaries}")


def certificates(directory):
    ca = directory / "ca.crt"
    ca_key = directory / "ca.key"
    wrong_ca = directory / "wrong-ca.crt"
    for cert, key, common_name in [
        (ca, ca_key, "zee-ephemeral-test-ca"),
        (wrong_ca, directory / "wrong-ca.key", "zee-ephemeral-wrong-ca"),
    ]:
        run(["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
             "-subj", f"/CN={common_name}", "-addext", "basicConstraints=critical,CA:TRUE",
             "-addext", "keyUsage=critical,keyCertSign,cRLSign", "-keyout", str(key), "-out", str(cert)])
    key = directory / "server.key"
    csr = directory / "server.csr"
    cert = directory / "server.crt"
    extensions = directory / "server.ext"
    extensions.write_text("basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\n"
                          "extendedKeyUsage=serverAuth\nsubjectAltName=DNS:localhost\n")
    run(["openssl", "req", "-new", "-newkey", "rsa:2048", "-nodes", "-subj", "/CN=localhost",
         "-keyout", str(key), "-out", str(csr)])
    run(["openssl", "x509", "-req", "-in", str(csr), "-CA", str(ca), "-CAkey", str(ca_key),
         "-CAcreateserial", "-days", "1", "-extfile", str(extensions), "-out", str(cert)])
    key.chmod(0o600)
    return ca, wrong_ca, cert, key


class DockerCluster:
    def __init__(self, directory, cert, key):
        self.name = "zee-tls-ci-" + uuid.uuid4().hex
        self.created = False
        self.directory, self.cert, self.key = directory, cert, key

    def start(self):
        run(["docker", "info", "--format", "{{.ServerVersion}}"])
        # Docker itself pulls this immutable index if it is absent; no mutable tag is used.
        self.created = True
        run(["docker", "run", "--detach", "--rm", "--name", self.name,
             "--publish", "127.0.0.1::5432", "--env", "POSTGRES_HOST_AUTH_METHOD=trust",
             "--entrypoint", "sh", IMAGE, "-c",
             "while [ ! -f /tmp/zee-tls-ready ]; do sleep 0.1; done; "
             "exec docker-entrypoint.sh postgres -c ssl=on -c ssl_min_protocol_version=TLSv1.2 "
             "-c ssl_max_protocol_version=TLSv1.2 -c ssl_cert_file=/tmp/server.crt "
             "-c ssl_key_file=/tmp/server.key -c hba_file=/tmp/zee-pg-hba.conf"], timeout=300)
        for source, target in [(self.cert, "server.crt"), (self.key, "server.key"),
                               (self.directory / "pg_hba.conf", "zee-pg-hba.conf")]:
            run(["docker", "cp", str(source), f"{self.name}:/tmp/{target}"])
        run(["docker", "exec", self.name, "chown", "postgres:postgres", "/tmp/server.key", "/tmp/server.crt"])
        run(["docker", "exec", self.name, "touch", "/tmp/zee-tls-ready"])
        for _ in range(120):
            ready = subprocess.run(["docker", "exec", self.name, "pg_isready", "-h", "127.0.0.1", "-U", "postgres"],
                                   capture_output=True, timeout=10)
            if ready.returncode == 0:
                break
            time.sleep(0.25)
        else:
            raise ValueError("ephemeral Docker PostgreSQL did not become ready")
        self.port = int(run(["docker", "port", self.name, "5432/tcp"]).strip().rsplit(":", 1)[1])
        self.version = run(["docker", "exec", self.name, "postgres", "--version"]).strip()
        self.psql = ["docker", "exec", self.name, "psql"]
        self.psql_port = 5432
        self.identity = {"image": IMAGE, "platform": run(["docker", "image", "inspect", IMAGE,
                         "--format", "{{.Os}}/{{.Architecture}} {{.Id}}"]).strip()}

    def close(self):
        if self.created:
            cleanup = subprocess.run(["docker", "rm", "--force", self.name], capture_output=True, timeout=60)
            if cleanup.returncode:
                exists = subprocess.run(["docker", "container", "inspect", self.name],
                                        capture_output=True, timeout=60)
                if exists.returncode == 0:
                    raise ValueError("ephemeral Docker PostgreSQL could not be removed")


class LocalCluster:
    def __init__(self, directory, cert, key, binaries):
        self.directory, self.cert, self.key = directory, cert, key
        self.binaries = Path(binaries).resolve()
        self.data = directory / "data"
        self.started = False
        # CI is its own job container. The generated directory is the only owned path.
        self.prefix = ["runuser", "-u", "postgres", "--"] if os.geteuid() == 0 else []

    def start(self):
        for binary in ["initdb", "postgres", "pg_ctl", "psql"]:
            if not (self.binaries / binary).is_file():
                raise ValueError(f"missing isolated PostgreSQL binary: {binary}")
        if self.prefix:
            run(["chown", "-R", "postgres:postgres", str(self.directory)])
        run(self.prefix + [str(self.binaries / "initdb"), "--no-locale", "--encoding=UTF8", "--auth=trust",
                           "--username=postgres", "--pgdata", str(self.data)])
        # Reserve no existing service: pg_ctl tries a generated loopback port and fails if occupied.
        import socket
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            self.port = listener.getsockname()[1]
        config = (f"listen_addresses='127.0.0.1'\nport={self.port}\n"
                  f"unix_socket_directories='{self.directory}'\nssl=on\n"
                  "ssl_min_protocol_version='TLSv1.2'\nssl_max_protocol_version='TLSv1.2'\n"
                  f"ssl_cert_file='{self.cert}'\nssl_key_file='{self.key}'\n"
                  f"hba_file='{self.directory / 'pg_hba.conf'}'\n")
        (self.data / "postgresql.conf").write_text(config)
        run(self.prefix + [str(self.binaries / "pg_ctl"), "--pgdata", str(self.data), "--wait",
                           "--timeout=30", "--log", str(self.directory / "server.log"), "start"])
        self.started = True
        self.version = run([str(self.binaries / "postgres"), "--version"]).strip()
        self.psql = self.prefix + [str(self.binaries / "psql")]
        self.psql_port = self.port
        self.identity = {"binaries": str(self.binaries), "platform": platform.platform()}

    def close(self):
        # pg_ctl status also catches a failed --wait after the server process started.
        if self.data.is_dir() and (self.data / "postmaster.pid").exists():
            run(self.prefix + [str(self.binaries / "pg_ctl"), "--pgdata", str(self.data),
                               "--mode=immediate", "--wait", "stop"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", choices=("docker", "local"), default="docker")
    parser.add_argument("--postgres-bin-dir", help="local engine's initdb/pg_ctl/postgres/psql directory")
    parser.add_argument("--output", type=Path, default=ROOT / "compiler/target/postgresql-tls-ci.json")
    args = parser.parse_args()
    if args.engine == "local" and not args.postgres_bin_dir:
        parser.error("local engine requires --postgres-bin-dir; no existing database may be used")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    source = run(["git", "-C", str(ROOT), "rev-parse", "HEAD"]).strip()
    timestamp = datetime.now(timezone.utc).isoformat()
    started = time.monotonic()
    with tempfile.TemporaryDirectory(prefix="zee-postgresql-tls-ci-") as temp:
        directory = Path(temp)
        ca, wrong_ca, cert, key = certificates(directory)
        (directory / "pg_hba.conf").write_text("local all all trust\nhostssl all all 0.0.0.0/0 trust\nhostnossl all all 0.0.0.0/0 reject\n")
        cluster = (DockerCluster(directory, cert, key) if args.engine == "docker" else
                   LocalCluster(directory, cert, key, args.postgres_bin_dir))
        try:
            cluster.start()
            # Independent connectivity preflight at 127.0.0.1 proves the negative hostname
            # case reaches the same server. psql's require deliberately omits identity checks.
            url = f"host=127.0.0.1 port={cluster.psql_port} user=postgres dbname=postgres sslmode=require"
            preflight = run(cluster.psql + ["--no-psqlrc", "--dbname", url, "--tuples-only", "--no-align",
                                            "--command", "SELECT ssl::text || ':' || version FROM pg_stat_ssl WHERE pid=pg_backend_pid()"])
            if preflight.strip() != "true:TLSv1.2":
                raise ValueError("fixture preflight did not negotiate TLS 1.2")
            env = {k: v for k, v in os.environ.items() if not k.startswith(CA_ENV_PREFIX)}
            env["CARGO_TERM_COLOR"] = "never"
            for name, host, root in [("DOGRU", "localhost", ca), ("YANLIS_CA", "localhost", wrong_ca),
                                     ("YANLIS_HOST", "127.0.0.1", ca)]:
                prefix = f"{CA_ENV_PREFIX}{name}"
                env[prefix + "_HEDEFI"] = f"postgresql://{host}:{cluster.port}/postgres"
                env[prefix + "_URL"] = f"host={host} port={cluster.port} user=postgres dbname=postgres sslmode=require"
                env[prefix + "_URL_TLS_CA_PEM"] = root.read_text()
            command = ["cargo", "test", "--locked", "--test", "postgresql_testi", "izole_tls_", "--",
                       "--ignored", "--test-threads=1", "--nocapture"]
            result = subprocess.run(command, cwd=ROOT / "compiler", env=env, capture_output=True, timeout=600)
            transcript = (result.stdout + result.stderr).decode(errors="replace")
            args.output.with_suffix(".log").write_text(transcript)
            hashes = {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
                      for path in [Path(__file__), ROOT / "compiler/tests/postgresql_testi.rs"]}
            report = {"schema": "zee-postgresql-tls-ci-1", "source_sha": source,
                      "started_utc": timestamp, "host": platform.platform(),
                      "rustc": run(["rustc", "--version"], cwd=ROOT / "compiler").strip(),
                      "source_clean": not run(["git", "-C", str(ROOT), "status", "--porcelain"]).strip(),
                      "fixture_hashes": hashes, "engine": args.engine, "server": cluster.version,
                      "server_identity": cluster.identity, "ca_sha256": hashlib.sha256(ca.read_bytes()).hexdigest(),
                      "tests": list(EXPECTED_TESTS), "command": command, "exit": result.returncode,
                      "elapsed_seconds": round(time.monotonic() - started, 3),
                      "boundary": "ephemeral loopback-only synthetic CA/key and cluster; TLS1.2 positive, wrong CA/SAN, real NULL/non-text and transaction health; no provider rotation, TLS1.1 refusal or production cluster evidence"}
            args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
            check_results(transcript, result.returncode)
            print(f"PostgreSQL TLS/resource gate: 5/5 passed, 0 ignored; {cluster.version}; report={args.output}")
        finally:
            cluster.close()


if __name__ == "__main__":
    main()
