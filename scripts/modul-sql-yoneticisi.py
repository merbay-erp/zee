#!/usr/bin/env python3
"""Owned SQL module lifecycle. Local trusted operator; no installer or DB/FS atomicity."""
import argparse
import base64
import contextlib
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import sys
import tempfile

BASE = None
try:
    helper = Path(__file__).resolve().with_name('modul-yoneticisi.py')
    fd = os.open(helper, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        info = os.fstat(fd)
        import stat
        if not stat.S_ISREG(info.st_mode) or info.st_size > 32768:
            raise ValueError()
        raw = os.read(fd, 32769)
        if hashlib.sha256(raw).hexdigest() != '84752ef56d5c1d347f2e211ebe721def648aa5e496df1bd8bfec2375933237b6':
            raise ValueError()
        after = os.fstat(fd)
        path = helper.stat(follow_symlinks=False)
        key = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns)
        if key(info) != key(after) or key(after) != key(path):
            raise ValueError()
        BASE = {'__name__': 'owned_sql_module_base', '__file__': str(helper)}
        exec(compile(raw, str(helper), 'exec'), BASE)
        if BASE['H'] is None:
            raise ValueError()
    finally:
        os.close(fd)
except (OSError, ValueError, KeyError, TypeError):
    BASE = None

class Rejected(ValueError):
    pass

class Visible(Rejected):
    pass

def need(value):
    if not value:
        raise Rejected('rejected')

def encode(value):
    return (json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':')) + '\n').encode()

def digest(raw):
    return hashlib.sha256(raw).hexdigest()

def ident(value):
    need(type(value) is str and re.fullmatch('[a-z][a-z0-9_]{0,23}', value) and value not in {'id', 'state', 'schema'})
    return value

def definition(value):
    need(type(value) is dict and set(value) == {'schema', 'id', 'label', 'fields'})
    need(value['schema'] == 'zee-sql-module-1')
    ident(value['id'])
    need(type(value['label']) is str and 1 <= len(value['label']) <= 80 and all(ord(x) >= 32 for x in value['label']))
    fields = value['fields']
    need(type(fields) is list and 1 <= len(fields) <= 8)
    names = set()
    for field in fields:
        need(type(field) is dict and set(field) == {'name', 'type'})
        name = ident(field['name'])
        need(name not in names and field['type'] in {'text', 'integer', 'boolean'})
        names.add(name)
    return value

def profile(value):
    need(type(value) is dict and set(value) == {'schema', 'target', 'environment'})
    need(value['schema'] == 'zee-sql-module-profile-1')
    need(type(value['target']) is str and re.fullmatch(r'postgresql://(?:localhost|127\.0\.0\.1):[0-9]{1,5}/[a-z][a-z0-9_]{0,31}', value['target']))
    port = int(value['target'].split(':')[-1].split('/')[0])
    need(1024 <= port <= 65535 and value['environment'] == 'ZEE_MODULE_DATABASE_URL')
    return value

def values(spec, value, operation):
    definition(spec)
    need(operation in {'create', 'read', 'update', 'delete', 'list'})
    need(type(value) is dict)
    names = {f['name'] for f in spec['fields']}
    expected = set() if operation == 'list' else ({'id'} | names if operation in {'create', 'update'} else {'id'})
    need(set(value) == expected)
    result = []
    if operation != 'list':
        need(type(value['id']) is str and 1 <= len(value['id']) <= 128 and '\x00' not in value['id'])
        result.append(value['id'])
    if operation in {'create', 'update'}:
        for field in spec['fields']:
            v = value[field['name']]
            if field['type'] == 'text':
                need(type(v) is str and len(v.encode()) <= 4096 and '\x00' not in v)
                result.append(v)
            elif field['type'] == 'integer':
                need(type(v) is int and -(2**63) <= v < 2**63)
                result.append(str(v))
            else:
                need(type(v) is bool)
                result.append('true' if v else 'false')
    return result

def table(spec):
    return 'public.zee_mod_' + definition(spec)['id']

def select_sql(spec, operation):
    columns = ['id::text AS id'] + ['"' + f['name'] + '"::text AS "' + f['name'] + '"' for f in spec['fields']]
    return 'SELECT ' + ','.join(columns) + ' FROM ' + table(spec) + (' ORDER BY id LIMIT 100' if operation == 'list' else ' WHERE id=$1')

def action_sql(spec, operation):
    fields = spec['fields']
    casts = {'text': 'text', 'integer': 'bigint', 'boolean': 'boolean'}
    parameters = ['$' + str(n + 2) + '::' + casts[f['type']] for n, f in enumerate(fields)]
    if operation == 'create':
        return 'INSERT INTO ' + table(spec) + '(id,' + ','.join('"' + f['name'] + '"' for f in fields) + ') VALUES ($1,' + ','.join(parameters) + ')'
    if operation == 'update':
        return 'UPDATE ' + table(spec) + ' SET ' + ','.join('"' + f['name'] + '"=' + p for f, p in zip(fields, parameters)) + ' WHERE id=$1'
    need(operation == 'delete')
    return 'DELETE FROM ' + table(spec) + ' WHERE id=$1'

def query_source(sql, params):
    arguments = ', '.join(json.dumps(x, ensure_ascii=False) for x in params)
    args = arguments + ' listesi' if params else 'boş liste'
    return ('parametreler ' + args + ' olsun\nsonuç ' + json.dumps(sql) + ' sorgusunu parametreler ile okumayı dene olsun\n'
            'sonuç başarılıysa\n    kayıtlar sonucun değeri olsun\n    kayıtların json metni yaz\ndeğilse\n    "SQL_REJECTED" yaz\n').encode()

def crud_source(spec, operation, data):
    params = values(spec, data, operation)
    if operation in {'list', 'read'}:
        return query_source(select_sql(spec, operation), params)
    args = ', '.join(json.dumps(x, ensure_ascii=False) for x in params) + ' listesi'
    return ('eylem değiştir\n    TamSayı sonucu döndürür\n    parametreler ' + args + ' olsun\n    sonuç ' + json.dumps(action_sql(spec, operation)) + ' sorgusunu parametreler ile değiştirmeyi dene olsun\n    sonuç döndür\n'
            'sonuç değiştir olsun\nsonuç başarılıysa\n    sonucun değeri yaz\ndeğilse\n    "SQL_REJECTED" yaz\n').encode()

def migration(spec, operation, initial=False):
    definition(spec)
    t = table(spec)
    meta = t + '_meta'
    archived = 'zee_mod_' + spec['id'] + '_archive'
    fingerprint = digest(encode(spec))
    if initial:
        need(operation == 'up')
        fields = ','.join('"' + f['name'] + '" ' + {'text': 'text', 'integer': 'bigint', 'boolean': 'boolean'}[f['type']] + ' NOT NULL' for f in spec['fields'])
        return (f"CREATE TABLE {meta}(singleton boolean PRIMARY KEY CHECK(singleton),schema_hash text NOT NULL,state text NOT NULL CHECK(state IN ('up','down')));\n"
                f"CREATE TABLE {t}(id text PRIMARY KEY CHECK(length(id) BETWEEN 1 AND 128),{fields});\n"
                f"INSERT INTO {meta} VALUES (true,'{fingerprint}','up');\n").encode()
    need(operation in {'up', 'down'})
    before = 'down' if operation == 'up' else 'up'
    guard = f"SELECT 1/(CASE WHEN (SELECT count(*) FROM {meta})=1 AND EXISTS(SELECT 1 FROM {meta} WHERE singleton AND schema_hash='{fingerprint}' AND state='{before}') THEN 1 ELSE 0 END);\n"
    rename = f'ALTER TABLE public.{archived} RENAME TO zee_mod_{spec["id"]};' if operation == 'up' else f'ALTER TABLE {t} RENAME TO {archived};'
    return (guard + rename + f"\nUPDATE {meta} SET state='{operation}' WHERE singleton;\n").encode()

Parent = BASE['Manager'] if BASE else object
class Manager(Parent):
    def __init__(self, root, binary, db_profile=None):
        need(BASE is not None and callable(getattr(os, 'waitid', None)) and hasattr(os, 'WNOWAIT'))
        self.db_profile = profile(db_profile if db_profile is not None else BASE['decode'](BASE['snapshot'](Path(root) / 'yonetim/sql-profile.json')))
        super().__init__(root, binary)
        self.sqlroot = self.control / 'sql'
        self.in_transition = False
    def render(self, receipt):
        result = super().render(receipt)
        manifest = result['proje.dil'].decode().replace('yetkinlikler "ağ-sunucusu" listesi olsun', 'yetkinlikler "ağ-sunucusu", "veritabanı" listesi olsun')
        manifest += 'veritabanı_hedefi ' + json.dumps(self.db_profile['target']) + ' olsun\nveritabanı_bağlantı_değişkeni "ZEE_MODULE_DATABASE_URL" olsun\nveritabanı_göçleri "yonetim/sql/gocler" olsun\n'
        result['proje.dil'] = manifest.encode()
        return result
    def init(self, port):
        super().init(port)
        self.sqlroot.mkdir(mode=0o700)
        (self.sqlroot / 'gocler').mkdir(mode=0o700)
        BASE['write'](self.control / 'sql-profile.json', encode(self.db_profile))
        BASE['write'](self.control / 'sql-state.json', encode({'schema': 'zee-sql-module-state-1', 'root': list(self.identity), 'profile': digest(encode(self.db_profile)), 'sequence': 0, 'modules': {}, 'migrations': {}}))
    def validate_state(self, s):
        need(type(s) is dict and set(s) == {'schema', 'root', 'profile', 'sequence', 'modules', 'migrations'})
        need(s['schema'] == 'zee-sql-module-state-1' and s['root'] == list(self.identity) and s['profile'] == digest(encode(self.db_profile)))
        need(type(s['sequence']) is int and 0 <= s['sequence'] <= 256 and type(s['modules']) is dict and len(s['modules']) <= 8)
        for name, entry in s['modules'].items():
            need(type(entry) is dict and set(entry) == {'definition', 'state'} and entry['state'] in {'up', 'down'})
            need(definition(entry['definition'])['id'] == name)
        need(type(s['migrations']) is dict and len(s['migrations']) == s['sequence'])
        need(set(s['migrations']) == {f'{n:06d}_module.sql' for n in range(1, s['sequence'] + 1)})
        for h in s['migrations'].values():
            need(type(h) is str and re.fullmatch('[0-9a-f]{64}', h))
        return s
    def state(self):
        self.directory(self.sqlroot)
        self.directory(self.sqlroot / 'gocler')
        need(profile(BASE['decode'](BASE['snapshot'](self.control / 'sql-profile.json'))) == self.db_profile)
        s = BASE['decode'](BASE['snapshot'](self.control / 'sql-state.json'))
        self.validate_state(s)
        actual = set()
        with os.scandir(self.sqlroot / 'gocler') as entries:
            for entry in entries:
                need(len(actual) < 256 and entry.name not in actual)
                actual.add(entry.name)
        need(actual == set(s['migrations']))
        for name, h in s['migrations'].items():
            need(re.fullmatch(r'[0-9]{6}_module.sql', name) and type(h) is str and re.fullmatch('[0-9a-f]{64}', h))
            need(digest(BASE['snapshot'](self.sqlroot / 'gocler' / name)) == h)
        return s
    def ready(self):
        need(self.in_transition or not any((self.control / n).exists() for n in ['sql-pending.json', 'sql-write-pending.json']))
        r = super().ready()
        if not self.in_transition:
            s = self.state()
            need(set(s['modules']) == set(r['modules']))
            if getattr(self, 'launching', False):
                need(all(x['state'] == 'up' for x in s['modules'].values()))
        return r
    def native(self, source, migrations=None, command='çalıştır'):
        need(self.large_binary() == self.binary_hash)
        with tempfile.TemporaryDirectory(prefix='zee-module-sql-') as td:
            p = Path(td).resolve()
            os.chmod(p, 0o700)
            manifest = 'proje "module-sql-operation" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\ngiriş "ana.dil" olsun\nyetkinlikler "veritabanı" listesi olsun\nveritabanı_hedefi ' + json.dumps(self.db_profile['target']) + ' olsun\nveritabanı_bağlantı_değişkeni "ZEE_MODULE_DATABASE_URL" olsun\nveritabanı_göçleri "gocler" olsun\n'
            BASE['write'](p / 'proje.dil', manifest.encode())
            BASE['write'](p / 'ana.dil', source)
            (p / 'gocler').mkdir(mode=0o700)
            for name, raw in (migrations or {}).items():
                need(re.fullmatch('[0-9]{6}_module.sql', name))
                BASE['write'](p / 'gocler' / name, raw)
            env = {k: v for k, v in os.environ.items() if not k.startswith(('GIT_', 'ZEE_'))}
            for key in ['ZEE_MODULE_DATABASE_URL', 'ZEE_MODULE_DATABASE_URL_TLS_CA_PEM']:
                if key in os.environ:
                    env[key] = os.environ[key]
            env.update(HOME=str(p), TMPDIR=str(p))
            try:
                BASE['H']['run']([self.binary, 'kilitle', '.'], cwd=p, env=env, timeout=15, limit=65536)
                result = BASE['H']['run']([self.binary, command, '.'], cwd=p, env=env, timeout=20, limit=65536)
            finally:
                need(self.large_binary() == self.binary_hash)
            return result
    def inspect(self, spec, desired):
        relation = table(spec) + ('_archive' if desired == 'down' else '')
        columns = 'id:text:true,' + ','.join(f['name'] + ':' + {'text': 'text', 'integer': 'bigint', 'boolean': 'boolean'}[f['type']] + ':true' for f in spec['fields'])
        sql = f"SELECT schema_hash::text AS schema_hash,state::text AS state,(SELECT string_agg(a.attname||':'||pg_catalog.format_type(a.atttypid,a.atttypmod)||':'||a.attnotnull::text,',' ORDER BY a.attnum) FROM pg_catalog.pg_attribute a WHERE a.attrelid='{relation}'::regclass AND a.attnum>0 AND NOT a.attisdropped)::text AS columns FROM {table(spec)}_meta WHERE singleton"
        result = BASE['decode'](self.native(query_source(sql, [])))
        need(result == [{'schema_hash': digest(encode(spec)), 'state': desired, 'columns': columns}])
    def crud(self, name, operation, data):
        ident(name)
        with self.locked():
            self.noactive(); self.ready(); s = self.state()
            need(name in s['modules'] and s['modules'][name]['state'] == 'up')
            spec = s['modules'][name]['definition']
            self.inspect(spec, 'up')
            source = crud_source(spec, operation, data)
            if operation not in {'read', 'list'}:
                pending = {'schema': 'zee-sql-module-write-pending-1', 'root': list(self.identity), 'binary': self.binary_hash, 'module': name, 'operation': operation, 'request_sha256': digest(encode(data)), 'id': data['id']}
                BASE['write'](self.control / 'sql-write-pending.json', encode(pending))
            result = self.native(source)
            if operation in {'read', 'list'}:
                rows = BASE['decode'](result)
                need(type(rows) is list and len(rows) <= 100 and all(type(x) is dict and set(x) == {'id'} | {f['name'] for f in spec['fields']} and all(type(v) is str for v in x.values()) for x in rows))
                return rows
            need(re.fullmatch(rb'[01]\n', result))
            (self.control / 'sql-write-pending.json').unlink()
            try: BASE['syncdir'](self.control)
            except OSError: raise Visible('write-completion-visible-durability-unconfirmed') from None
            return {'schema': 'zee-sql-module-result-1', 'status': 'completed', 'affected': int(result.strip())}
    def candidate(self, receipt, spec):
        # K273's original manifest and package contract remain unchanged.
        i = spec['id']
        m = {'schema': 'zee-modul-1', 'id': i, 'version': '0.1.0', 'core_min': '0.1.0', 'core_max_exclusive': '0.2.0', 'package_name': i, 'package_root': 'paket', 'route_adapter': 'rota.dil', 'menu': {'slug': i, 'label': spec['label'], 'path': '/' + i}, 'content_type': i, 'migration_range': [10000 + len(receipt['modules']) * 100, 10099 + len(receipt['modules']) * 100], 'files': {}}
        files = {'paket/ana.dil': b'# Pure module package; SQL belongs to the application adapter.\n', 'paket/proje.dil': f'proje "{i}" olsun\nsürüm "0.1.0" olsun\nmorfoloji "zee-tr-1" olsun\ngiriş "ana.dil" olsun\nyetkinlikler boş liste olsun\nyerel_bağımlılıklar boş liste olsun\n'.encode(), 'rota.dil': f'GET "/{i}" adresine istek geldiğinde\n    '.encode() + json.dumps(spec['label'], ensure_ascii=False).encode() + ' yanıtını gönder\n'.encode()}
        route = query_source(select_sql(spec, 'list'), []).decode().replace('kayıtların json metni yaz', 'metin kayıtların json metni olsun\n    guvenli metnin html güvenlisi olsun\n    \"<pre>\" ile guvenli ile \"</pre>\" yanıtını gönder').replace('"SQL_REJECTED" yaz', '"module-sql-unavailable" yanıtını 503 durumuyla gönder')
        files['rota.dil'] = (f'GET "/{i}" adresine istek geldiğinde\n' + ''.join('    ' + line + '\n' for line in route.splitlines())).encode()
        m['files'] = {n: digest(b) for n, b in files.items()}
        BASE['metadata'](m)
        data = {n: BASE['snapshot'](self.root / n) for n in receipt['files']}
        for n, b in files.items():
            target = 'modul_' + i + '.dil' if n == 'rota.dil' else 'modules/' + i + '/' + n
            need(target not in data); data[target] = b
        receipt['modules'][i] = m
        data.update(self.render(receipt))
        data = self.validate(data)
        receipt['files'] = {n: digest(b) for n, b in data.items()}
        return receipt, data
    def transition(self, operation, spec_or_name):
        with self.locked():
            self.noactive(); r = self.ready(); old = self.state(); s = copy.deepcopy(old)
            initial = type(spec_or_name) is dict
            if initial:
                spec = definition(spec_or_name); name = spec['id']
                need(operation == 'up' and name not in s['modules'] and name not in r['retired'] and len(s['modules']) < 8)
                candidate, data = self.candidate(copy.deepcopy(r), spec)
            else:
                name = ident(spec_or_name); need(name in s['modules'])
                spec = s['modules'][name]['definition']
                need(s['modules'][name]['state'] != operation)
                self.inspect(spec, s['modules'][name]['state'])
                candidate, data = r, {n: BASE['snapshot'](self.root / n) for n in r['files']}
            if not old['modules']:
                empty = BASE['decode'](self.native(query_source("SELECT count(*)::text AS relations FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relkind IN ('r','p','v','m','f')", [])))
                need(empty == [{'relations': '0'}])
            s['sequence'] += 1; need(s['sequence'] <= 256)
            name_sql = f'{s["sequence"]:06d}_module.sql'
            raw = migration(spec, operation, initial)
            s['migrations'][name_sql] = digest(raw)
            s['modules'][name] = {'definition': spec, 'state': operation}
            pending = {'schema': 'zee-sql-module-pending-1', 'root': list(self.identity), 'binary': self.binary_hash, 'old_state': digest(encode(old)), 'state': s, 'name': name, 'migration': name_sql, 'projection': {'receipt': candidate, 'data': {n: base64.b64encode(b).decode() for n, b in data.items()}}}
            need(len(encode(pending)) <= BASE['MAX_BYTES'])
            BASE['write'](self.control / 'sql-pending.json', encode(pending))
            BASE['write'](self.sqlroot / 'gocler' / name_sql, raw)
            migrations = {n: BASE['snapshot'](self.sqlroot / 'gocler' / n) for n in s['migrations']}
            self.native(b'"MIGRATION_PROJECT" yaz\n', migrations, 'göçür')
            self.finish_sql(pending)
    def finish_sql(self, pending):
        need(type(pending) is dict and set(pending) == {'schema', 'root', 'binary', 'old_state', 'state', 'name', 'migration', 'projection'})
        need(pending['schema'] == 'zee-sql-module-pending-1' and pending['root'] == list(self.identity) and pending['binary'] == self.binary_hash)
        s = self.validate_state(pending['state']); name = ident(pending['name']); spec = definition(s['modules'][name]['definition'])
        old_raw = BASE['snapshot'](self.control / 'sql-state.json')
        need(digest(old_raw) in {pending['old_state'], digest(encode(s))})
        old = self.validate_state(BASE['decode'](old_raw))
        if digest(old_raw) == pending['old_state']:
            expected = copy.deepcopy(old)
            expected['sequence'] += 1
            expected['modules'][name] = s['modules'][name]
            expected['migrations'][pending['migration']] = s['migrations'][pending['migration']]
            need(expected == s)
            if name in old['modules']:
                need(old['modules'][name]['definition'] == spec and old['modules'][name]['state'] != s['modules'][name]['state'])
            else: need(s['modules'][name]['state'] == 'up')
        need(pending['migration'] == f'{s["sequence"]:06d}_module.sql')
        migration_name = pending['migration']; need(re.fullmatch('[0-9]{6}_module.sql', migration_name))
        h = s['migrations'][migration_name]
        need(digest(BASE['snapshot'](self.sqlroot / 'gocler' / migration_name)) == h)
        for file, expected_hash in s['migrations'].items():
            need(digest(BASE['snapshot'](self.sqlroot / 'gocler' / file)) == expected_hash)
        self.inspect(spec, s['modules'][name]['state'])
        ledger = BASE['decode'](self.native(query_source('SELECT ad::text AS ad,sha256::text AS sha256 FROM public._zee_gocleri ORDER BY surum', [])))
        need(ledger == [{'ad': file, 'sha256': value} for file, value in sorted(s['migrations'].items())])
        self.in_transition = True
        try:
            projection = pending['projection']; need(type(projection) is dict and set(projection) == {'receipt', 'data'})
            data = {n: base64.b64decode(v, validate=True) for n, v in projection['data'].items()}
            need(len(data) <= BASE['MAX_FILES'] and sum(len(x) for x in data.values()) <= BASE['MAX_BYTES'])
            need(projection['receipt'].get('files') == {n: digest(b) for n, b in data.items()})
            self.record(projection['receipt'])
            for n in data: self.allowed(n)
            if (self.control / 'journal.json').exists():
                self.finish(BASE['decode'](BASE['snapshot'](self.control / 'journal.json')))
            else:
                self.publish(projection['receipt'], data)
            BASE['write'](self.control / 'sql-state.json', encode(s))
            self.state(); self.receipt()
            (self.control / 'sql-pending.json').unlink()
            try: BASE['syncdir'](self.control)
            except OSError: raise Visible('completion-visible-durability-unconfirmed') from None
        finally:
            self.in_transition = False
    def reconcile_sql(self):
        with self.locked():
            self.noactive()
            self.finish_sql(BASE['decode'](BASE['snapshot'](self.control / 'sql-pending.json')))
    def launch(self):
        self.launching = True
        try: return super().launch()
        finally: self.launching = False
    def reconcile_write(self, review):
        with self.locked():
            self.noactive(); self.receipt(); self.state()
            pending_raw = BASE['snapshot'](self.control / 'sql-write-pending.json')
            p = BASE['decode'](pending_raw)
            need(type(p) is dict and set(p) == {'schema', 'root', 'binary', 'module', 'operation', 'request_sha256', 'id'})
            need(p['schema'] == 'zee-sql-module-write-pending-1' and p['root'] == list(self.identity) and p['binary'] == self.binary_hash and p['operation'] in {'create', 'update', 'delete'})
            ident(p['module'])
            need(type(p['request_sha256']) is str and re.fullmatch('[0-9a-f]{64}', p['request_sha256']))
            need(type(p['id']) is str and 1 <= len(p['id']) <= 128 and '\x00' not in p['id'])
            s = self.state(); spec = definition(s['modules'][p['module']]['definition'])
            need(s['modules'][p['module']]['state'] == 'up')
            self.inspect(spec, 'up')
            rows = BASE['decode'](self.native(crud_source(spec, 'read', {'id': p['id']})))
            need(type(review) is dict and set(review) == {'schema', 'pending_sha256', 'observed', 'decision'})
            need(review['schema'] == 'zee-sql-module-write-review-1' and review['pending_sha256'] == digest(pending_raw) and review['observed'] == rows and review['decision'] == 'accept-observed-state-no-replay')
            BASE['write'](self.control / 'sql-write-review.json', encode(review))
            (self.control / 'sql-write-pending.json').unlink()
            try: BASE['syncdir'](self.control)
            except OSError: raise Visible('review-visible-durability-unconfirmed') from None
    def change(self, *args, **kwargs):
        raise Rejected('use-sql-lifecycle')
    def reconcile(self):
        raise Rejected('use-sql-reconcile')

def main():
    try:
        need(BASE is not None)
        class Parser(argparse.ArgumentParser):
            def error(self, message): raise Rejected('arguments')
        p = Parser(allow_abbrev=False)
        p.add_argument('--root', type=Path, required=True)
        p.add_argument('--binary', type=Path, required=True)
        p.add_argument('--profile', type=Path)
        p.add_argument('command', choices=['init', 'up', 'down', 'reapply', 'reconcile', 'reconcile-write', 'launch', 'create', 'read', 'update', 'delete', 'list'])
        p.add_argument('value', nargs='?')
        p.add_argument('--data', type=Path)
        a = p.parse_args()
        db = profile(BASE['decode'](BASE['snapshot'](a.profile))) if a.profile else None
        m = Manager(a.root, a.binary, db)
        old = {}
        def cancel(sig, frame): BASE['H']['CANCELLED'] = True
        try:
            for sig in (signal.SIGTERM, signal.SIGINT): old[sig] = signal.signal(sig, cancel)
            if a.command == 'init': m.init(int(a.value))
            elif a.command == 'up': m.transition('up', definition(BASE['decode'](BASE['snapshot'](Path(a.value)))))
            elif a.command in {'down', 'reapply'}: m.transition('down' if a.command == 'down' else 'up', a.value)
            elif a.command == 'reconcile': m.reconcile_sql()
            elif a.command == 'reconcile-write': m.reconcile_write(BASE['decode'](BASE['snapshot'](a.data)))
            elif a.command == 'launch': return m.launch()
            else:
                result = m.crud(a.value, a.command, BASE['decode'](BASE['snapshot'](a.data)) if a.data else {})
                sys.stdout.buffer.write(encode(result))
        finally:
            for sig, handler in old.items(): signal.signal(sig, handler)
        return 0
    except BaseException as error:
        if isinstance(error, (KeyboardInterrupt, SystemExit)) and not isinstance(error, Exception):
            return 2
        visible = isinstance(error, Visible) or (BASE and isinstance(error, BASE['Published']))
        sys.stderr.write('sql-module-completion-visible-durability-unconfirmed\n' if visible else 'sql-module-operation-rejected\n')
        return 4 if visible else 2

if __name__ == '__main__':
    sys.exit(main())
