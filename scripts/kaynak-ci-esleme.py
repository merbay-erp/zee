#!/usr/bin/env python3
"""K-224: exact Git archive / source ZIP / explicit CI-record verification.

No network, artifact download, checkout mutation or implicit HEAD acceptance.
Only the requested report is written, outside tracked/non-ignored sources.
"""

import argparse
import hashlib
import io
import json
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile
import zipfile

SHA = re.compile(r"[0-9a-f]{40}\Z")
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
MANIFEST_LINE = re.compile(r"([0-9a-f]{64})  (.+)\Z")
PREFIX = "zee/"
METADATA = "KAYNAK-SHA256.txt"
MAX_SOURCE_BYTES = 512 * 1024 * 1024
MAX_RECORD_BYTES = 4 * 1024 * 1024


class Rejected(ValueError):
    def __init__(self, code, message):
        self.code = code
        super().__init__(message)


def reject(code, message):
    raise Rejected(code, message)


def sha256(value):
    return hashlib.sha256(value).hexdigest()


def source_path(value):
    if (not value or value.startswith("/") or "\\" in value or "\x00" in value
            or any(part in ("", ".", "..") for part in value.split("/"))):
        reject("SOURCE_PATH", "Kaynak yolu kanonik göreli POSIX yolu olmalı.")
    return value


def git(repo, *args):
    try:
        return subprocess.run(["git", "-C", str(repo), *args], check=True,
                              capture_output=True, timeout=60).stdout
    except (OSError, subprocess.SubprocessError):
        reject("GIT_SOURCE", "Exact Git kaynağı salt okunur olarak alınamadı.")


def archive_hashes(repo, commit):
    if not SHA.fullmatch(commit):
        reject("SOURCE_SHA", "Kaynak kimliği tam küçük harfli 40 haneli commit SHA olmalı.")
    if git(repo, "rev-parse", commit + "^{commit}").decode().strip() != commit:
        reject("SOURCE_SHA", "Kaynak kimliği exact commit nesnesi olmalı.")
    raw = git(repo, "archive", "--format=tar", commit)
    if len(raw) > MAX_SOURCE_BYTES:
        reject("SOURCE_BUDGET", "Kaynak arşivi denetleme boyut bütçesini aşıyor.")
    result = {}
    with tarfile.open(fileobj=io.BytesIO(raw)) as archive:
        for item in archive:
            if item.isdir():
                source_path(item.name.rstrip("/"))
                continue
            name = source_path(item.name)
            if not item.isfile() or name in result or name == METADATA:
                reject("ARCHIVE_ENTRY", "Git arşivi yalnız tekil normal kaynak dosyaları taşımalı.")
            result[name] = sha256(archive.extractfile(item).read())
    if not result:
        reject("EMPTY_SOURCE", "Kaynak arşivi boş olamaz.")
    return result


def zip_hashes(raw, commit, archive):
    result, directories, manifest = {}, [], None
    with zipfile.ZipFile(io.BytesIO(raw)) as snapshot:
        if snapshot.comment != commit.encode("ascii"):
            reject("ZIP_COMMENT", "ZIP yorumu exact kaynak commit'i ile eşleşmiyor.")
        names = set()
        if sum(item.file_size for item in snapshot.infolist()) > MAX_SOURCE_BYTES:
            reject("SOURCE_BUDGET", "ZIP kaynakları denetleme boyut bütçesini aşıyor.")
        for item in snapshot.infolist():
            if item.filename in names:
                reject("ZIP_DUPLICATE", "ZIP içinde yinelenen yol var.")
            names.add(item.filename)
            if not item.filename.startswith(PREFIX):
                reject("SOURCE_PATH", "ZIP kaynakları yalnız zee/ kökünde bulunmalı.")
            relative = item.filename[len(PREFIX):]
            if item.is_dir():
                if relative:
                    directories.append(source_path(relative.rstrip("/")))
                continue
            name = source_path(relative)
            kind = (item.external_attr >> 16) & 0o170000
            if kind not in (0, 0o100000):
                reject("ZIP_ENTRY", "ZIP kaynak dosyası normal dosya olmalı.")
            if name == METADATA:
                if item.file_size > MAX_RECORD_BYTES:
                    reject("MANIFEST_BUDGET", "Kaynak manifesti boyut bütçesini aşıyor.")
                manifest = snapshot.read(item)
            else:
                result[name] = sha256(snapshot.read(item))
        for directory in directories:
            if not any(name.startswith(directory + "/") for name in result):
                reject("ZIP_EXTRA", "ZIP içinde kaynak ağacına ait olmayan dizin var.")
    if manifest is None:
        reject("MANIFEST_MISSING", "ZIP kaynak SHA-256 manifestini taşımalı.")
    declared = {}
    for line in manifest.decode("utf-8").splitlines():
        match = MANIFEST_LINE.fullmatch(line)
        if not match:
            reject("MANIFEST_FORMAT", "Manifest satırı SHA-256 ve kanonik kaynak yolu taşımalı.")
        digest, name = match.groups()
        source_path(name)
        if name in declared:
            reject("MANIFEST_DUPLICATE", "Manifestte yinelenen kaynak yolu var.")
        declared[name] = digest
    if archive.keys() != result.keys() or archive.keys() != declared.keys():
        reject("SOURCE_SET", "Git, ZIP ve manifest kaynak dosyası kümeleri eş değil.")
    if archive != result or archive != declared:
        reject("SOURCE_HASH", "Git, ZIP ve manifest kaynak SHA-256 değerleri eş değil.")
    return sha256(manifest)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            reject("CI_DUPLICATE_KEY", "CI JSON kaydında yinelenen alan var.")
        result[key] = value
    return result


def ci_records(path, commit):
    if path.stat().st_size > MAX_RECORD_BYTES:
        reject("CI_BUDGET", "CI kayıt dosyası boyut bütçesini aşıyor.")
    raw = path.read_bytes()
    data = json.loads(raw, object_pairs_hook=unique_object)
    if not isinstance(data, dict):
        reject("CI_SCHEMA", "CI kaydı şemalı nesne olmalı.")
    if data.get("schema") == "zee-ci-kaydi-1":
        entries = data.get("records")
    elif data.get("schema") == "zee-source-ci-evidence-1":
        ci = data.get("ci", {})
        if not isinstance(ci, dict):
            reject("CI_SCHEMA", "CI kanıt alanı nesne olmalı.")
        entries = []
        for value in ci.values():
            if not isinstance(value, dict):
                continue
            source = value.get("record", value)
            if not isinstance(source, dict):
                reject("CI_RECORD", "CI koşusu nesne olmalı.")
            record = dict(source)
            if "head_context_commit" in record:
                if record.get("source_sha", record["head_context_commit"]) != record["head_context_commit"]:
                    reject("CI_RECORD", "CI koşusunda çelişen kaynak kimlikleri var.")
                record["source_sha"] = record.pop("head_context_commit")
            if "evidence_level" in value:
                record["evidence_level"] = value["evidence_level"]
            entries.append(record)
    else:
        reject("CI_SCHEMA", "CI kaydı desteklenen açık şema kimliği taşımalı.")
    if not isinstance(entries, list) or not entries:
        reject("CI_RECORD", "CI kaydı en az bir koşu kaydı taşımalı.")
    records, seen = [], set()
    for record in entries:
        if not isinstance(record, dict):
            reject("CI_RECORD", "CI koşusu nesne olmalı.")
        fields = {name: record.get(name) for name in
                  ("provider", "run_index", "source_sha", "status", "evidence_level")}
        if (not isinstance(fields["provider"], str) or not fields["provider"]
                or not isinstance(fields["evidence_level"], str) or not fields["evidence_level"]
                or type(fields["run_index"]) is not int or fields["run_index"] < 1
                or not isinstance(fields["source_sha"], str) or not SHA.fullmatch(fields["source_sha"])
                or fields["status"] not in ("success", "failure", "cancelled", "pending", "running")):
            reject("CI_RECORD", "CI koşusu sağlayıcı, sıra, exact SHA, durum ve kanıt düzeyi taşımalı.")
        identity = fields["provider"], fields["run_index"]
        if identity in seen:
            reject("CI_DUPLICATE_RUN", "CI kaydında yinelenen koşu kimliği var.")
        seen.add(identity)
        fields["exact_source"] = fields["source_sha"] == commit
        records.append(fields)
    exact = [record for record in records if record["exact_source"]]
    if not exact:
        reject("CI_EXACT_SOURCE", "Compiler eşliği exact kaynak SHA'sına ait CI kaydı yerine geçmez.")
    if not any(record["status"] == "success" for record in exact):
        reject("CI_SUCCESS", "Exact kaynak SHA'sına ait başarılı CI durum kaydı yok.")
    return sha256(raw), records


def safe_output(repo, output, inputs):
    if output in inputs:
        reject("REPORT_SOURCE", "Rapor kaynak ZIP veya CI kaydının üzerine yazamaz.")
    try:
        relative = output.relative_to(repo)
    except ValueError:
        return
    if relative.parts and relative.parts[0] == ".git":
        reject("REPORT_SOURCE", "Rapor Git metadata yoluna yazılamaz.")
    allowed = subprocess.run(["git", "-C", str(repo), "check-ignore", "--quiet", "--",
                              relative.as_posix()], capture_output=True, timeout=60)
    if allowed.returncode != 0:
        reject("REPORT_SOURCE", "Depo içi rapor yalnız Git dışı hedef dizinine yazılabilir.")


def write_report(path, report):
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=path.parent,
                                     prefix=".kaynak-ci-", delete=False) as stream:
        temporary = Path(stream.name)
        json.dump(report, stream, ensure_ascii=False, sort_keys=True, indent=2)
        stream.write("\n")
    try:
        temporary.replace(path)
    finally:
        temporary.unlink(missing_ok=True)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--depo", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--sha", required=True)
    parser.add_argument("--zip", type=Path, required=True)
    parser.add_argument("--zip-sha256", required=True, help="Güvenilir sidecar/teslim kaydının ZIP SHA-256 değeri")
    parser.add_argument("--ci-kaydi", type=Path, required=True)
    parser.add_argument("--rapor", type=Path, required=True)
    args = parser.parse_args(argv)
    repo, output = args.depo.resolve(), args.rapor.resolve()
    inputs = {args.zip.resolve(), args.ci_kaydi.resolve()}
    try:
        safe_output(repo, output, inputs)
    except (Rejected, OSError, subprocess.SubprocessError) as error:
        print("Kaynak/CI eşleme reddedildi: " + str(error))
        return 2
    try:
        if not SHA256.fullmatch(args.zip_sha256):
            reject("ZIP_SHA256", "Beklenen ZIP özeti tam küçük harfli SHA-256 olmalı.")
        if args.zip.stat().st_size > MAX_SOURCE_BYTES:
            reject("SOURCE_BUDGET", "ZIP teslimi denetleme boyut bütçesini aşıyor.")
        zip_bytes = args.zip.read_bytes()
        if len(zip_bytes) > MAX_SOURCE_BYTES:
            reject("SOURCE_BUDGET", "ZIP teslimi denetleme boyut bütçesini aşıyor.")
        zip_sha = sha256(zip_bytes)
        if zip_sha != args.zip_sha256:
            reject("ZIP_SHA256", "ZIP baytları beklenen teslim SHA-256 değeriyle eşleşmiyor.")
        archive = archive_hashes(repo, args.sha)
        manifest_sha = zip_hashes(zip_bytes, args.sha, archive)
        record_sha, records = ci_records(args.ci_kaydi, args.sha)
        fingerprint = sha256("".join(digest + "  " + name + "\n"
                                     for name, digest in sorted(archive.items())).encode())
        report = {"schema": "zee-kaynak-ci-esleme-1", "status": "passed",
                  "source_sha": args.sha, "source_count": len(archive),
                  "source_manifest_sha256": fingerprint, "zip_sha256": zip_sha,
                  "zip_comment": args.sha, "zip_manifest_sha256": manifest_sha,
                  "ci_record_sha256": record_sha, "ci_records": records,
                  "ci_acceptance": "explicit-recorded-commit-status",
                  "network_queried": False, "artifact_download": False,
                  "job_logs_read_by_tool": False, "runtime_release_acceptance": False}
    except Rejected as error:
        report = {"schema": "zee-kaynak-ci-esleme-1", "status": "rejected",
                  "error_code": error.code, "reason": str(error)}
    except (OSError, ValueError, RuntimeError, TypeError, tarfile.TarError, zipfile.BadZipFile):
        report = {"schema": "zee-kaynak-ci-esleme-1", "status": "rejected",
                  "error_code": "INPUT_READ", "reason": "Kaynak veya CI girdi dosyası okunamadı."}
    try:
        write_report(output, report)
    except OSError:
        print("Kaynak/CI eşleme reddedildi: sonuç raporu yazılamadı.")
        return 2
    print("Kaynak/CI eşleme: " + report["status"])
    return 0 if report["status"] == "passed" else 2


if __name__ == "__main__":
    raise SystemExit(main())
