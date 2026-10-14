//! Yapısal belge tazelik kapısı: yeni karar/spec dosyası indekslenmeden ve
//! yerel Markdown bağlantısı kırıkken toplu iş tamamlanamaz.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

fn markdown_dosyalari(kok: &Path) -> Vec<PathBuf> {
    fn gez(klasor: &Path, sonuc: &mut Vec<PathBuf>) {
        let mut girdiler = std::fs::read_dir(klasor)
            .expect("belge klasörü okunmalı")
            .collect::<Result<Vec<_>, _>>()
            .expect("belge girdileri okunmalı");
        girdiler.sort_by_key(|girdi| girdi.file_name());
        for girdi in girdiler {
            let yol = girdi.path();
            let tur = girdi.file_type().expect("dosya türü");
            if tur.is_dir() {
                gez(&yol, sonuc);
            } else if tur.is_file() && yol.extension().and_then(|e| e.to_str()) == Some("md") {
                sonuc.push(yol);
            }
        }
    }
    let mut sonuc = Vec::new();
    gez(kok, &mut sonuc);
    sonuc
}

fn indeksi_dogrula(klasor: &Path, indeks: &Path, on_ek_uzunlugu: usize) {
    let metin = std::fs::read_to_string(indeks).expect("indeks okunmalı");
    for yol in markdown_dosyalari(klasor) {
        if yol == indeks {
            continue;
        }
        let ad = yol
            .file_name()
            .and_then(|ad| ad.to_str())
            .expect("UTF-8 ad");
        let numarali = ad
            .chars()
            .take(on_ek_uzunlugu)
            .all(|karakter| karakter.is_ascii_digit());
        if numarali {
            assert!(
                metin.contains(&format!("({})", ad)),
                "{} dosyası {} içinde indekslenmemiş",
                yol.display(),
                indeks.display()
            );
        }
    }
}

fn yerel_baglantilari_dogrula(dosya: &Path) {
    let metin = std::fs::read_to_string(dosya).expect("Markdown okunmalı");
    for parca in metin.split("](").skip(1) {
        let Some(mut hedef) = parca.split(')').next() else {
            continue;
        };
        hedef = hedef.trim().trim_start_matches('<').trim_end_matches('>');
        if hedef.is_empty()
            || hedef.starts_with('#')
            || hedef.starts_with("http://")
            || hedef.starts_with("https://")
            || hedef.starts_with("mailto:")
        {
            continue;
        }
        hedef = hedef.split('#').next().unwrap_or(hedef);
        let cozulmus = dosya.parent().expect("belge ebeveyni").join(hedef);
        assert!(
            cozulmus.exists(),
            "{} içindeki {:?} bağlantısı var olmayan {} hedefine gidiyor",
            dosya.display(),
            hedef,
            cozulmus.display()
        );
    }
}

fn numarali_belgeler(klasor: &Path, basamak: usize, sablon: &str) -> BTreeSet<String> {
    markdown_dosyalari(klasor)
        .into_iter()
        .filter_map(|yol| {
            let ad = yol.file_name()?.to_str()?;
            let on_ek = ad.as_bytes().get(..basamak)?;
            if on_ek.iter().all(u8::is_ascii_digit)
                && ad.as_bytes().get(basamak) == Some(&b'-')
                && &ad[..basamak] != sablon
            {
                Some(ad.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn rfc_adr_ve_spec_dosyalari_indeksli() {
    let depo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü");
    indeksi_dogrula(&depo.join("rfcs"), &depo.join("rfcs/README.md"), 4);
    indeksi_dogrula(&depo.join("adr"), &depo.join("adr/README.md"), 3);
    indeksi_dogrula(&depo.join("spec"), &depo.join("spec/README.md"), 2);
}

#[test]
fn rfc_adr_ve_spec_numaralari_benzersizdir() {
    // K-164 sonrası kural: aynı numarayı taşıyan iki karar/spec dosyası
    // (örn. iki ayrı `spec/27-*.md`) sessizce yan yana yaşayamaz; numara
    // kimliktir, indeks ve kanıt haritası ona bağlanır. Paralel dallarda
    // doğan çakışma birleştirmede burada yakalanır.
    let depo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü");
    for (klasor, basamak, sablon) in [("rfcs", 4, "0000"), ("adr", 3, "000"), ("spec", 2, "00")] {
        let mut gorulen: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for ad in numarali_belgeler(&depo.join(klasor), basamak, sablon) {
            gorulen
                .entry(ad[..basamak].to_string())
                .or_default()
                .push(ad);
        }
        let cakisan = gorulen
            .iter()
            .filter(|(_, dosyalar)| dosyalar.len() > 1)
            .map(|(numara, dosyalar)| format!("{klasor}/{numara}: {}", dosyalar.join(", ")))
            .collect::<Vec<_>>();
        assert!(
            cakisan.is_empty(),
            "aynı numaralı belge çakışması; birini yeniden numaralandır ve indeksi/kanıt haritasını güncelle:\n{}",
            cakisan.join("\n")
        );
    }
}

#[test]
fn yerel_markdown_baglantilari_bayat_degil() {
    let depo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü");
    let mut dosyalar = vec![depo.join("README.md"), depo.join("AGENTS.md")];
    for klasor in ["docs", "rfcs", "adr", "spec"] {
        dosyalar.extend(markdown_dosyalari(&depo.join(klasor)));
    }
    for dosya in dosyalar {
        yerel_baglantilari_dogrula(&dosya);
    }
}

#[test]
fn her_normatif_belge_makine_okunur_test_kaniti_tasir() {
    let depo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü");
    let mut beklenen = BTreeSet::new();
    for (klasor, basamak, sablon) in [("rfcs", 4, "0000"), ("adr", 3, "000"), ("spec", 2, "00")] {
        beklenen.extend(
            numarali_belgeler(&depo.join(klasor), basamak, sablon)
                .into_iter()
                .map(|ad| format!("{klasor}/{ad}")),
        );
    }

    let harita_yolu = depo.join("docs/kanit-haritasi-v1.tsv");
    let harita = std::fs::read_to_string(&harita_yolu).expect("kanıt haritası okunmalı");
    let mut gorulen = BTreeSet::new();
    for (sira, satir) in harita.lines().enumerate() {
        if satir.is_empty() || satir.starts_with('#') {
            continue;
        }
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        assert_eq!(
            alanlar.len(),
            4,
            "kanıt haritası satırı {} dört sekmeli alan taşımalı",
            sira + 1
        );
        let [belge, durum, testler, not] = alanlar.as_slice() else {
            unreachable!("alan sayısı doğrulandı")
        };
        assert!(
            beklenen.contains(*belge),
            "bilinmeyen kanıt belgesi: {belge}"
        );
        assert!(
            gorulen.insert((*belge).to_string()),
            "yinelenen kanıt belgesi: {belge}"
        );
        assert!(matches!(*durum, "kanitli" | "kismi" | "taslak"));
        assert!(!not.trim().is_empty(), "{belge}: kapsam notu boş olamaz");

        if *durum == "taslak" {
            assert_eq!(*testler, "-", "{belge}: taslak kanıtı '-' olmalı");
            continue;
        }
        assert_ne!(*testler, "-", "{belge}: {durum} belge test kanıtı ister");
        for test in testler.split(',') {
            let test_yolu = depo.join(test);
            assert!(test_yolu.is_file(), "{belge}: test kanıtı yok: {test}");
            assert_eq!(
                test_yolu.extension().and_then(|uzanti| uzanti.to_str()),
                Some("rs"),
                "{belge}: kanıt Rust test dosyası olmalı: {test}"
            );
            let kaynak = std::fs::read_to_string(&test_yolu).expect("test kanıtı okunmalı");
            assert!(
                kaynak.contains("#[test]") || kaynak.contains("```compile_fail"),
                "{belge}: {test} yürütülebilir test taşımıyor"
            );
        }
    }

    assert_eq!(
        gorulen, beklenen,
        "kanıt haritası bütün RFC/ADR/spec belgelerini birebir kapsamalı"
    );
    assert_eq!(
        gorulen.len(),
        175,
        "şema-1 belge tabanı beklenmedik biçimde değişti"
    );
}

#[test]
fn readme_canli_depo_sayilari_ureticisiyle_gunceldir() {
    let depo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü");
    let cikti = Command::new(env!("CARGO_BIN_EXE_depo_sayilari"))
        .arg("--denetle")
        .current_dir(depo)
        .output()
        .expect("depo sayıları aracı çalışmalı");
    assert!(
        cikti.status.success(),
        "{}{}",
        String::from_utf8_lossy(&cikti.stdout),
        String::from_utf8_lossy(&cikti.stderr)
    );
}

#[test]
fn kaynak_arsivi_build_ve_kisisel_artifaktlari_disarida_tutar() {
    let depo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü");
    let gitignore =
        std::fs::read_to_string(depo.join(".gitignore")).expect("kök .gitignore okunmalı");
    for kalip in [
        "target/",
        "compiler/fuzz/artifacts/",
        "__MACOSX/",
        "*.profraw",
        "*.profdata",
        "*.zip",
    ] {
        assert!(
            gitignore.lines().any(|satir| satir == kalip),
            "kaynak arşivi hijyeni için .gitignore kalıbı eksik: {kalip}"
        );
    }

    let script = std::fs::read_to_string(depo.join("scripts/temiz-kaynak-arsivi.sh"))
        .expect("temiz arşiv scripti okunmalı");
    assert!(script.contains("git archive --format=zip"));
    assert!(script.contains("unzip -Z1"));
    assert!(script.contains("__MACOSX|target|artifacts"));
    assert!(script.contains("KAYNAK-SHA256.txt"));
    assert!(script.contains("git rev-parse --verify 'HEAD^{commit}'"));
    assert!(script.contains("git ls-tree -r -z --name-only \"$sha\""));
    assert!(script.contains("git show \"$sha:$dosya\""));
    assert!(script.contains("verifier.zip_hashes"));
    #[cfg(unix)]
    {
        // Aynı gerçek Cargo vakası kaynak aracının kontrolsüz HEAD okumasını
        // ve doğrulanmamış ZIP yayınlamasını izole gerçek Git ile sınar.
        let sonuc = Command::new("python3")
            .args(["-I", "-c"])
            .arg(
                r#"
from pathlib import Path
import hashlib, os, shutil, subprocess, sys, tempfile, zipfile

source = Path(sys.argv[1])
real_git = shutil.which('git')
assert real_git
with tempfile.TemporaryDirectory(prefix='zee-archive-k243-') as temp:
    base = Path(temp).resolve()
    repo = base / 'repo'
    repo.mkdir()
    def git(*args):
        return subprocess.check_output([real_git, '-C', str(repo), *args])
    git('init', '-q')
    git('config', 'user.name', 'Zee fixture')
    git('config', 'user.email', 'zee@example.invalid')
    git('config', 'commit.gpgsign', 'false')
    git('config', 'core.quotepath', 'true')
    (base / 'hooks').mkdir()
    git('config', 'core.hooksPath', str(base / 'hooks'))
    (repo / 'scripts').mkdir()
    for name in ['temiz-kaynak-arsivi.sh', 'kaynak-ci-esleme.py']:
        shutil.copyfile(source / 'scripts' / name, repo / 'scripts' / name)
    for name in ['a.txt', 'b.txt']:
        (repo / name).write_text('old ' + name + '\n')
    git('add', '.')
    git('commit', '-qm', 'synthetic source A')
    first = git('rev-parse', 'HEAD').decode().strip()
    for name in ['a.txt', 'b.txt']:
        (repo / name).write_text('new ' + name + '\n')
    git('add', '.')
    git('commit', '-qm', 'synthetic source B')
    second = git('rev-parse', 'HEAD').decode().strip()
    git('reset', '--hard', first)
    tools = base / 'tools'
    tools.mkdir()
    wrapper = tools / 'git'
    wrapper.write_text('''#!/usr/bin/env python3
import os, subprocess, sys, zipfile
from pathlib import Path
args = sys.argv[1:]
r = subprocess.run([os.environ['PROBE_REAL_GIT'], *args], capture_output=True)
mode = os.environ.get('PROBE_MODE')
if mode == 'move' and len(args) == 2 and args[0] == 'show' and args[1].endswith(':a.txt'):
    subprocess.run([os.environ['PROBE_REAL_GIT'], 'reset', '--hard', os.environ['PROBE_SECOND_SHA']], stdout=subprocess.DEVNULL, check=True)
if mode == 'tamper' and args[:2] == ['archive', '--format=zip'] and r.returncode == 0:
    path = Path(next(a.split('=', 1)[1] for a in args if a.startswith('--output=')))
    with zipfile.ZipFile(path) as original:
        entries = [(i, original.read(i)) for i in original.infolist()]
        comment = original.comment
    with zipfile.ZipFile(path, 'w') as altered:
        altered.comment = comment
        for item, data in entries:
            altered.writestr(item, b'tampered source' if item.filename == 'zee/a.txt' else data)
sys.stdout.buffer.write(r.stdout)
sys.stderr.buffer.write(r.stderr)
sys.exit(r.returncode)
''')
    wrapper.chmod(0o755)
    environment = {k: v for k, v in os.environ.items() if not k.startswith('PYTHON')}
    environment.update(PATH=str(tools) + os.pathsep + os.environ['PATH'],
                       PROBE_REAL_GIT=real_git, PROBE_SECOND_SHA=second)
    def run(output, mode='normal'):
        return subprocess.run(['bash', str(repo / 'scripts/temiz-kaynak-arsivi.sh'), str(output)],
                              cwd=repo, env=dict(environment, PROBE_MODE=mode),
                              capture_output=True, timeout=30)
    def verify(path, expected_sha, expected_a):
        with zipfile.ZipFile(path) as archive:
            assert archive.comment.decode() == expected_sha
            assert archive.read('zee/a.txt') == expected_a
            lines = archive.read('zee/KAYNAK-SHA256.txt').decode().splitlines()
            for line in lines:
                expected, name = line.split('  ', 1)
                assert hashlib.sha256(archive.read('zee/' + name)).hexdigest() == expected
        declared, filename = Path(str(path) + '.sha256').read_text().strip().split('  ', 1)
        assert filename == path.name
        assert declared == hashlib.sha256(path.read_bytes()).hexdigest()
    dirty_output = repo / 'dist/dirty.zip'
    (repo / 'a.txt').write_bytes(b'uncommitted caller source')
    result = run(dirty_output)
    assert result.returncode == 0, result.stderr
    verify(dirty_output, first, b'old a.txt\n')
    assert (repo / 'a.txt').read_bytes() == b'uncommitted caller source'
    git('reset', '--hard', first)
    moved_output = repo / 'dist/moved.zip'
    result = run(moved_output, 'move')
    assert result.returncode == 0, result.stderr
    assert git('rev-parse', 'HEAD').decode().strip() == second
    verify(moved_output, first, b'old a.txt\n')
    rejected = repo / 'dist/rejected.zip'
    shutil.copyfile(moved_output, rejected)
    Path(str(rejected) + '.sha256').write_text(Path(str(moved_output) + '.sha256').read_text().replace('moved.zip', 'rejected.zip'))
    before = {p: p.read_bytes() for p in [rejected, Path(str(rejected) + '.sha256')]}
    result = run(rejected, 'tamper')
    assert result.returncode != 0
    assert 'Kaynak ZIP/comment/manifest'.encode() in result.stderr
    assert all(p.read_bytes() == data for p, data in before.items())
    assert not list((repo / 'dist').glob('.zee-kaynak-arsivi.*'))
    result = run(rejected)
    assert result.returncode == 0, result.stderr
    verify(rejected, second, b'new a.txt\n')
    outside = base / 'outside'
    result = run(repo / '../outside/no-create.zip')
    assert result.returncode == 2 and not outside.exists()
    outside.mkdir()
    (repo / 'link').symlink_to(outside, target_is_directory=True)
    result = run(repo / 'link/no-create.zip')
    assert result.returncode == 2 and not (outside / 'no-create.zip').exists()
    protected = outside / 'protected'
    protected.write_bytes(b'foreign file')
    for suffix in ['', '.sha256']:
        destination = repo / 'dist' / ('symlink-' + ('zip' if not suffix else 'sidecar') + '.zip')
        Path(str(destination) + suffix).symlink_to(protected)
        result = run(destination)
        assert result.returncode == 2
        assert protected.read_bytes() == b'foreign file'
    for suffix in ['', '.sha256']:
        destination = repo / 'dist' / ('directory-' + ('zip' if not suffix else 'sidecar') + '.zip')
        Path(str(destination) + suffix).mkdir()
        result = run(destination)
        assert result.returncode == 2
        assert not list(Path(str(destination) + suffix).iterdir())
    result = run(repo / '.git/new/archive.zip')
    assert result.returncode == 2 and not (repo / '.git/new').exists()
    config_before = (repo / '.git/config').read_bytes()
    result = run(repo / '.GIT/CONFIG')
    assert result.returncode == 2 and (repo / '.git/config').read_bytes() == config_before
    alias = repo / 'A.TXT'
    if alias.exists() and alias.samefile(repo / 'a.txt'):
        source_before = (repo / 'a.txt').read_bytes()
        result = run(alias)
        assert result.returncode == 2 and (repo / 'a.txt').read_bytes() == source_before
    else:
        # Case-sensitive FS: a distinct untracked name is a legitimate output.
        result = run(alias)
        assert result.returncode == 0, result.stderr
        verify(alias, second, b'new a.txt\n')
    # Deletion must not turn a logical tracked name into a new alias output.
    missing_before = (repo / 'a.txt').read_bytes()
    insensitive = (repo / '.GIT').exists() and (repo / '.GIT').samefile(repo / '.git')
    (repo / 'a.txt').unlink()
    result = run(alias)
    if insensitive:
        assert result.returncode == 2 and not (repo / 'a.txt').exists()
    else:
        assert result.returncode == 0, result.stderr
        verify(alias, second, b'new a.txt\n')
        assert not (repo / 'a.txt').exists()
    (repo / 'a.txt').write_bytes(missing_before)
    before = (repo / 'a.txt').read_bytes()
    result = run(repo / 'a.txt')
    assert result.returncode == 2 and (repo / 'a.txt').read_bytes() == before
    # Index and frozen tree both protect file identity, not only a spelling.
    for frozen_only in [False, True]:
        if frozen_only:
            git('rm', '--cached', 'a.txt')
        for suffix in ['', '.sha256']:
            name = ('frozen' if frozen_only else 'indexed') + ('-sidecar' if suffix else '-output')
            destination = repo / 'dist' / (name + '.zip')
            identity_alias = Path(str(destination) + suffix)
            os.link(repo / 'a.txt', identity_alias)
            result = run(destination)
            assert result.returncode == 2
            assert identity_alias.read_bytes() == before and (repo / 'a.txt').read_bytes() == before
        if frozen_only:
            git('add', 'a.txt')
    tracked_sidecar = repo / 'tracked.zip.sha256'
    tracked_sidecar.write_bytes(b'tracked source sidecar')
    git('add', 'tracked.zip.sha256')
    git('commit', '-qm', 'synthetic tracked sidecar')
    result = run(repo / 'tracked.zip')
    assert result.returncode == 2 and not (repo / 'tracked.zip').exists()
    assert tracked_sidecar.read_bytes() == b'tracked source sidecar'
    import unicodedata
    composed = 'café.txt'
    decomposed = unicodedata.normalize('NFD', composed)
    (repo / composed).write_bytes(b'normalized synthetic source\n')
    git('config', 'core.precomposeunicode', 'true')
    git('add', composed)
    git('commit', '-qm', 'synthetic normalization source')
    normalizing = (repo / decomposed).exists() and (repo / decomposed).samefile(repo / composed)
    (repo / composed).unlink()
    result = run(repo / decomposed)
    if normalizing:
        assert result.returncode == 2 and not (repo / composed).exists()
    else:
        assert result.returncode == 0, result.stderr
        assert not (repo / composed).exists()
    (repo / composed).write_bytes(b'normalized synthetic source\n')
    (repo / 'Türkçe [1].dil').write_bytes(b'"merhaba" yaz\n')
    git('add', 'Türkçe [1].dil')
    git('commit', '-qm', 'synthetic UTF8 source')
    utf_sha = git('rev-parse', 'HEAD').decode().strip()
    utf_output = repo / 'dist/utf8.zip'
    result = run(utf_output)
    assert result.returncode == 0, result.stderr
    verify(utf_output, utf_sha, b'new a.txt\n')
    with zipfile.ZipFile(utf_output) as archive:
        assert archive.read('zee/Türkçe [1].dil') == b'"merhaba" yaz\n'
print('K243: snapshot/ZIP, confinement, present/deleted case+normalization aliases, hardlinks and UTF8 passed')
"#,
            )
            .arg(depo)
            .output()
            .expect("kaynak arşivinin gerçek izole Git kampanyası çalışmalı");
        assert!(
            sonuc.status.success(),
            "{}{}",
            String::from_utf8_lossy(&sonuc.stdout),
            String::from_utf8_lossy(&sonuc.stderr)
        );
    }
}
