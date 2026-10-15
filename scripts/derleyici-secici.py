#!/usr/bin/env python3
"""K-233 pilot: offline exact project compiler pin, authenticated local bundles."""
import argparse
from contextlib import contextmanager
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import stat
import subprocess
import sys
import tempfile

PIN_FILE = "derleyici.kilit.json"
PIN_FIELDS = {"schema", "version", "source_sha", "channel", "platform"}
META = {"GIT_SHA", "SHA256SUMS", "SHA256SUMS.zee-imza", "sbom.spdx.json", "provenance.intoto.json"}
CHANNELS = {"stable", "beta", "nightly", "development"}
PLATFORMS = {"aarch64-apple-darwin", "x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu", "x86_64-pc-windows-msvc"}
LIMITS = {"pin": 64 * 1024, "config": 64 * 1024, "signature": 64 * 1024,
          "catalog": 2 * 1024 * 1024, "sbom": 4 * 1024 * 1024,
          "provenance": 16 * 1024 * 1024, "executable": 64 * 1024 * 1024,
          "checksums": 64 * 1024, "git_sha": 64}
META_LIMITS = {"GIT_SHA": LIMITS["git_sha"], "SHA256SUMS": LIMITS["checksums"],
               "SHA256SUMS.zee-imza": LIMITS["signature"],
               "sbom.spdx.json": LIMITS["sbom"], "provenance.intoto.json": LIMITS["provenance"]}
HOST_FLAGS = {"--güvenli", "--guvenli", "--deneysel-web", "--web-proxy",
              "--güvenli-web-proxy", "--web-worker-port"}
SOURCE_URI = "git+https://github.com/merbay-erp/zee@"
HEX64 = re.compile(r"[0-9a-f]{64}\Z")
HEX40 = re.compile(r"[0-9a-f]{40}\Z")


class Reject(ValueError):
    pass


def require(ok, message):
    if not ok:
        raise Reject(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def no_duplicate_keys(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "yinelenen JSON alanı")
        result[key] = value
    return result


def load_json(data):
    try:
        return json.loads(data, object_pairs_hook=no_duplicate_keys,
                          parse_constant=lambda _: (_ for _ in ()).throw(Reject("JSON sabiti geçersiz")))
    except Reject:
        raise
    except (UnicodeDecodeError, json.JSONDecodeError, RecursionError, ValueError):
        raise Reject("geçersiz UTF-8/JSON veya JSON derinlik sınırı") from None


def regular_bytes(path, limit=LIMITS["pin"]):
    path = Path(path)
    before = path.lstat()
    require(stat.S_ISREG(before.st_mode), f"düzenli gerçek dosya gerekli: {path.name}")
    require(type(limit) is int and limit > 0 and before.st_size <= limit, "dosya sınırı aşıldı")
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0) | getattr(os, "O_CLOEXEC", 0)
    fd = os.open(path, flags)
    try:
        opened = os.fstat(fd)
        require(stat.S_ISREG(opened.st_mode), "açılan dosya düzenli değil")
        require((before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) ==
                (opened.st_dev, opened.st_ino, opened.st_size, opened.st_mtime_ns), "dosya doğrulama sırasında değişti")
        stream = os.fdopen(fd, "rb")
        fd = None
        with stream:
            data = stream.read(limit + 1)
            after = os.fstat(stream.fileno())
        require(len(data) <= limit, "dosya okuma sınırı aşıldı")
        require(stat.S_ISREG(after.st_mode) and len(data) == opened.st_size and
                (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns) ==
                (opened.st_dev, opened.st_ino, opened.st_size, opened.st_mtime_ns), "dosya okunurken değişti")
        current = path.lstat()
        require(stat.S_ISREG(current.st_mode) and
                (current.st_dev, current.st_ino, current.st_size, current.st_mtime_ns) ==
                (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns), "dosya yolu okuma sırasında değişti")
        return data
    finally:
        if fd is not None:
            os.close(fd)


def root_directory(path):
    resolved = Path(path).resolve(strict=True)
    require(resolved.is_dir(), "gerçek klasör gerekli")
    return resolved


def child_directory(parent, name):
    child = parent / name
    require(child.is_dir() and not child.is_symlink(), f"yerel exact paket yok veya symlink: {child}")
    require(child.resolve().parent == parent, "paket yolu depo dışına çıkamaz")
    return child


def host_platform():
    system, machine = platform.system(), platform.machine().lower()
    values = {("Darwin", "arm64"): "aarch64-apple-darwin", ("Linux", "x86_64"): "x86_64-unknown-linux-gnu",
              ("Linux", "aarch64"): "aarch64-unknown-linux-gnu", ("Windows", "amd64"): "x86_64-pc-windows-msvc"}
    require((system, machine) in values, "host platformu bu pilot seçici tarafından desteklenmiyor")
    return values[(system, machine)]


def validate_pin(pin, pilot):
    require(isinstance(pin, dict) and set(pin) == PIN_FIELDS, "pin şeması exact beş alan ister; path/command/key alanı kabul edilmez")
    require(pin["schema"] == "zee-derleyici-pin-1", "pin şeması bilinmiyor")
    require(isinstance(pin["source_sha"], str) and HEX40.fullmatch(pin["source_sha"]), "pin source SHA exact 40 küçük hex ister")
    require(isinstance(pin["version"], str) and re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?", pin["version"]), "pin SemVer geçersiz")
    require(isinstance(pin["channel"], str) and isinstance(pin["platform"], str) and pin["channel"] in CHANNELS and pin["platform"] in PLATFORMS, "pin kanal/platform bilinmiyor")
    require(pin["platform"] == host_platform(), "başka platform ikilisi seçilemez")
    require(not (pin["channel"] == "stable" and "-" in pin["version"]), "ön sürüm kararlı kanal olamaz")
    require(pilot or pin["channel"] != "development", "development kanal explicit --pilot ister")
    return pin


def identity(record):
    return tuple(record[field] for field in ("source_sha", "version", "channel", "platform"))


def trusted_release(trust, pin):
    require(isinstance(trust, dict) and set(trust) == {"schema", "publishers", "releases"}, "güven kataloğu şeması geçersiz")
    require(trust["schema"] == "zee-derleyici-guven-1", "güven kataloğu sürümü bilinmiyor")
    publishers = trust["publishers"]
    require(isinstance(publishers, list) and all(isinstance(key, str) for key in publishers) and len(publishers) == len(set(publishers)), "yinelenen yayıncı kimliği")
    for key in publishers:
        require(isinstance(key, str) and re.fullmatch(r"sha256:[0-9a-f]{64}", key), "yayıncı kimliği geçersiz")
    require(isinstance(trust["releases"], list), "release listesi gerekli")
    seen, selected = set(), None
    for record in trust["releases"]:
        require(isinstance(record, dict) and set(record) == PIN_FIELDS - {"schema"} | {"publisher", "file_hashes"}, "release kaydı exact alan ister")
        require(isinstance(record["source_sha"], str) and HEX40.fullmatch(record["source_sha"]), "katalog source SHA biçimi geçersiz")
        require(isinstance(record["channel"], str) and isinstance(record["platform"], str) and record["channel"] in CHANNELS and record["platform"] in PLATFORMS, "katalog kanal/platform bilinmiyor")
        require(isinstance(record["version"], str) and re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?", record["version"]), "katalog SemVer geçersiz")
        key = identity(record)
        require(key not in seen, "yinelenen exact release kimliği")
        seen.add(key)
        require(isinstance(record["publisher"], str) and record["publisher"] in publishers, "release yayıncısı trusted allowlist'te yok")
        hashes = record["file_hashes"]
        require(isinstance(hashes, dict) and set(hashes) == META, "sidecar hash kümesi exact olmalı")
        require(all(isinstance(value, str) and HEX64.fullmatch(value) for value in hashes.values()), "sidecar hash biçimi geçersiz")
        if key == identity(pin):
            selected = record
    require(selected is not None, "exact pin güven kataloğunda yok; latest/fallback seçilmez")
    return selected


def binary_names(pin):
    suffix = ".exe" if pin["platform"].endswith("windows-msvc") else ""
    return {"dil" + suffix, "dillsp" + suffix}


def checksums(data, names):
    try:
        text = data.decode("ascii")
    except UnicodeDecodeError as error:
        raise Reject("SHA256SUMS ASCII değil") from error
    result = {}
    for line in text.splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  ([A-Za-z.]+)", line)
        require(match is not None, "SHA256SUMS yolu/hash biçimi geçersiz")
        sha, name = match.groups()
        require(name in names and name not in result, "SHA256SUMS yinelenen veya izin verilmeyen executable adı")
        result[name] = sha
    require(set(result) == names, "dil ve dillsp ikilisi birlikte zorunlu")
    return result


def validate_metadata(files, pin, sums, release):
    require(files["GIT_SHA"] == (pin["source_sha"] + "\n").encode(), "GIT_SHA pin ile uyuşmuyor")
    signature = load_json(files["SHA256SUMS.zee-imza"])
    require(isinstance(signature, dict) and set(signature) == {"sema", "dosya", "sha256", "anahtar_kimligi", "acik_anahtar", "ed25519"}, "imza exact mevcut şemayı ister")
    require(signature["sema"] == "zee-surum-imza-v1" and signature["dosya"] == "SHA256SUMS", "imza alanı/dosyası geçersiz")
    require(signature["sha256"] == digest(files["SHA256SUMS"]) and signature["anahtar_kimligi"] == release["publisher"], "imza hash/yayıncı farklı")
    sbom = load_json(files["sbom.spdx.json"])
    require(isinstance(sbom, dict), "SBOM nesne olmalı")
    require(sbom.get("@context") == "https://spdx.org/rdf/3.0.1/spdx-context.jsonld", "SPDX context bilinmiyor")
    graph = sbom.get("@graph")
    require(isinstance(graph, list) and all(isinstance(node, dict) for node in graph), "SPDX graph geçersiz")
    packages = [node for node in graph if node.get("type") == "software_Package" and node.get("name") == "dil"]
    require(len(packages) == 1, "SBOM tek dil kök paketi ister")
    require(packages[0].get("software_packageVersion") == pin["version"] and
            packages[0].get("software_downloadLocation") == SOURCE_URI + pin["source_sha"], "SBOM sürüm/kaynak pin ile farklı")
    found = {}
    for node in graph:
        if node.get("type") == "software_File":
            name = node.get("name")
            require(isinstance(name, str) and name in sums and name not in found, "SBOM executable adı eksik/yinelenen/izinsiz")
            hashes = node.get("verifiedUsing")
            require(hashes == [{"type": "Hash", "algorithm": "sha256", "hashValue": sums[name]}], "SBOM ikili hash farklı")
            found[name] = sums[name]
    require(found == sums, "SBOM bütün binary hash'lerini taşımalı")
    provenance = load_json(files["provenance.intoto.json"])
    require(isinstance(provenance, dict), "provenance nesne olmalı")
    require(provenance.get("_type") == "https://in-toto.io/Statement/v1" and
            provenance.get("predicateType") == "https://slsa.dev/provenance/v1", "provenance şeması bilinmiyor")
    subjects = provenance.get("subject")
    require(isinstance(subjects, list) and len(subjects) == len(sums) and all(isinstance(node, dict) and isinstance(node.get("name"), str) for node in subjects), "provenance subject sayısı farklı")
    require({node.get("name"): node.get("digest") for node in subjects} == {name: {"sha256": sha} for name, sha in sums.items()}, "provenance binary hash farklı")
    predicate = provenance.get("predicate", {})
    require(isinstance(predicate, dict), "provenance predicate nesne olmalı")
    definition = predicate.get("buildDefinition", {})
    require(isinstance(definition, dict), "provenance buildDefinition nesne olmalı")
    parameters = definition.get("externalParameters", {})
    require(isinstance(parameters, dict), "provenance externalParameters nesne olmalı")
    require(definition.get("buildType") == "https://zee-lang.dev/surum/tekrar-uretilebilir/v1", "buildType farklı")
    for field, value in (("gitCommit", pin["source_sha"]), ("surum", pin["version"]), ("platform", pin["platform"])):
        require(parameters.get(field) == value, f"provenance {field} pin ile farklı")
    require(definition.get("resolvedDependencies") == [{"uri": SOURCE_URI + pin["source_sha"], "digest": {"gitCommit": pin["source_sha"]}}], "provenance kaynak dependency farklı")
    repetition = predicate.get("zeeTekrarUretim", {})
    require(isinstance(repetition, dict), "tekrar üretim kaydı nesne olmalı")
    require(repetition.get("ikiTemizKlonEsit") is True, "iki temiz klon eşlik kanıtı yok")
    details = predicate.get("runDetails", {})
    require(isinstance(details, dict), "provenance runDetails nesne olmalı")
    byproducts = details.get("byproducts")
    expected = {"SHA256SUMS", "SHA256SUMS.ikinci-klon"}
    require(isinstance(byproducts, list) and len(byproducts) == 2 and all(isinstance(node, dict) and isinstance(node.get("name"), str) for node in byproducts) and {node.get("name") for node in byproducts} == expected, "iki klon byproduct kümesi farklı")
    require(all(node.get("digest") == {"sha256": digest(files["SHA256SUMS"])} for node in byproducts), "iki klon hashleri farklı")
    epoch = parameters.get("sourceDateEpoch")
    require(type(epoch) is int and 0 <= epoch <= 253402300799, "kaynak epoch geçersiz")
    return datetime.fromtimestamp(epoch, timezone.utc).strftime("%Y-%m-%d")


def safe_environment():
    return {key: value for key, value in os.environ.items() if not key.upper().startswith(("LD_", "DYLD_", "PYTHON"))}


def verify_signed_bytes(data, signature_data, name, key, verifier_data):
    signature = load_json(signature_data)
    require(isinstance(signature, dict) and set(signature) == {"sema", "dosya", "sha256", "anahtar_kimligi", "acik_anahtar", "ed25519"}, "imza şeması geçersiz")
    require(signature["sema"] == "zee-surum-imza-v1" and signature["dosya"] == name, "imza dosya/alanı farklı")
    require(signature["sha256"] == digest(data) and signature["anahtar_kimligi"] == key, "imza içerik/yayıncı hash farklı")
    require(isinstance(key, str) and re.fullmatch(r"sha256:[0-9a-f]{64}", key), "explicit trusted anahtar kimliği gerekli")
    with tempfile.TemporaryDirectory(prefix="zee-signature-") as temp:
        root = Path(temp)
        executable = root / ("verifier.exe" if sys.platform == "win32" else "verifier")
        executable.write_bytes(verifier_data)
        executable.chmod(0o755)
        (root / "payload").write_bytes(data)
        (root / "signature.json").write_bytes(signature_data)
        command = [str(executable), "dogrula", "--dosya", str(root / "payload"), "--imza", str(root / "signature.json"), "--kimlik", key]
        checked = subprocess.run(command, capture_output=True, timeout=15, env=safe_environment())
        require(checked.returncode == 0, "Ed25519 imza doğrulanamadı")


@contextmanager
def authenticated_snapshot(pin, store, trust_file, trust_key, verifier, verifier_sha, pilot, project, bundle_source=None):
    validate_pin(pin, pilot)
    require(isinstance(verifier_sha, str) and HEX64.fullmatch(verifier_sha), "trusted verifier SHA-256 gerekli")
    verifier_data = regular_bytes(Path(verifier), LIMITS["executable"])
    require(digest(verifier_data) == verifier_sha, "trusted verifier hash farklı")
    trust_path = Path(trust_file)
    require(re.fullmatch(r"[A-Za-z0-9_.-]+", trust_path.name), "trusted katalog dosya adı geçersiz")
    trust_data = regular_bytes(trust_path, LIMITS["catalog"])
    verify_signed_bytes(trust_data, regular_bytes(Path(str(trust_path) + ".zee-imza"), LIMITS["signature"]), trust_path.name, trust_key, verifier_data)
    release = trusted_release(load_json(trust_data), pin)
    bundle = (root_directory(bundle_source) if bundle_source is not None else
              child_directory(child_directory(root_directory(store), pin["source_sha"]), pin["platform"]))
    expected_names = META | binary_names(pin)
    names = set()
    observed = 0
    with os.scandir(bundle) as entries:
        for item in entries:
            require(observed < len(expected_names) and item.name not in names, "bundle dosya kümesi exact olmalı")
            observed += 1
            names.add(item.name)
    require(names == expected_names, "bundle dosya kümesi exact olmalı")
    files = {name: regular_bytes(bundle / name, META_LIMITS.get(name, LIMITS["executable"])) for name in META | binary_names(pin)}
    for name, sha in release["file_hashes"].items():
        require(digest(files[name]) == sha, f"trusted sidecar hash farklı: {name}")
    sums = checksums(files["SHA256SUMS"], binary_names(pin))
    for name, sha in sums.items():
        require(digest(files[name]) == sha, f"binary SHA-256 farklı: {name}")
    source_date = validate_metadata(files, pin, sums, release)
    verify_signed_bytes(files["SHA256SUMS"], files["SHA256SUMS.zee-imza"], "SHA256SUMS", release["publisher"], verifier_data)
    with tempfile.TemporaryDirectory(prefix="zee-compiler-selection-") as temp:
        snapshot = Path(temp)
        for name, data in files.items():
            (snapshot / name).write_bytes(data)
            (snapshot / name).chmod(0o755 if name in sums else 0o600)
        compiler = snapshot / ("dil.exe" if pin["platform"].endswith("windows-msvc") else "dil")
        probed = subprocess.run([str(compiler), "sürüm", "--json"], cwd=project, capture_output=True, timeout=15, env=safe_environment())
        require(probed.returncode == 0 and len(probed.stdout) <= 8192, "authenticated compiler build JSON kimliği alınamadı; fallback yok")
        build = load_json(probed.stdout)
        require(isinstance(build, dict) and set(build) == {"schema", "version", "source_sha", "source_date", "channel", "dirty"}, "compiler build şeması farklı")
        require(build["schema"] == "zee-build-v1" and build["version"] == pin["version"] and build["channel"] == pin["channel"], "compiler sürüm/kanal pin ile farklı")
        require(build["source_sha"] == pin["source_sha"] or (pilot and build["source_sha"] == "unknown"), "compiler kaynak SHA pin ile farklı")
        require(build["source_date"] == source_date or (pilot and build["source_date"] == "unknown"), "compiler kaynak tarihi provenance ile farklı")
        require(type(build["dirty"]) is bool and (pilot or build["dirty"] is False), "kirli compiler explicit pilot ister")
        require(pilot or build["source_sha"] != "unknown", "unknown compiler explicit pilot ister")
        require(pin["channel"] != "stable" or
                (build["source_sha"] != "unknown" and build["source_date"] != "unknown" and not build["dirty"]),
                "kararlı kanal unknown/kirli compiler kabul edemez; pilot bunu gevşetmez")
        config = project / "zee.ayar"
        if config.exists() or config.is_symlink():
            regular_bytes(config, LIMITS["config"])
            queried = subprocess.run([str(compiler), "sürüm", "--ayar-semasi"], cwd=project,
                                     capture_output=True, timeout=15, env=safe_environment())
            require(queried.returncode == 0 and not queried.stderr and len(queried.stdout) <= 8192,
                    "zee.ayar için compiler capability sorgusu başarısız; eski sürüme fallback yok")
            capability = load_json(queried.stdout)
            require(isinstance(capability, dict) and set(capability) == {"schema", "config_schemas", "unknown_security", "version"},
                    "config capability şeması farklı")
            require(capability["schema"] == "zee-config-capability-1" and capability["config_schemas"] == ["zee-ayar-1"]
                    and capability["unknown_security"] == "reject" and capability["version"] == build["version"],
                    "compiler zee-ayar-1 fail-closed güvenlik yetkinliği taşımıyor")
        yield snapshot, build


def write_pin(project, pin, replace):
    target = project / PIN_FILE
    require(not target.is_symlink(), "pin symlink olamaz")
    require(replace or not target.exists(), "pin zaten var; bilinçli rollback/değişiklik için --degistir gerekli")
    temp = None
    try:
        with tempfile.NamedTemporaryFile(dir=project, prefix=".zee-pin-", delete=False) as stream:
            temp = Path(stream.name)
            stream.write((json.dumps(pin, ensure_ascii=False, indent=2) + "\n").encode())
            stream.flush()
            os.fsync(stream.fileno())
        if replace:
            os.replace(temp, target)
        else:
            os.link(temp, target)  # no-clobber publication
    finally:
        if temp is not None:
            temp.unlink(missing_ok=True)


def scoped_arguments(args, project):
    if not args:
        return ["çalıştır", str(project)]
    command, tail = args[0], args[1:]
    if command == "çalıştır":
        require(all(isinstance(arg, str) and arg.split("=", 1)[0] not in HOST_FLAGS and
                    not arg.split("=", 1)[0].startswith("--web-oran-") for arg in tail),
                "çalıştır program argümanında global host/güvenlik bayrağı reddedildi; zee.ayar kullan")
        return [command, str(project)] + tail
    allowed = {"denetle": {"--json"}, "dene": set(), "biçimle": {"--denetle"}, "göçür": {"--durum", "--kuru"}}
    require(command in allowed and len(tail) == len(set(tail)) and set(tail) <= allowed[command],
            "pilot yalnız proje köküne bağlı denetle/dene/çalıştır/biçimle/göçür çağrısı sağlar; başka proje yolu kabul edilmez")
    require(command != "göçür" or len(tail) <= 1, "göç durum/kuru birlikte seçilemez")
    return [command] + tail + [str(project)]


def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    split = argv.index("--") if "--" in argv else len(argv)
    options, compiler_args = argv[:split], argv[split + 1:]
    keys = [arg.split("=", 1)[0] for arg in options if arg.startswith("--")]
    require(len(keys) == len(set(keys)), "yinelenen seçici seçeneği")
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("komut", choices=("seç", "çalıştır", "lsp", "pinle"))
    for name in ("proje", "depo", "guven", "guven-anahtari", "dogrulayici", "dogrulayici-sha256"):
        p.add_argument("--" + name, required=True)
    p.add_argument("--pilot", action="store_true")
    p.add_argument("--degistir", action="store_true")
    for name in ("surum", "sha", "kanal", "platform"):
        p.add_argument("--" + name)
    a = p.parse_args(options)
    require(not compiler_args or a.komut in ("çalıştır", "lsp"), "compiler argümanları yalnız çalıştır/lsp komutunda")
    project = root_directory(a.proje)
    if a.komut == "pinle":
        pin = {"schema": "zee-derleyici-pin-1", "version": a.surum, "source_sha": a.sha,
               "channel": a.kanal, "platform": a.platform or host_platform()}
    else:
        require(not any((a.surum, a.sha, a.kanal, a.platform, a.degistir)), "pin override yok; mevcut exact proje pini kullanılır")
        path = project / PIN_FILE
        require(path.exists(), f"{PIN_FILE} yok; önce doğrulanmış exact sürüm pinle; global/latest fallback yok")
        pin = load_json(regular_bytes(path, LIMITS["pin"]))
    with authenticated_snapshot(pin, a.depo, a.guven, a.guven_anahtari, a.dogrulayici, a.dogrulayici_sha256, a.pilot, project) as (snapshot, build):
        if a.komut == "pinle":
            write_pin(project, pin, a.degistir)
        if a.komut in ("pinle", "seç"):
            print(json.dumps({"pin": pin, "authenticated_build": build}, ensure_ascii=False))
            return 0
        name = "dil" if a.komut == "çalıştır" else "dillsp"
        if pin["platform"].endswith("windows-msvc"):
            name += ".exe"
        require(a.komut != "lsp" or not compiler_args, "pilot LSP ek executable argümanı kabul etmez")
        args = scoped_arguments(compiler_args, project) if a.komut == "çalıştır" else []
        return subprocess.run([str(snapshot / name)] + args, cwd=project, env=safe_environment()).returncode


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (Reject, OSError, subprocess.TimeoutExpired) as error:
        print(f"Derleyici seçimi reddedildi: {error}", file=sys.stderr)
        raise SystemExit(2)
    except (RecursionError, ValueError, KeyError, TypeError, AttributeError):
        print("Derleyici seçimi reddedildi: girdi tipi veya JSON geçersiz", file=sys.stderr)
        raise SystemExit(2)
