#!/usr/bin/env python3
"""K-245: POSIX credential dosyalarını doğrula, mevcut Zee CLI'ını başlat.

Compiler API'si değildir. Çocuk stdio'su kapalıdır; yalnız sabit şemalı
source/exit metadata üretilir. Yollar/sırlar/hata ayrıntıları yazılmaz.
"""
import argparse
import json
import os
import re
import stat
import subprocess
import sys

MAX_BINDINGS = 16
MAX_FILE_BYTES = 65_536
MAX_TOTAL_BYTES = 65_536
MAX_PATH_BYTES = 4096
MAX_PATH_PARTS = 64
ENV_NAME = re.compile(r"[A-Z_][A-Z0-9_]{0,63}\Z")
FILE_NAME = re.compile(r"[A-Za-z0-9_][A-Za-z0-9_.-]{0,63}\Z")
RESERVED = frozenset({
    "PATH", "HOME", "SHELL", "USER", "LOGNAME", "PWD", "OLDPWD", "TMP",
    "TEMP", "TMPDIR", "LANG", "LANGUAGE", "TZ", "IFS", "ENV", "BASH_ENV",
    "SHELLOPTS", "BASHOPTS", "CDPATH", "GLOBIGNORE", "PSMODULEPATH",
    "SYSTEMROOT", "WINDIR", "COMSPEC", "PATHEXT", "CREDENTIALS_DIRECTORY",
    "CODEX_HOME",
})
RESERVED_PREFIXES = ("LD_", "DYLD_", "PYTHON", "RUST", "CARGO", "LC_", "XDG_", "DOTNET_", "POWERSHELL_")


class Rejected(ValueError):
    """İç ret ayrıntısı kullanıcıya veya günlüğe taşınmaz."""


def require(condition):
    if not condition:
        raise Rejected()


class MaskedParser(argparse.ArgumentParser):
    def error(self, message):
        raise Rejected()

    def exit(self, status=0, message=None):
        raise Rejected()


def supported_platform():
    return os.name == "posix" and all(hasattr(os, name) for name in
        ("O_DIRECTORY", "O_NOFOLLOW", "O_NONBLOCK", "O_CLOEXEC"))


def absolute_parts(path):
    require(isinstance(path, str) and path.startswith("/"))
    require(0 < len(path.encode("utf-8")) <= MAX_PATH_BYTES)
    require(not any(char in path for char in "\x00\t\r\n"))
    parts = path.split("/")[1:]
    require(0 < len(parts) <= MAX_PATH_PARTS)
    require(all(part and part not in (".", "..") for part in parts))
    return parts


def identity(info):
    return (info.st_dev, info.st_ino, info.st_mode, info.st_uid,
            info.st_nlink, info.st_size, info.st_mtime_ns, info.st_ctime_ns)


def private_directory(info):
    require(stat.S_ISDIR(info.st_mode))
    require(info.st_uid in (0, os.geteuid()))
    require(stat.S_IMODE(info.st_mode) in (0o500, 0o700))


def open_root(path):
    """Her bileşeni önceki açık dirfd'ye göre nofollow aç; son kökü tut."""
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC
    fd = os.open("/", flags)
    try:
        for part in absolute_parts(path):
            child = os.open(part, flags, dir_fd=fd)
            os.close(fd)
            fd = child
        info = os.fstat(fd)
        private_directory(info)
        result, fd = fd, None
        return result, identity(info)
    finally:
        if fd is not None:
            os.close(fd)


def private_file(info):
    require(stat.S_ISREG(info.st_mode))
    require(info.st_uid in (0, os.geteuid()))
    require(stat.S_IMODE(info.st_mode) in (0o400, 0o600))
    require(info.st_nlink == 1)
    require(0 < info.st_size <= MAX_FILE_BYTES)


def read_credential(root_fd, name):
    before = os.stat(name, dir_fd=root_fd, follow_symlinks=False)
    private_file(before)
    flags = os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC
    fd = os.open(name, flags, dir_fd=root_fd)
    try:
        opened = os.fstat(fd)
        private_file(opened)
        require(identity(before) == identity(opened))
        chunks = []
        size = 0
        while size <= MAX_FILE_BYTES:
            chunk = os.read(fd, min(8192, MAX_FILE_BYTES + 1 - size))
            if not chunk:
                break
            chunks.append(chunk)
            size += len(chunk)
        after = os.fstat(fd)
        require(size == opened.st_size and size <= MAX_FILE_BYTES)
        require(identity(opened) == identity(after))
        current = os.stat(name, dir_fd=root_fd, follow_symlinks=False)
        private_file(current)
        require(identity(after) == identity(current))
        data = b"".join(chunks)
        value = data.decode("utf-8")
        # Ürün URL/key/bearer dilimi: exact tek satır, trim veya shell yorumlama yok.
        require(value and not value.startswith("\ufeff"))
        require(not any(char in value for char in "\x00\r\n"))
        return value, size, identity(current)
    finally:
        os.close(fd)


def parse_arguments(argv):
    require("--" in argv)
    split = argv.index("--")
    command = argv[split + 1:]
    require(len(command) >= 2 and command[0] in ("çalıştır", "calistir"))
    parser = MaskedParser(add_help=False, allow_abbrev=False)
    parser.add_argument("--credential-root", required=True)
    parser.add_argument("--env", action="append", required=True)
    parser.add_argument("--dil", required=True)
    options = argv[:split]
    require(options.count("--credential-root") == 1 and options.count("--dil") == 1)
    parsed = parser.parse_args(options)
    require(0 < len(parsed.env) <= MAX_BINDINGS)
    absolute_parts(parsed.credential_root)
    absolute_parts(parsed.dil)
    executable = os.lstat(parsed.dil)
    require(stat.S_ISREG(executable.st_mode) and os.access(parsed.dil, os.X_OK))
    bindings = []
    names, files = set(), set()
    for text in parsed.env:
        require(text.count("=") == 1)
        name, file = text.split("=")
        require(ENV_NAME.fullmatch(name) and FILE_NAME.fullmatch(file))
        require(name not in RESERVED and not name.startswith(RESERVED_PREFIXES))
        require(name not in names and file not in files)
        # Env/file önceliği uydurulmaz: mapped ortam değeri varsa ret, fallback yok.
        require(name not in os.environ)
        names.add(name)
        files.add(file)
        bindings.append((name, file))
    return parsed.credential_root, bindings, [parsed.dil] + command


def child_environment(root, bindings):
    root_fd, root_identity = open_root(root)
    try:
        values = {}
        records = []
        total = 0
        for name, file in bindings:
            value, size, file_identity = read_credential(root_fd, file)
            total += size
            require(total <= MAX_TOTAL_BYTES)
            values[name] = value
            records.append((file, file_identity))
        # Daha önce okunan credential başka eşleme okunurken değişmiş olamaz.
        for file, expected in records:
            current = os.stat(file, dir_fd=root_fd, follow_symlinks=False)
            private_file(current)
            require(identity(current) == expected)
        require(identity(os.fstat(root_fd)) == root_identity)
        fresh, fresh_identity = open_root(root)
        try:
            require(fresh_identity == root_identity)
        finally:
            os.close(fresh)
        environment = {key: value for key, value in os.environ.items()
                       if not key.upper().startswith(("LD_", "DYLD_", "PYTHON"))}
        environment.update(values)
        return environment
    finally:
        os.close(root_fd)


def run_child(command, environment):
    # Aynı process group; restart/grup sonlandırma dış host supervisor'a aittir.
    child = None
    try:
        child = subprocess.Popen(command, env=environment, stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, close_fds=True)
        environment.clear()
        code = child.wait()
        return code if code >= 0 else min(255, 128 - code)
    except BaseException:
        if child is not None and child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()
        raise
    finally:
        environment.clear()


def main(argv=None):
    source, code = "launcher", 2
    try:
        require(supported_platform())
        root, bindings, command = parse_arguments(list(sys.argv[1:] if argv is None else argv))
        environment = child_environment(root, bindings)
        code = run_child(command, environment)
        source = "child"
    except KeyboardInterrupt:
        code = 130
    except Exception:
        pass
    # Hatalar, child argv, credential ad/yol/değerleri ve stream'ler yazılmaz.
    print(json.dumps({"schema": "zee-credential-baslat-1", "source": source, "exit": code}, separators=(",", ":")))
    return code


if __name__ == "__main__":
    raise SystemExit(main())
