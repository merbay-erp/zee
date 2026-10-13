#!/usr/bin/env python3
"""K-196: Rust 1.93.1 gerçek linker izinden iki release ikilisinin ortamı.

Rustc RUSTC_LOG sözleşmesi: https://rustc-dev-guide.rust-lang.org/tracing.html
Log bir shell komutu olarak çalıştırılmaz; quoted Debug String alanları okunur.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PureWindowsPath
import re
import shutil
import subprocess
import sys


def quoted(line):
    return [json.loads(m.group()) for m in re.finditer(r'"(?:[^"\\]|\\.)*"', line)]


def arac_surumu(executable):
    if executable.name.lower() in ("link.exe", "cl.exe", "lib.exe", "ml.exe", "ml64.exe"):
        # MSVC help çağrısının exit/DLL ortamını sürüm kanıtıyla karıştırma.
        # Yol komut metnine eklenmez; .NET dosya metadata'sı doğrudan okunur.
        query = ("$ErrorActionPreference='Stop'; "
                 "$v=[System.Diagnostics.FileVersionInfo]::GetVersionInfo($env:ZEE_LINKER_ID_PATH); "
                 "[Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false); "
                 "if (!$v.FileVersion) { throw 'Missing linker file version' }; "
                 "Write-Output ('FileVersion: '+$v.FileVersion); "
                 "Write-Output ('ProductVersion: '+$v.ProductVersion)")
        command = ["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", query]
        environment = dict(os.environ, ZEE_LINKER_ID_PATH=str(executable))
        source = "Windows FileVersionInfo"
    else:
        command = [str(executable), "--version"]
        environment = None
        source = "--version"
    result = subprocess.run(command, capture_output=True, timeout=20, env=environment)
    value = (result.stdout + result.stderr).decode("utf-8", errors="replace")[:4000].strip()
    if result.returncode != 0 or not value:
        raise ValueError(f"Linker sürümü okunamadı: {executable}: exit={result.returncode}: {value}")
    return value, source


DEBUG_STRING = r'"(?:[^"\\]|\\.)*"'
NATIVE_COMMAND = re.compile(
    r'^(?:env )?(?:(?:-u [A-Za-z_][A-Za-z_0-9]* |'
    r'[A-Za-z_][A-Za-z_0-9]*=' + DEBUG_STRING + r' )*)'
    r'(?P<program>' + DEBUG_STRING + r')(?P<args>(?: ' + DEBUG_STRING + r')*)$')


def native_kaydet(text):
    """Locked cc 1.4.4 Debug Command sözleşmesi; shell/eval kullanılmaz."""
    tools, invocations = {}, []
    for line in text.splitlines():
        owner = re.match(r'^\[([^] ]+ [^] ]+)\] running: (.*)$', line)
        if not owner:
            continue
        match = NATIVE_COMMAND.fullmatch(owner[2])
        if not match:
            raise ValueError('Native komut biçimi çözülemedi: ' + owner[1])
        program = json.loads(match['program'])
        args = quoted(match['args'])
        environment_path = re.search(r'\bPATH=(' + DEBUG_STRING + ')', owner[2][:match.start('program')])
        resolved = shutil.which(program, path=json.loads(environment_path[1]) if environment_path else None)
        if not resolved:
            raise ValueError('Native araç bulunamadı: ' + program)
        executable = Path(resolved).resolve()
        identity = str(executable)
        if identity not in tools:
            try:
                version, source = arac_surumu(executable)
                query = {'status': 'available', 'value': version, 'source': source}
            except ValueError as error:
                # Apple ar --version desteklemez. Hash kimliği sürüm adı değildir.
                query = {'status': 'unavailable', 'reason': str(error)}
            tools[identity] = {'path': identity,
                               'sha256': hashlib.sha256(executable.read_bytes()).hexdigest(),
                               'versionQuery': query}
        name = PureWindowsPath(program).name.lower()
        if name in ('ar', 'llvm-ar', 'gcc-ar', 'lib.exe'):
            role = 'archive'
        elif name == 'xcrun':
            role = 'sdk-query'
        elif any(arg.lower() in ('-c', '/c') for arg in args):
            role = 'compile'
        else:
            role = 'probe'
        invocations.append({'owner': owner[1], 'program': program, 'tool': identity,
                            'arguments': args, 'role': role, 'command': owner[2]})
    roles = {entry['role'] for entry in invocations}
    if not {'compile', 'archive'} <= roles:
        raise ValueError('Native compile/archive komut kanıtı eksik')
    return {'schema': 'zee-native-tools-1', 'tools': list(tools.values()),
            'invocations': invocations,
            'boundary': 'Locked cc Debug Command driver çağrıları; sürüm sorgusu başarısızlığı açık kalır. Driver alt süreçleri, pkg-config/vcpkg ve işletim sistemi process trace kapsamı değildir.'}


def kaydet(logs, sha, native_required=False):
    if not re.fullmatch(r"[0-9a-f]{40}", sha):
        raise ValueError("Kaynak SHA exact 40 hex olmalı")
    clones = {}
    for clone, log in logs.items():
        text = log.read_text(encoding="utf-8", errors="strict")
        records = {}
        for line in text.splitlines():
            if "rustc_codegen_ssa::back::link" not in line or "preparing " in line:
                continue
            fields = quoted(line)
            binary = None
            for value in fields:
                # MSVC çıktı anahtarı Cargo'nun hash'li veya düz exe adını taşır.
                value = re.sub(r"^/OUT:", "", value, flags=re.IGNORECASE)
                name = PureWindowsPath(value).name if "\\" in value else Path(value).name
                for candidate in ("dil", "dillsp"):
                    if re.fullmatch(candidate + r"(?:-[0-9a-f]+)?(?:\.exe)?", name):
                        binary = candidate
            if binary is None:
                continue
            programs = [v for v in fields if PureWindowsPath(v).name.lower() in
                        ("cc", "gcc", "clang", "link.exe", "lld-link.exe", "rust-lld", "ld")]
            if len(programs) != 1:
                raise ValueError(f"Gerçek linker programı tekil değil: {binary}")
            program = programs[0]
            path_match = re.search(r'PATH=("(?:[^"\\]|\\.)*")', line)
            search_path = json.loads(path_match.group(1)) if path_match else None
            resolved = shutil.which(program, path=search_path)
            if not resolved:
                raise ValueError(f"İzdeki linker bulunamadı: {program}")
            executable = Path(resolved).resolve()
            version, version_source = arac_surumu(executable)
            sdk_paths = sorted(set(v for v in fields if "Windows Kits" in v or
                                   ".sdk" in v or v.startswith("/LIBPATH:")))
            records[binary] = {"program": program, "path": str(executable),
                               "sha256": hashlib.sha256(executable.read_bytes()).hexdigest(),
                               "version": version, "version_source": version_source,
                               "command": line.strip(), "sdk_references": sdk_paths}
        if set(records) != {"dil", "dillsp"}:
            raise ValueError(f"İki gerçek release link komutu yok: {clone}")
        sdk_observations = sorted(set(
            [v for v in quoted(text) if ".sdk" in v or "Windows Kits" in v] +
            re.findall(r'(?im)[a-z]:[\\/][^\r\n"]*?Windows Kits[\\/]10[\\/]Lib[\\/][0-9.]+[\\/][^\r\n"]*', text)))
        sdk_versions = {}
        for reference in sdk_observations:
            windows_sdk = re.search(r"Windows Kits[\\/]10[\\/]Lib[\\/]([0-9.]+)[\\/]", reference, re.IGNORECASE)
            if windows_sdk:
                sdk_versions[reference] = windows_sdk.group(1)
            if reference.endswith(".sdk"):
                settings = Path(reference) / "SDKSettings.json"
                if settings.is_file():
                    sdk_versions[reference] = json.loads(settings.read_text())["Version"]
        clones[clone] = {"binaries": records, "sdkObservations": sdk_observations,
                         "sdkVersions": sdk_versions}
        if native_required:
            clones[clone]["nativeTools"] = native_kaydet(text)
    if set(clones) != {"a", "b"}:
        raise ValueError("İki klon kanıtı gerekir")
    return {"schema": "zee-build-environment-1", "gitCommit": sha, "clones": clones,
            "boundary": "Gerçek rustc linker programı ve komut SDK referansları; minimum OS veya tüm native C build-script araçlarının kanıtı değildir."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--a", type=Path, required=True)
    parser.add_argument("--b", type=Path, required=True)
    parser.add_argument("--sha", required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--native-target", default="none")
    args = parser.parse_args()
    try:
        data = kaydet({"a": args.a, "b": args.b}, args.sha, native_required=True)
        if args.native_target != "none":
            for clone, log in {"a":args.a,"b":args.b}.items():
                native=json.loads(log.with_name('native-deployment.json').read_bytes())
                if native.get('schema')!='zee-native-deployment-1' or native.get('target')!=args.native_target:
                    raise ValueError('Native deployment source/target mismatch')
                data['clones'][clone]['nativeDeployment']=native
            data['nativeDeploymentTarget']='MACOSX_DEPLOYMENT_TARGET='+args.native_target
        args.out.write_bytes((json.dumps(data, ensure_ascii=False, indent=2) + "\n").encode())
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print("build-environment: failed: " + str(error).encode("ascii", errors="backslashreplace").decode("ascii"), file=sys.stderr)
        return 1
    print("build-environment: passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
