#!/usr/bin/env python3
"""İncelenen sürüm notuyla site kaynağının eşleşmesini dağıtımdan önce denetle."""
import argparse
import hashlib
import json
from pathlib import Path


def ozet(site):
    files = [site / 'proje.dil']
    for folder in ('kaynak', 'static', 'scripts', 'deploy'):
        files.extend(p for p in (site / folder).rglob('*')
                     if p.is_file() and '__pycache__' not in p.parts)
    rows = [p.relative_to(site).as_posix() + '\t' +
            hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(files)]
    return hashlib.sha256(('\n'.join(rows) + '\n').encode()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--denetle', action='store_true')
    mode.add_argument('--yaz', action='store_true')
    parser.add_argument('--kayit', help='docs/surumler.md içindeki exact ## başlığı')
    args = parser.parse_args()
    site = Path(__file__).resolve().parents[1]
    notes = site.parents[1] / 'docs/surumler.md'
    record = site / 'surum-kaydi.json'
    source = ozet(site)
    text = notes.read_text(encoding='utf-8')
    note_hash = hashlib.sha256(notes.read_bytes()).hexdigest()
    if args.yaz:
        if not args.kayit or '\n## ' + args.kayit + '\n' not in text:
            parser.error('--kayit mevcut bir tarihli sürüm notu başlığı olmalı')
        if record.exists():
            old = json.loads(record.read_text())
            if old['kaynak_sha256'] != source and old['surum_notlari_sha256'] == note_hash:
                parser.error('Kaynak değişti; önce docs/surumler.md kaydına değişikliği yazın.')
        data = {'schema': 'zee-site-release-note-1', 'kayit': args.kayit,
                'sayfa': '/belge/surumler/', 'kaynak_sha256': source,
                'surum_notlari_sha256': note_hash}
        record.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
    else:
        if not record.exists():
            parser.error('Sürüm kaydı yok; README yayın sırasını izleyin.')
        data = json.loads(record.read_text())
        if (data.get('schema') != 'zee-site-release-note-1' or
                data.get('kaynak_sha256') != source or
                data.get('surum_notlari_sha256') != note_hash or
                '\n## ' + data.get('kayit', '') + '\n' not in text):
            parser.error('Sürüm notları/kaynak eşleşmesi bayat; README yayın sırasını izleyin.')
    print('Sürüm notları ve site kaynağı eşleşiyor: ' + data['kayit'])


if __name__ == '__main__':
    main()
