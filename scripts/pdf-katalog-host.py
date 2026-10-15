#!/usr/bin/env python3
"""K275 owned fixed PDF prototype: trusted assets only, never an arbitrary PDF service."""
import argparse
import hashlib
import io
import importlib.metadata
import json
import os
from pathlib import Path
import stat
import sys

EXPECTED_LIBRARIES = {"reportlab": "4.4.9", "pypdf": "6.10.0", "Pillow": "12.3.0"}

MAX_JOB = 8192
MAX_PDF = 1024 * 1024
ASSETS = {'base.pdf': '7c25bd391859088307fa0b1816eca1d2c80634e6a950b9018152bf28ffdaf05f',
          'font.ttf': '7da195a74c55bef988d0d48f9508bd5d849425c1770dba5d7bfc6ce9ed848954',
          'product.png': '9a09e7df94e28fd35c0d255419f254bc1f4b1a51e323a490d758ebd8b711c9d5'}

class Rejected(Exception):
    pass

def need(condition):
    if not condition:
        raise Rejected()

class Parser(argparse.ArgumentParser):
    def error(self, message):
        raise Rejected()

class BoundedBuffer(io.BytesIO):
    def write(self, data):
        need(self.tell() + len(data) <= MAX_PDF)
        return super().write(data)

def bounded_file(path, limit):
    before = path.lstat()
    need(stat.S_ISREG(before.st_mode) and before.st_size <= limit)
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC)
    key = lambda s: (s.st_dev, s.st_ino, s.st_mode, s.st_size, s.st_mtime_ns)
    try:
        initial = os.fstat(fd)
        need(key(initial) == key(before))
        chunks, count = [], 0
        while True:
            chunk = os.read(fd, min(65536, limit - count + 1))
            if not chunk:
                break
            count += len(chunk)
            need(count <= limit)
            chunks.append(chunk)
        need(count == before.st_size and key(initial) == key(os.fstat(fd)) == key(path.lstat()))
        return b''.join(chunks)
    finally:
        os.close(fd)

def pairs(values):
    out = {}
    for key, value in values:
        need(key not in out)
        out[key] = value
    return out

def validate(data):
    need(len(data) <= MAX_JOB)
    job = json.loads(data.decode('utf-8'), object_pairs_hook=pairs)
    need(type(job) is dict and set(job) == {'schema', 'locale', 'title', 'product', 'description', 'selection', 'placement'})
    need(all(type(value) is str for value in job.values()))
    need(job['schema'] == 'zee-pdf-job-1' and job['locale'] in ('tr', 'en'))
    need(job['selection'] in ('all', 'cover-back') and job['placement'] == 'after-cover')
    for key, limit in [('title', 80), ('product', 100), ('description', 200)]:
        text = job[key]
        need(0 < len(text) <= limit and all(32 <= ord(c) <= 0xFFFF and not 0xD800 <= ord(c) <= 0xDFFF for c in text))
    return job

def backend_contract():
    actual = {name: importlib.metadata.version(name) for name in EXPECTED_LIBRARIES}
    need(actual == EXPECTED_LIBRARIES)
    return actual

def produce(job, root):
    backend_contract()
    # All parsers receive immutable, bounded bytes of fixed owned resources.
    resources = {name: bounded_file(root / 'assets' / name, MAX_PDF) for name in ASSETS}
    need(all(hashlib.sha256(data).hexdigest() == ASSETS[name] for name, data in resources.items()))
    from pypdf import PdfReader, PdfWriter
    from reportlab.pdfbase import pdfmetrics
    from reportlab.pdfbase.ttfonts import TTFont
    from reportlab.pdfgen import canvas
    from reportlab.lib.utils import ImageReader
    from reportlab.lib.pagesizes import A4
    font = TTFont('OwnedPdfFont', io.BytesIO(resources['font.ttf']))
    allowed = font.face.charToGlyph
    need(all(ord(c) in allowed for key in ('title', 'product', 'description') for c in job[key]))
    pdfmetrics.registerFont(font)
    page = BoundedBuffer()
    c = canvas.Canvas(page, pagesize=A4, invariant=1, pageCompression=1)
    c.setTitle(job['title'])
    c.setAuthor('Anonymous owned fixture')
    c.setFillColorRGB(.07, .18, .24)
    c.setFont('OwnedPdfFont', 22)
    need(pdfmetrics.stringWidth(job['title'], 'OwnedPdfFont', 22) <= 480)
    c.drawString(54, 765, job['title'])
    c.setFont('OwnedPdfFont', 14)
    need(pdfmetrics.stringWidth(job['product'], 'OwnedPdfFont', 14) <= 480)
    c.drawString(54, 725, job['product'])
    c.drawImage(ImageReader(io.BytesIO(resources['product.png'])), 54, 430, width=480, height=240)
    c.setFont('OwnedPdfFont', 11)
    # Canvas text only; no Paragraph/HTML, URI or string-selected image/font.
    words, line, lines = job['description'].split(' '), '', []
    for word in words:
        next_line = (line + ' ' + word).strip()
        if pdfmetrics.stringWidth(next_line, 'OwnedPdfFont', 11) > 480:
            need(bool(line) and pdfmetrics.stringWidth(word, 'OwnedPdfFont', 11) <= 480)
            lines.append(line)
            line = word
        else:
            line = next_line
    lines.append(line)
    need(len(lines) <= 5)
    for n, text in enumerate(lines):
        c.drawString(54, 395 - 18 * n, text)
    c.setFont('OwnedPdfFont', 9)
    c.drawString(54, 48, 'B3 - owned technical fixture / teknik sınama')
    c.showPage()
    c.save()
    base = PdfReader(io.BytesIO(resources['base.pdf']), strict=True)
    need(len(base.pages) == 3 and not base.is_encrypted)
    selected = [0, 1, 2] if job['selection'] == 'all' else [0, 2]
    extracted = PdfWriter()
    for i in selected:
        extracted.add_page(base.pages[i])
    generated = PdfReader(io.BytesIO(page.getvalue()), strict=True)
    combined = PdfWriter()
    combined.add_page(base.pages[selected[0]])
    combined.add_page(generated.pages[0])
    for i in selected[1:]:
        combined.add_page(base.pages[i])
    results = {'generated.pdf': page.getvalue()}
    for name, writer in [('selected.pdf', extracted), ('catalog.pdf', combined)]:
        out = BoundedBuffer()
        writer.write(out)
        results[name] = out.getvalue()
    return results

def main():
    parser = Parser(allow_abbrev=False)
    for name in ('fixture-root', 'job', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    need(os.name == 'posix')
    job = validate(bounded_file(args.job, MAX_JOB))
    results = produce(job, args.fixture_root.resolve(strict=True))
    # Publication is a fresh owned directory; no existing target is overwritten.
    args.output.mkdir(mode=0o700)
    for name, data in results.items():
        with (args.output / name).open('xb') as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
    print(json.dumps({'schema': 'zee-owned-pdf-receipt-1', 'status': 'produced', 'files': [
        {'name': name, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()} for name, data in results.items()]}))

if __name__ == '__main__':
    try:
        main()
    except (Exception, KeyboardInterrupt):
        print('{"type":"Rejected","profile":"owned-pdf-prototype"}', file=sys.stderr)
        sys.exit(2)
