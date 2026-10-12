#!/usr/bin/env python3
"""K-200: macOS build-script nesnelerinin deployment başlığını denetle.

Başlık uyumu gerçek minimum OS desteği değildir; başlıksız assembly açık kalır.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys


def surum(value):
    if not re.fullmatch(r"[0-9]+\.[0-9]+(?:\.[0-9]+)?", value):
        raise ValueError("Deployment version invalid")
    parts = list(map(int, value.split('.')))
    return tuple(parts + [0] * (3 - len(parts)))


def nesne_basligi(text):
    # LC_BUILD_VERSION minos veya eski LC_VERSION_MIN_MACOSX version;
    # object dosyasında LC_SOURCE_VERSION/tool version bu kanıt değildir.
    blocks = re.split(r"(?m)^Load command [0-9]+\s*$", text)
    result = []
    for block in blocks:
        if not re.search(r"\bcmd LC_(?:BUILD_VERSION|VERSION_MIN_MACOSX)\b", block):
            continue
        platform = re.search(r"(?m)^\s*platform\s+(\S+)\s*$", block)
        if platform and platform.group(1) not in ('1', 'MACOS'):
            raise ValueError('Native object platform is not macOS')
        result.extend(re.findall(r"(?m)^\s*(?:minos|version)\s+([0-9]+\.[0-9]+(?:\.[0-9]+)?)\s*$", block))
    return result


def denetle(root, target):
    expected = surum(target)
    objects = []
    for obj in sorted(root.glob('**/build/*/out/**/*.o')):
        result = subprocess.run(['otool', '-l', str(obj)], capture_output=True, timeout=20)
        if result.returncode:
            raise ValueError(f"Native object header unreadable: {obj.name}")
        versions = nesne_basligi(result.stdout.decode('utf-8', errors='strict'))
        if any(surum(v) > expected for v in versions):
            raise ValueError(f"Native object deployment exceeds Rust target: {obj.name}: {versions} > {target}")
        objects.append({'path':str(obj.relative_to(root)), 'sha256':hashlib.sha256(obj.read_bytes()).hexdigest(),
                        'deployment_versions':versions, 'header':result.stdout.decode('utf-8')})
    if not any(obj['deployment_versions'] for obj in objects):
        raise ValueError('Native deployment version evidence missing')
    return {'schema':'zee-native-deployment-1','target':target,'objects':objects,
            'boundary':'Yalnız build-script .o başlıkları; başlıksız assembly ve gerçek minimum OS/runtime desteği değildir.'}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,required=True)
    parser.add_argument('--target',required=True)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args()
    try:
        report=denetle(args.root,args.target)
        args.out.write_bytes((json.dumps(report,ensure_ascii=False,indent=2)+'\n').encode('utf-8'))
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print('native-deployment: failed: '+str(error).encode('ascii',errors='backslashreplace').decode('ascii'),file=sys.stderr)
        return 1
    print('native-deployment: passed')
    return 0


if __name__=='__main__':
    sys.exit(main())
