#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented
    )
)]

//! `olcum` — faz ve kullanıcı yüzeyi performans gözlem koşucusu.
//!
//! Paylaşımlı CI sonucu correctness kapısı değildir. JSON, Markdown ve geçmiş
//! TSV artefaktları p50/p95 eğilimini görünür kılar; hard eşik ancak kararlı,
//! adanmış makinede açık `--esik-yuzde` seçeneğiyle etkinleşir.

use dil::agac::Program;
use dil::faz::KaynakMetni;
use dil::lsp::Sunucu;
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs;
use std::hint::black_box;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const SEMA: &str = "zee-performans-2";
const GECMIS_SEMASI: &str = "# zee-performans-gecmisi-2";
const VARSAYILAN_TUR: usize = 25;
const HIZLI_TUR: usize = 3;
const ISINMA_TURU: usize = 2;
const LSP_OLCEKLERI: [usize; 4] = [2_000, 5_000, 10_000, 20_000];
const LSP_ESIKLERI_MS: [u64; 3] = [250, 500, 1_000];

#[derive(Debug)]
struct Ayarlar {
    tur: usize,
    json: Option<PathBuf>,
    rapor: Option<PathBuf>,
    gecmis: Option<PathBuf>,
    gecmis_cikti: Option<PathBuf>,
    kayit: String,
    git_sha: String,
    milestone: String,
    dillsp: PathBuf,
    lsp_olcek: bool,
    esik_yuzde: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
struct Olcum {
    kimlik: &'static str,
    aciklama: &'static str,
    birim: &'static str,
    #[serde(rename = "sampling_semantics")]
    ornekleme: &'static str,
    #[serde(rename = "sample_count")]
    ornek_sayisi: usize,
    #[serde(rename = "warmup_count")]
    isinma_turu: usize,
    #[serde(rename = "raw_samples")]
    ornekler: Vec<u64>,
    en_az: u64,
    p50: u64,
    p95: u64,
    en_cok: u64,
}

#[derive(Debug, Serialize)]
struct Rapor<'a> {
    sema: &'static str,
    kayit: &'a str,
    git_sha: &'a str,
    milestone: &'a str,
    git_dirty: bool,
    unix_zamani: u64,
    platform: &'a str,
    os: &'a str,
    cpu: &'a str,
    #[serde(rename = "ram_bytes")]
    ram_bayt: u64,
    rustc: &'a str,
    #[serde(rename = "build_profile")]
    profil: &'static str,
    #[serde(rename = "sample_count")]
    tur: usize,
    #[serde(rename = "warmup_count")]
    isinma_turu: usize,
    esik_uygulandi: bool,
    olcumler: &'a [Olcum],
}

#[derive(Clone, Debug)]
struct GecmisSatiri {
    kayit: String,
    git_sha: String,
    milestone: String,
    git_dirty: bool,
    platform: String,
    os: String,
    cpu: String,
    ram_bayt: u64,
    rustc: String,
    profil: String,
    ornek_sayisi: usize,
    isinma_sayisi: usize,
    ornekleme: String,
    kimlik: String,
    birim: String,
    p50: u64,
    p95: u64,
}

fn deger_iste(
    argumanlar: &mut impl Iterator<Item = String>,
    secenek: &str,
) -> Result<String, String> {
    argumanlar
        .next()
        .ok_or_else(|| format!("{secenek} bir değer ister"))
}

fn komut_satirini_oku() -> Result<Ayarlar, String> {
    let mut tur = VARSAYILAN_TUR;
    let mut json = None;
    let mut rapor = None;
    let mut gecmis = None;
    let mut gecmis_cikti = None;
    let mut kayit = None;
    let mut git_sha = None;
    let mut milestone = None;
    let mut dillsp = None;
    let mut lsp_olcek = false;
    let mut esik_yuzde = None;
    let mut argumanlar = env::args().skip(1);
    while let Some(arguman) = argumanlar.next() {
        match arguman.as_str() {
            "--hizli" => tur = HIZLI_TUR,
            "--tur" => {
                let metin = deger_iste(&mut argumanlar, "--tur")?;
                tur = metin
                    .parse()
                    .map_err(|_| format!("--tur pozitif sayı olmalı: {metin}"))?;
            }
            "--json" => json = Some(PathBuf::from(deger_iste(&mut argumanlar, "--json")?)),
            "--rapor" => {
                rapor = Some(PathBuf::from(deger_iste(&mut argumanlar, "--rapor")?));
            }
            "--gecmis" => {
                gecmis = Some(PathBuf::from(deger_iste(&mut argumanlar, "--gecmis")?));
            }
            "--gecmis-cikti" => {
                gecmis_cikti = Some(PathBuf::from(deger_iste(
                    &mut argumanlar,
                    "--gecmis-cikti",
                )?));
            }
            "--kayit" => kayit = Some(deger_iste(&mut argumanlar, "--kayit")?),
            "--git-sha" => git_sha = Some(deger_iste(&mut argumanlar, "--git-sha")?),
            "--milestone" => milestone = Some(deger_iste(&mut argumanlar, "--milestone")?),
            "--dillsp" => dillsp = Some(PathBuf::from(deger_iste(&mut argumanlar, "--dillsp")?)),
            "--lsp-olcek" => lsp_olcek = true,
            "--esik-yuzde" => {
                let metin = deger_iste(&mut argumanlar, "--esik-yuzde")?;
                esik_yuzde = Some(
                    metin
                        .parse()
                        .map_err(|_| format!("--esik-yuzde doğal sayı olmalı: {metin}"))?,
                );
            }
            "--yardim" | "-h" => {
                return Err(
                    "kullanım: olcum [--hizli|--tur N] [--json YOL] [--rapor YOL] \
                     [--gecmis YOL] [--gecmis-cikti YOL] [--kayit AD] \
                     [--git-sha SHA] [--milestone AD] [--dillsp YOL] [--lsp-olcek] \
                     [--esik-yuzde N]"
                        .into(),
                );
            }
            _ => return Err(format!("bilinmeyen argüman: {arguman}")),
        }
    }
    if tur == 0 {
        return Err("--tur sıfır olamaz".into());
    }
    if esik_yuzde.is_some() && gecmis.is_none() {
        return Err("--esik-yuzde aynı exact ortamdan --gecmis ister".into());
    }
    let unix_zamani = unix_zamani()?;
    Ok(Ayarlar {
        tur,
        json,
        rapor,
        gecmis,
        gecmis_cikti,
        kayit: kayit.unwrap_or_else(|| format!("yerel-{unix_zamani}")),
        git_sha: git_sha.map_or_else(git_revizyonu, Ok)?,
        milestone: milestone.unwrap_or_else(|| "yerel".into()),
        dillsp: dillsp.map_or_else(varsayilan_dillsp_yolu, Ok)?,
        lsp_olcek,
        esik_yuzde,
    })
}

fn unix_zamani() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|sure| sure.as_secs())
        .map_err(|hata| format!("sistem saati UNIX çağından önce: {hata}"))
}

fn komutun_tek_satiri(program: &str, argumanlar: &[&str]) -> Option<String> {
    let cikti = Command::new(program).args(argumanlar).output().ok()?;
    if !cikti.status.success() {
        return None;
    }
    let satir = String::from_utf8(cikti.stdout).ok()?;
    Some(satir.trim().to_string())
}

fn gecerli_git_sha(deger: &str) -> bool {
    deger.len() == 40
        && deger
            .bytes()
            .all(|bayt| bayt.is_ascii_hexdigit() && !bayt.is_ascii_uppercase())
}

fn git_revizyonu() -> Result<String, String> {
    let sha = komutun_tek_satiri("git", &["rev-parse", "HEAD"])
        .ok_or_else(|| "Git HEAD çözülemedi".to_string())?;
    if !gecerli_git_sha(&sha) {
        return Err(format!("Git HEAD tam 40 haneli commit SHA değil: {sha}"));
    }
    Ok(sha)
}

fn varsayilan_dillsp_yolu() -> Result<PathBuf, String> {
    let mut yol = env::current_exe()
        .map_err(|hata| format!("çalışan ölçüm ikilisinin yolu bulunamadı: {hata}"))?;
    yol.set_file_name(format!("dillsp{}", env::consts::EXE_SUFFIX));
    Ok(yol)
}

fn platform() -> String {
    format!("{}-{}", env::consts::OS, env::consts::ARCH)
}

fn temiz_tek_satir(metin: String) -> String {
    metin
        .chars()
        .map(|karakter| match karakter {
            '\t' | '\r' | '\n' => ' ',
            baska => baska,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn cpu_adi() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(metin) = fs::read_to_string("/proc/cpuinfo") {
            if let Some(ad) = metin.lines().find_map(|satir| {
                satir
                    .split_once(':')
                    .filter(|(anahtar, _)| *anahtar == "model name")
                    .map(|(_, deger)| deger.trim())
            }) {
                return temiz_tek_satir(ad.to_string());
            }
        }
    }
    #[cfg(target_os = "macos")]
    if let Some(ad) = komutun_tek_satiri("sysctl", &["-n", "machdep.cpu.brand_string"]) {
        return temiz_tek_satir(ad);
    }
    #[cfg(target_os = "windows")]
    if let Ok(ad) = env::var("PROCESSOR_IDENTIFIER") {
        return temiz_tek_satir(ad);
    }
    "bilinmiyor".into()
}

fn os_surumu() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(metin) = fs::read_to_string("/etc/os-release") {
            if let Some(ad) = metin.lines().find_map(|satir| {
                satir
                    .strip_prefix("PRETTY_NAME=")
                    .map(|deger| deger.trim_matches('"'))
            }) {
                let cekirdek = komutun_tek_satiri("uname", &["-sr"])
                    .unwrap_or_else(|| "çekirdek bilinmiyor".into());
                return temiz_tek_satir(format!("{ad}; {cekirdek}"));
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(surum) = komutun_tek_satiri("sw_vers", &["-productVersion"]) {
            let derleme = komutun_tek_satiri("sw_vers", &["-buildVersion"])
                .unwrap_or_else(|| "bilinmiyor".into());
            return temiz_tek_satir(format!("macOS {surum} ({derleme})"));
        }
    }
    #[cfg(target_os = "windows")]
    if let Some(surum) = komutun_tek_satiri("cmd", &["/C", "ver"]) {
        return temiz_tek_satir(surum);
    }
    "bilinmiyor".into()
}

fn toplam_ram_bayt() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let metin = fs::read_to_string("/proc/meminfo").ok()?;
        let kib = metin.lines().find_map(|satir| {
            let deger = satir.strip_prefix("MemTotal:")?.trim();
            deger.strip_suffix(" kB")?.trim().parse::<u64>().ok()
        })?;
        return kib.checked_mul(1024);
    }
    #[cfg(target_os = "macos")]
    {
        return komutun_tek_satiri("sysctl", &["-n", "hw.memsize"])?
            .parse()
            .ok();
    }
    #[cfg(target_os = "windows")]
    {
        return komutun_tek_satiri(
            "powershell",
            &[
                "-NoProfile",
                "-Command",
                "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
            ],
        )?
        .parse()
        .ok();
    }
    #[allow(unreachable_code)]
    None
}

fn rustc_surumu() -> String {
    komutun_tek_satiri("rustc", &["--version"]).unwrap_or_else(|| "bilinmiyor".into())
}

fn git_calisma_agaci_kirli() -> Result<bool, String> {
    let durum = komutun_tek_satiri(
        "git",
        &["status", "--porcelain=v1", "--untracked-files=normal"],
    )
    .ok_or_else(|| "Git çalışma ağacı durumu okunamadı".to_string())?;
    Ok(!durum.is_empty())
}

fn tarihce_provenance_denetle(git_dirty: bool) -> Result<(), String> {
    if git_dirty {
        return Err("tarihçe kirli çalışma ağacından üretilemez".into());
    }
    Ok(())
}

fn satirli_kaynak(satir_sayisi: usize) -> String {
    let mut kaynak = String::new();
    for sira in 0..satir_sayisi / 4 {
        kaynak.push_str(&format!(
            "değer{sira} {sira} olsun\nkatı{sira} değer{sira} ile 3 ün çarpımı olsun\nkatı{sira} 0 dan büyükse\n    toplam{sira} katı{sira} ile 1 in toplamı olsun\n"
        ));
    }
    kaynak
}

fn lsp_degisim_olcumu(satir_sayisi: usize, tur: usize) -> Result<Olcum, String> {
    let kaynak = satirli_kaynak(satir_sayisi);
    let ac = ac_mesaji(&kaynak);
    let degistir = degistir_mesaji(&kaynak);
    let kimlik = match satir_sayisi {
        2_000 => "lsp_degistir_2k",
        5_000 => "lsp_degistir_5k",
        10_000 => "lsp_degistir_10k",
        20_000 => "lsp_degistir_20k",
        _ => return Err(format!("desteklenmeyen LSP ölçek boyutu: {satir_sayisi}")),
    };
    let aciklama = match satir_sayisi {
        2_000 => "açık 2000 satır belgede tam metin didChange",
        5_000 => "açık 5000 satır belgede tam metin didChange",
        10_000 => "açık 10000 satır belgede tam metin didChange",
        20_000 => "açık 20000 satır belgede tam metin didChange",
        _ => return Err(format!("desteklenmeyen LSP ölçek boyutu: {satir_sayisi}")),
    };
    zaman_olcumu(
        kimlik,
        aciklama,
        ornekle(
            tur,
            || {
                let mut sunucu = Sunucu::yeni();
                black_box(sunucu.mesaj_isle(initialize_mesaji()));
                black_box(sunucu.mesaj_isle(&ac));
                Ok(sunucu)
            },
            |mut sunucu| {
                let cikti = sunucu.mesaj_isle(black_box(&degistir));
                if cikti.govdeler.is_empty() {
                    return Err("LSP didChange tanı bildirimi üretmedi".into());
                }
                black_box(cikti);
                Ok(())
            },
        )?,
    )
}

fn yurutme_kaynagi() -> &'static str {
    "toplam 0 olsun\n100000 kez tekrarla\n    toplamı 1 artır\ntoplam yaz\n"
}

fn ham_program(kaynak: &str) -> Result<Program, String> {
    let ast = KaynakMetni::yeni(kaynak)
        .sozcukle()
        .and_then(|tokenlar| tokenlar.ayristir(Vec::new()))
        .map_err(|tani| format!("ölçüm AST'si kurulamadı: {tani}"))?;
    Ok(Program {
        cumleler: ast.into_cumleler(),
        islemler: HashMap::new(),
        yapilar: Vec::new(),
        testler: Vec::new(),
    })
}

fn ac_mesaji(kaynak: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": "file:///tmp/zee-performans.dil",
                "languageId": "dil",
                "version": 1,
                "text": kaynak
            }
        }
    })
    .to_string()
}

fn degistir_mesaji(kaynak: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didChange",
        "params": {
            "textDocument": {
                "uri": "file:///tmp/zee-performans.dil",
                "version": 2
            },
            "contentChanges": [{"text": kaynak}]
        }
    })
    .to_string()
}

fn initialize_mesaji() -> &'static str {
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#
}

fn lsp_cercevesini_oku(girdi: &mut impl Read) -> Result<Vec<u8>, String> {
    let mut baslik = Vec::new();
    let mut son_dort = [0u8; 4];
    while &son_dort != b"\r\n\r\n" {
        if baslik.len() >= 8 * 1024 {
            return Err("LSP cold-start yanıt başlığı 8 KiB sınırını aştı".into());
        }
        let mut bayt = [0u8; 1];
        girdi
            .read_exact(&mut bayt)
            .map_err(|hata| format!("LSP cold-start yanıt başlığı okunamadı: {hata}"))?;
        baslik.push(bayt[0]);
        son_dort.rotate_left(1);
        son_dort[3] = bayt[0];
    }
    let baslik = std::str::from_utf8(&baslik)
        .map_err(|_| "LSP cold-start yanıt başlığı UTF-8 değil".to_string())?;
    let uzunluk = baslik
        .split("\r\n")
        .find_map(|satir| {
            let (ad, deger) = satir.split_once(':')?;
            ad.eq_ignore_ascii_case("Content-Length")
                .then(|| deger.trim().parse::<usize>().ok())
                .flatten()
        })
        .ok_or_else(|| "LSP cold-start yanıtında Content-Length yok".to_string())?;
    if uzunluk > 8 * 1024 * 1024 {
        return Err("LSP cold-start yanıt gövdesi 8 MiB sınırını aştı".into());
    }
    let mut govde = vec![0u8; uzunluk];
    girdi
        .read_exact(&mut govde)
        .map_err(|hata| format!("LSP cold-start yanıt gövdesi okunamadı: {hata}"))?;
    Ok(govde)
}

fn lsp_process_cold_start_ornegi(dillsp: &Path) -> Result<u64, String> {
    let baslangic = Instant::now();
    let mut cocuk = Command::new(dillsp)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|hata| format!("{} başlatılamadı: {hata}", dillsp.display()))?;
    let sonuc = (|| {
        let mesaj = initialize_mesaji().as_bytes();
        let mut stdin = cocuk
            .stdin
            .take()
            .ok_or_else(|| "dillsp stdin açılamadı".to_string())?;
        write!(stdin, "Content-Length: {}\r\n\r\n", mesaj.len())
            .and_then(|_| stdin.write_all(mesaj))
            .and_then(|_| stdin.flush())
            .map_err(|hata| format!("dillsp initialize yazılamadı: {hata}"))?;
        drop(stdin);
        let mut stdout = cocuk
            .stdout
            .take()
            .ok_or_else(|| "dillsp stdout açılamadı".to_string())?;
        let govde = lsp_cercevesini_oku(&mut stdout)?;
        let gecen = baslangic.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        let metin = std::str::from_utf8(&govde)
            .map_err(|_| "dillsp initialize yanıtı UTF-8 değil".to_string())?;
        if !metin.contains("\"id\":1") || !metin.contains("\"capabilities\"") {
            return Err(format!(
                "dillsp initialize capabilities yanıtı vermedi: {metin}"
            ));
        }
        Ok(gecen)
    })();
    let _ = cocuk.kill();
    let _ = cocuk.wait();
    sonuc
}

fn lsp_process_cold_start_ornekleri(dillsp: &Path, tur: usize) -> Result<Vec<u64>, String> {
    let mut ornekler = Vec::with_capacity(tur);
    for sira in 0..(ISINMA_TURU + tur) {
        let nanosaniye = lsp_process_cold_start_ornegi(dillsp)?;
        if sira >= ISINMA_TURU {
            ornekler.push(nanosaniye);
        }
    }
    Ok(ornekler)
}

fn ornekle<S, H, C>(tur: usize, mut hazirla: H, mut calistir: C) -> Result<Vec<u64>, String>
where
    H: FnMut() -> Result<S, String>,
    C: FnMut(S) -> Result<(), String>,
{
    let mut ornekler = Vec::with_capacity(tur);
    for sira in 0..(ISINMA_TURU + tur) {
        let durum = hazirla()?;
        let baslangic = Instant::now();
        calistir(black_box(durum))?;
        let nanosaniye = baslangic.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        if sira >= ISINMA_TURU {
            ornekler.push(nanosaniye);
        }
    }
    Ok(ornekler)
}

fn yuzdelik(sirali: &[u64], yuzde: usize) -> u64 {
    let sira = (sirali.len() * yuzde).div_ceil(100).saturating_sub(1);
    sirali[sira.min(sirali.len() - 1)]
}

fn olcum(
    kimlik: &'static str,
    aciklama: &'static str,
    birim: &'static str,
    ornekleme: &'static str,
    isinma_turu: usize,
    ornekler: Vec<u64>,
) -> Result<Olcum, String> {
    if ornekler.is_empty() {
        return Err(format!("{kimlik}: ölçüm örneği yok"));
    }
    let mut sirali = ornekler.clone();
    sirali.sort_unstable();
    let ornek_sayisi = ornekler.len();
    Ok(Olcum {
        kimlik,
        aciklama,
        birim,
        ornekleme,
        ornek_sayisi,
        isinma_turu,
        en_az: sirali[0],
        p50: yuzdelik(&sirali, 50),
        p95: yuzdelik(&sirali, 95),
        en_cok: sirali[sirali.len() - 1],
        ornekler,
    })
}

fn zaman_olcumu(
    kimlik: &'static str,
    aciklama: &'static str,
    ornekler: Vec<u64>,
) -> Result<Olcum, String> {
    olcum(
        kimlik,
        aciklama,
        "ns",
        "bagimsiz_tur",
        ISINMA_TURU,
        ornekler,
    )
}

#[cfg(unix)]
fn tepe_bellek_kib() -> Option<u64> {
    let mut kullanim = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: `getrusage` başarıda verilen geçerli, yazılabilir `rusage`
    // alanını bütünüyle başlatır; pointer çağrı sonuna kadar canlıdır.
    let sonuc = unsafe { libc::getrusage(libc::RUSAGE_SELF, kullanim.as_mut_ptr()) };
    if sonuc != 0 {
        return None;
    }
    // SAFETY: sıfır dönüş, `getrusage` sözleşmesine göre yapıyı başlatmıştır.
    let rss = unsafe { kullanim.assume_init() }.ru_maxrss;
    let rss = u64::try_from(rss).ok()?;
    #[cfg(target_os = "macos")]
    return Some(rss / 1024);
    #[cfg(not(target_os = "macos"))]
    Some(rss)
}

#[cfg(not(unix))]
fn tepe_bellek_kib() -> Option<u64> {
    None
}

fn olcumleri_al(tur: usize, dillsp: &Path, lsp_olcek: bool) -> Result<Vec<Olcum>, String> {
    let kaynak = satirli_kaynak(2_000);
    let ham = ham_program(&kaynak)?;
    let bos_program = dil::kaynagi_fazli_derle("")
        .map_err(|tani| format!("boş runtime programı derlenemedi: {tani}"))?;
    let yurutme_programi = dil::kaynagi_fazli_derle(yurutme_kaynagi())
        .map_err(|tani| format!("runtime iş yükü derlenemedi: {tani}"))?;
    let ac = ac_mesaji(&kaynak);
    let mut sonuc = Vec::new();

    sonuc.push(zaman_olcumu(
        "parse_gecikmesi",
        "2000 satırda lexer + parser",
        ornekle(
            tur,
            || Ok(()),
            |_| {
                let ast = KaynakMetni::yeni(black_box(&kaynak))
                    .sozcukle()
                    .and_then(|tokenlar| tokenlar.ayristir(Vec::new()))
                    .map_err(|tani| format!("parse ölçümü başarısız: {tani}"))?;
                black_box(ast);
                Ok(())
            },
        )?,
    )?);

    sonuc.push(zaman_olcumu(
        "typecheck_gecikmesi",
        "hazır AST'de resolver + checker ve HIR kanıt toplama",
        ornekle(
            tur,
            || Ok(ham.clone()),
            |mut program| {
                dil::cozumleyici::denetle(&mut program)
                    .map_err(|tani| format!("typecheck ölçümü başarısız: {tani}"))?;
                black_box(program);
                Ok(())
            },
        )?,
    )?);

    sonuc.push(zaman_olcumu(
        "hir_olusturma",
        "2000 satır kaynak -> bağlı typed HIR tam ön uç",
        ornekle(
            tur,
            || Ok(()),
            |_| {
                let program = dil::kaynagi_fazli_derle(black_box(&kaynak))
                    .map_err(|tani| format!("HIR ölçümü başarısız: {tani}"))?;
                black_box(program);
                Ok(())
            },
        )?,
    )?);

    sonuc.push(zaman_olcumu(
        "runtime_baslangici",
        "önceden derlenmiş boş typed HIR runtime dispatch",
        ornekle(
            tur,
            || Ok(()),
            |_| {
                let cikti = dil::yorumlayici::calistir_baglanmis(&bos_program)
                    .map_err(|tani| format!("runtime başlangıç ölçümü başarısız: {tani}"))?;
                black_box(cikti);
                Ok(())
            },
        )?,
    )?);

    sonuc.push(zaman_olcumu(
        "yurutme_gecikmesi",
        "önceden derlenmiş 100 bin turluk sayaç",
        ornekle(
            tur,
            || Ok(()),
            |_| {
                let cikti = dil::yorumlayici::calistir_baglanmis(&yurutme_programi)
                    .map_err(|tani| format!("yürütme ölçümü başarısız: {tani}"))?;
                if cikti.last().map(String::as_str) != Some("100000") {
                    return Err(format!("yürütme iş yükü yanlış çıktı verdi: {cikti:?}"));
                }
                black_box(cikti);
                Ok(())
            },
        )?,
    )?);

    sonuc.push(zaman_olcumu(
        "lsp_engine_initialize",
        "in-process LSP engine kurma + initialize isteği",
        ornekle(
            tur,
            || Ok(()),
            |_| {
                let mut sunucu = Sunucu::yeni();
                let cikti = sunucu.mesaj_isle(initialize_mesaji());
                if cikti.govdeler.len() != 1 {
                    return Err("LSP initialize yanıt üretmedi".into());
                }
                black_box(cikti);
                Ok(())
            },
        )?,
    )?);

    sonuc.push(zaman_olcumu(
        "lsp_process_cold_start",
        "dillsp process spawn + stdio initialize capabilities yanıtı",
        lsp_process_cold_start_ornekleri(dillsp, tur)?,
    )?);

    sonuc.push(zaman_olcumu(
        "lsp_ac",
        "initialize edilmiş sunucuda 2000 satır didOpen",
        ornekle(
            tur,
            || {
                let mut sunucu = Sunucu::yeni();
                black_box(sunucu.mesaj_isle(initialize_mesaji()));
                Ok(sunucu)
            },
            |mut sunucu| {
                let cikti = sunucu.mesaj_isle(black_box(&ac));
                if cikti.govdeler.is_empty() {
                    return Err("LSP didOpen tanı bildirimi üretmedi".into());
                }
                black_box(cikti);
                Ok(())
            },
        )?,
    )?);

    let olcekler = if lsp_olcek {
        LSP_OLCEKLERI.as_slice()
    } else {
        &LSP_OLCEKLERI[..1]
    };
    for satir_sayisi in olcekler {
        sonuc.push(lsp_degisim_olcumu(*satir_sayisi, tur)?);
    }

    if let Some(kib) = tepe_bellek_kib() {
        sonuc.push(olcum(
            "tepe_bellek",
            "koşucu süreç tepe resident set'i",
            "KiB",
            "surec_tepe_anlik_goruntusu",
            0,
            vec![kib],
        )?);
    }
    Ok(sonuc)
}

fn gecmisi_oku(yol: Option<&Path>) -> Result<(String, Vec<GecmisSatiri>), String> {
    let Some(yol) = yol else {
        return Ok((format!(
            "{GECMIS_SEMASI}\n# kayit\tgit_sha\tmilestone\tgit_dirty\tplatform\tos\tcpu\tram_bytes\trustc\tbuild_profile\tsample_count\twarmup_count\tsampling_semantics\tolcum\tbirim\tp50\tp95\n"
        ), Vec::new()));
    };
    let metin = fs::read_to_string(yol)
        .map_err(|hata| format!("{} geçmişi okunamadı: {hata}", yol.display()))?;
    let satirlar = gecmis_metinini_ayristir(&metin, yol)?;
    Ok((metin, satirlar))
}

fn gecmis_metinini_ayristir(metin: &str, yol: &Path) -> Result<Vec<GecmisSatiri>, String> {
    if metin.lines().next() != Some(GECMIS_SEMASI) {
        return Err(format!(
            "{} bilinmeyen geçmiş şeması taşıyor",
            yol.display()
        ));
    }
    if !metin.ends_with('\n') || metin.contains('\r') {
        return Err(format!("{} kanonik LF metni değil", yol.display()));
    }
    let mut satirlar = Vec::new();
    let mut gorulenler = BTreeSet::new();
    let mut kayit_metadatasi = BTreeMap::new();
    for (sira, satir) in metin.lines().enumerate() {
        if satir.is_empty() || satir.starts_with('#') {
            continue;
        }
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let [kayit, git_sha, milestone, git_dirty, platform, os, cpu, ram_bayt, rustc, profil, ornek_sayisi, isinma_sayisi, ornekleme, kimlik, birim, p50, p95] =
            alanlar.as_slice()
        else {
            return Err(format!("geçmiş satırı {} on yedi alan taşımalı", sira + 1));
        };
        if [
            kayit, git_sha, milestone, git_dirty, platform, os, cpu, rustc, profil, ornekleme,
            kimlik, birim,
        ]
        .iter()
        .any(|alan| alan.is_empty())
        {
            return Err(format!("geçmiş satırı {} boş alan taşıyor", sira + 1));
        }
        if !gecerli_git_sha(git_sha) {
            return Err(format!(
                "geçmiş satırı {} gerçek tam Git SHA taşımıyor",
                sira + 1
            ));
        }
        if *git_dirty != "false" {
            return Err(format!(
                "geçmiş satırı {} temiz Git ağacı taşımıyor",
                sira + 1
            ));
        }
        if *profil != "release" {
            return Err(format!(
                "geçmiş satırı {} release profili taşımıyor",
                sira + 1
            ));
        }
        if !matches!(*birim, "ns" | "KiB") {
            return Err(format!(
                "geçmiş satırı {} bilinmeyen birim taşıyor: {birim}",
                sira + 1
            ));
        }
        if !gorulenler.insert(((*kayit).to_string(), (*kimlik).to_string())) {
            return Err(format!(
                "geçmiş satırı {} yinelenen kayıt/ölçüm taşıyor: {kayit}/{kimlik}",
                sira + 1
            ));
        }
        let metadata = (
            (*git_sha).to_string(),
            (*milestone).to_string(),
            (*git_dirty).to_string(),
            (*platform).to_string(),
            (*os).to_string(),
            (*cpu).to_string(),
            (*ram_bayt).to_string(),
            (*rustc).to_string(),
            (*profil).to_string(),
        );
        if let Some(onceki) = kayit_metadatasi.insert((*kayit).to_string(), metadata.clone()) {
            if onceki != metadata {
                return Err(format!(
                    "geçmiş satırı {} aynı kayıt için farklı metadata taşıyor: {kayit}",
                    sira + 1
                ));
            }
        }
        let ram_bayt = ram_bayt
            .parse()
            .map_err(|_| format!("geçmiş satırı {} RAM değeri bozuk", sira + 1))?;
        if ram_bayt == 0 {
            return Err(format!("geçmiş satırı {} sıfır RAM taşıyor", sira + 1));
        }
        let ornek_sayisi = ornek_sayisi
            .parse()
            .map_err(|_| format!("geçmiş satırı {} örnek sayısı bozuk", sira + 1))?;
        if ornek_sayisi == 0 {
            return Err(format!("geçmiş satırı {} sıfır örnek taşıyor", sira + 1));
        }
        let isinma_sayisi = isinma_sayisi
            .parse()
            .map_err(|_| format!("geçmiş satırı {} ısınma sayısı bozuk", sira + 1))?;
        let p50 = p50
            .parse()
            .map_err(|_| format!("geçmiş satırı {} p50 bozuk", sira + 1))?;
        let p95 = p95
            .parse()
            .map_err(|_| format!("geçmiş satırı {} p95 bozuk", sira + 1))?;
        if p50 > p95 {
            return Err(format!(
                "geçmiş satırı {} p50 değeri p95'ten büyük",
                sira + 1
            ));
        }
        match *ornekleme {
            "bagimsiz_tur" if isinma_sayisi > 0 => {}
            "surec_tepe_anlik_goruntusu"
                if *kimlik == "tepe_bellek"
                    && *birim == "KiB"
                    && ornek_sayisi == 1
                    && isinma_sayisi == 0
                    && p50 == p95 => {}
            _ => {
                return Err(format!(
                    "geçmiş satırı {} örnekleme semantiğiyle tutarsız",
                    sira + 1
                ));
            }
        }
        satirlar.push(GecmisSatiri {
            kayit: (*kayit).into(),
            git_sha: (*git_sha).into(),
            milestone: (*milestone).into(),
            git_dirty: false,
            platform: (*platform).into(),
            os: (*os).into(),
            cpu: (*cpu).into(),
            ram_bayt,
            rustc: (*rustc).into(),
            profil: (*profil).into(),
            ornek_sayisi,
            isinma_sayisi,
            ornekleme: (*ornekleme).into(),
            kimlik: (*kimlik).into(),
            birim: (*birim).into(),
            p50,
            p95,
        });
    }
    Ok(satirlar)
}

fn son_eslesenler<'a>(
    gecmis: &'a [GecmisSatiri],
    rapor: &Rapor<'_>,
) -> BTreeMap<&'a str, &'a GecmisSatiri> {
    let mut son = BTreeMap::new();
    for satir in gecmis.iter().filter(|satir| {
        satir.platform == rapor.platform
            && satir.os == rapor.os
            && satir.cpu == rapor.cpu
            && satir.ram_bayt == rapor.ram_bayt
            && satir.rustc == rapor.rustc
            && satir.profil == rapor.profil
    }) {
        son.insert(satir.kimlik.as_str(), satir);
    }
    son
}

fn sureyi_yaz(deger: u64, birim: &str) -> String {
    if birim == "KiB" {
        return format!("{deger} KiB");
    }
    if deger >= 1_000_000 {
        format!("{:.3} ms", deger as f64 / 1_000_000.0)
    } else if deger >= 1_000 {
        format!("{:.3} µs", deger as f64 / 1_000.0)
    } else {
        format!("{deger} ns")
    }
}

fn yuzde_farki(eski: u64, yeni: u64) -> String {
    if eski == 0 {
        return "—".into();
    }
    let fark = (yeni as f64 - eski as f64) * 100.0 / eski as f64;
    format!("{fark:+.1}%")
}

fn lsp_olcek_markdowni(olcumler: &[Olcum]) -> String {
    let olcek = LSP_OLCEKLERI
        .iter()
        .filter_map(|satir| {
            let kimlik = format!("lsp_degistir_{}k", satir / 1_000);
            olcumler
                .iter()
                .find(|olcum| olcum.kimlik == kimlik)
                .map(|olcum| (*satir, olcum))
        })
        .collect::<Vec<_>>();
    if olcek.len() != LSP_OLCEKLERI.len() {
        return String::new();
    }

    let mut metin = String::from(
        "\n## LSP tam-metin değişiklik ölçeği\n\n\
         Bu gözlem bir optimizasyon ya da hard CI kapısı değildir. Eşikler ölçümden önce \
         250/500/1000 ms olarak sabitlenmiştir ve karar p95 üzerinden verilir.\n\n\
         | Satır | p50 | p95 |\n|---:|---:|---:|\n",
    );
    for (satir, olcum) in &olcek {
        metin.push_str(&format!(
            "| {} | {} | {} |\n",
            satir,
            sureyi_yaz(olcum.p50, olcum.birim),
            sureyi_yaz(olcum.p95, olcum.birim)
        ));
    }
    metin.push_str("\n| p95 eşiği | İlk aşım |\n|---:|---|\n");
    for esik_ms in LSP_ESIKLERI_MS {
        let ilk_asim = olcek
            .iter()
            .find(|(_, olcum)| olcum.p95 > esik_ms * 1_000_000)
            .map(|(satir, _)| format!("{} satır", satir))
            .unwrap_or_else(|| "20.000 satıra kadar aşılmadı".into());
        metin.push_str(&format!("| {esik_ms} ms | {ilk_asim} |\n"));
    }
    metin
}

fn markdown_raporu(rapor: &Rapor<'_>, gecmis: &[GecmisSatiri], ihlaller: &[String]) -> String {
    let onceki = son_eslesenler(gecmis, rapor);
    let taban = onceki.values().next().map_or_else(
        || "yok; bu platform için ilk gözlem".to_string(),
        |satir| {
            format!(
                "`{}` / `{}` / `{}`; Git {}; OS `{}`; CPU `{}`; RAM `{}` bayt; Rust `{}`; profil `{}`; {} örnek + {} ısınma; `{}`",
                satir.kayit,
                satir.git_sha,
                satir.milestone,
                if satir.git_dirty { "kirli" } else { "temiz" },
                satir.os,
                satir.cpu,
                satir.ram_bayt,
                satir.rustc,
                satir.profil,
                satir.ornek_sayisi,
                satir.isinma_sayisi,
                satir.ornekleme
            )
        },
    );
    let mut metin = format!(
        "# Zee performans gözlemi\n\n\
         - Şema: `{}`\n\
         - Kayıt: `{}`\n\
         - Git SHA: `{}`\n\
         - Milestone: `{}`\n\
         - Git çalışma ağacı: **{}**\n\
         - Platform: `{}`\n\
         - OS: `{}`\n\
         - CPU: `{}`\n\
         - RAM: **{} bayt**\n\
         - Rust: `{}`\n\
         - Profil: **{}**, ölçüm turu: **{}**, ısınma: **{}**\n\
         - Exact ortam karşılaştırma tabanı: {}\n\
         - Politika: {}\n\n\
         | Ölçüm | p50 | p95 | En az | En çok | Önceki p50 | Önceki p95 | Eğilim |\n\
         |---|---:|---:|---:|---:|---:|---:|---:|\n",
        rapor.sema,
        rapor.kayit,
        rapor.git_sha,
        rapor.milestone,
        if rapor.git_dirty { "kirli" } else { "temiz" },
        rapor.platform,
        rapor.os,
        rapor.cpu,
        rapor.ram_bayt,
        rapor.rustc,
        rapor.profil,
        rapor.tur,
        rapor.isinma_turu,
        taban,
        if rapor.esik_uygulandi {
            "adanmış koşucu eşiği etkin"
        } else {
            "yalnız gözlem; shared CI hard gate değildir"
        }
    );
    for olcum in rapor.olcumler {
        let eski = onceki
            .get(olcum.kimlik)
            .filter(|satir| satir.birim == olcum.birim);
        let eski_p95 = eski
            .map(|satir| sureyi_yaz(satir.p95, &satir.birim))
            .unwrap_or_else(|| "—".into());
        let eski_p50 = eski
            .map(|satir| sureyi_yaz(satir.p50, &satir.birim))
            .unwrap_or_else(|| "—".into());
        let egilim = eski
            .map(|satir| yuzde_farki(satir.p95, olcum.p95))
            .unwrap_or_else(|| "—".into());
        metin.push_str(&format!(
            "| `{}` — {} | {} | {} | {} | {} | {} | {} | {} |\n",
            olcum.kimlik,
            olcum.aciklama,
            sureyi_yaz(olcum.p50, olcum.birim),
            sureyi_yaz(olcum.p95, olcum.birim),
            sureyi_yaz(olcum.en_az, olcum.birim),
            sureyi_yaz(olcum.en_cok, olcum.birim),
            eski_p50,
            eski_p95,
            egilim
        ));
    }
    metin.push_str(&lsp_olcek_markdowni(rapor.olcumler));
    if !ihlaller.is_empty() {
        metin.push_str("\n## Eşik ihlalleri\n\n");
        for ihlal in ihlaller {
            metin.push_str(&format!("- {ihlal}\n"));
        }
    }
    metin.push_str(
        "\nShared CI sayıları makine seçimi ve komşu iş yüklerinden etkilenir; \
         doğruluk başarısı veya sürüm engeli değildir. Eşik yalnız aynı exact ortamda \
         sabitlenmiş adanmış benchmark koşucusunda açıkça istenir.\n",
    );
    metin
}

fn gecmis_satirlarini_yaz(onceki_metin: &str, rapor: &Rapor<'_>) -> Result<String, String> {
    for deger in [
        rapor.kayit,
        rapor.git_sha,
        rapor.milestone,
        rapor.platform,
        rapor.os,
        rapor.cpu,
        rapor.rustc,
    ] {
        if deger.contains(['\t', '\r', '\n']) {
            return Err("geçmiş metadata'sı sekme veya satır sonu taşıyamaz".into());
        }
    }
    if !gecerli_git_sha(rapor.git_sha) {
        return Err("geçmiş metadata'sı tam 40 haneli Git SHA ister".into());
    }
    if rapor.git_dirty || rapor.ram_bayt == 0 || rapor.profil != "release" {
        return Err("geçmiş metadata'sı temiz Git ağacı, RAM ve release profili ister".into());
    }
    let mut metin = onceki_metin.to_string();
    for olcum in rapor.olcumler {
        metin.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            rapor.kayit,
            rapor.git_sha,
            rapor.milestone,
            rapor.git_dirty,
            rapor.platform,
            rapor.os,
            rapor.cpu,
            rapor.ram_bayt,
            rapor.rustc,
            rapor.profil,
            olcum.ornek_sayisi,
            olcum.isinma_turu,
            olcum.ornekleme,
            olcum.kimlik,
            olcum.birim,
            olcum.p50,
            olcum.p95
        ));
    }
    Ok(metin)
}

fn esik_ihlalleri(
    rapor: &Rapor<'_>,
    gecmis: &[GecmisSatiri],
    esik_yuzde: Option<u64>,
) -> Result<Vec<String>, String> {
    let Some(esik) = esik_yuzde else {
        return Ok(Vec::new());
    };
    let onceki = son_eslesenler(gecmis, rapor);
    let mut ihlaller = Vec::new();
    let mut karsilastirilan = 0usize;
    for olcum in rapor.olcumler {
        let Some(eski) = onceki
            .get(olcum.kimlik)
            .filter(|satir| satir.birim == olcum.birim)
        else {
            continue;
        };
        karsilastirilan += 1;
        let izinli = (eski.p95 as u128) * (100 + esik as u128) / 100;
        if olcum.p95 as u128 > izinli {
            ihlaller.push(format!(
                "`{}` p95 {} → {} ({})",
                olcum.kimlik,
                sureyi_yaz(eski.p95, &eski.birim),
                sureyi_yaz(olcum.p95, olcum.birim),
                yuzde_farki(eski.p95, olcum.p95)
            ));
        }
    }
    if karsilastirilan == 0 {
        return Err(format!(
            "--esik-yuzde için {} exact ortam geçmişinde karşılaştırılabilir ölçüm yok",
            rapor.platform
        ));
    }
    Ok(ihlaller)
}

fn dosyaya_yaz(yol: &Path, metin: &str) -> Result<(), String> {
    if let Some(ebeveyn) = yol.parent() {
        fs::create_dir_all(ebeveyn)
            .map_err(|hata| format!("{} oluşturulamadı: {hata}", ebeveyn.display()))?;
    }
    fs::write(yol, metin).map_err(|hata| format!("{} yazılamadı: {hata}", yol.display()))
}

fn olcumleri_calistir() -> Result<(), String> {
    let ayarlar = komut_satirini_oku()?;
    let head = git_revizyonu()?;
    if ayarlar.git_sha != head {
        return Err(format!(
            "--git-sha ölçülen HEAD ile aynı olmalı: {} != {head}",
            ayarlar.git_sha
        ));
    }
    let git_dirty = git_calisma_agaci_kirli()?;
    let platform = platform();
    let os = os_surumu();
    let cpu = cpu_adi();
    let ram_bayt = toplam_ram_bayt().ok_or_else(|| "toplam RAM belirlenemedi".to_string())?;
    let rustc = rustc_surumu();
    let profil = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    if profil != "release" && (ayarlar.json.is_some() || ayarlar.gecmis_cikti.is_some()) {
        return Err("arşiv/artefakt yalnız --release profiliyle üretilebilir".into());
    }
    if ayarlar.milestone.is_empty()
        || [os.as_str(), cpu.as_str(), rustc.as_str()].contains(&"bilinmiyor")
    {
        return Err("ölçüm milestone, OS, CPU ve Rust provenance alanlarını ister".into());
    }
    if ayarlar.gecmis_cikti.is_some() {
        tarihce_provenance_denetle(git_dirty)?;
    }

    let olcumler = olcumleri_al(ayarlar.tur, &ayarlar.dillsp, ayarlar.lsp_olcek)?;
    let (gecmis_metni, gecmis) = gecmisi_oku(ayarlar.gecmis.as_deref())?;
    let rapor = Rapor {
        sema: SEMA,
        kayit: &ayarlar.kayit,
        git_sha: &ayarlar.git_sha,
        milestone: &ayarlar.milestone,
        git_dirty,
        unix_zamani: unix_zamani()?,
        platform: &platform,
        os: &os,
        cpu: &cpu,
        ram_bayt,
        rustc: &rustc,
        profil,
        tur: ayarlar.tur,
        isinma_turu: ISINMA_TURU,
        esik_uygulandi: ayarlar.esik_yuzde.is_some(),
        olcumler: &olcumler,
    };
    let ihlaller = esik_ihlalleri(&rapor, &gecmis, ayarlar.esik_yuzde)?;
    let markdown = markdown_raporu(&rapor, &gecmis, &ihlaller);
    print!("{markdown}");

    if let Some(yol) = &ayarlar.json {
        let json = serde_json::to_string_pretty(&rapor)
            .map_err(|hata| format!("JSON raporu üretilemedi: {hata}"))?;
        dosyaya_yaz(yol, &format!("{json}\n"))?;
    }
    if let Some(yol) = &ayarlar.rapor {
        dosyaya_yaz(yol, &markdown)?;
    }
    if let Some(yol) = &ayarlar.gecmis_cikti {
        let birlesik = gecmis_satirlarini_yaz(&gecmis_metni, &rapor)?;
        dosyaya_yaz(yol, &birlesik)?;
    }
    if !ihlaller.is_empty() {
        return Err(format!("{} performans eşiği ihlali", ihlaller.len()));
    }
    Ok(())
}

fn main() -> ExitCode {
    match olcumleri_calistir() {
        Ok(()) => ExitCode::SUCCESS,
        Err(hata) => {
            eprintln!("Ölçüm hatası: {hata}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    const SHA: &str = "df737f643c4ee9c8525ce7e972660230e75f5f45";

    fn gecmis_satiri(kayit: &str, p95: u64) -> GecmisSatiri {
        GecmisSatiri {
            kayit: kayit.into(),
            git_sha: SHA.into(),
            milestone: "K-148".into(),
            git_dirty: false,
            platform: "linux-x86_64".into(),
            os: "Linux test".into(),
            cpu: "cpu".into(),
            ram_bayt: 1024,
            rustc: "rust".into(),
            profil: "release".into(),
            ornek_sayisi: 20,
            isinma_sayisi: 2,
            ornekleme: "bagimsiz_tur".into(),
            kimlik: "parse".into(),
            birim: "ns".into(),
            p50: 10,
            p95,
        }
    }

    fn test_raporu(olcumler: &[Olcum]) -> Rapor<'_> {
        Rapor {
            sema: SEMA,
            kayit: "test",
            git_sha: SHA,
            milestone: "K-148",
            git_dirty: false,
            unix_zamani: 0,
            platform: "linux-x86_64",
            os: "Linux test",
            cpu: "cpu",
            ram_bayt: 1024,
            rustc: "rust",
            profil: "release",
            tur: 20,
            isinma_turu: 2,
            esik_uygulandi: false,
            olcumler,
        }
    }

    #[test]
    fn yuzdelikler_nearest_rank_ile_kararlidir() {
        let sirali = (1..=20).collect::<Vec<_>>();
        assert_eq!(yuzdelik(&sirali, 50), 10);
        assert_eq!(yuzdelik(&sirali, 95), 19);
    }

    #[test]
    fn gecmis_son_ayni_platform_kaydini_secer() {
        let gecmis = vec![gecmis_satiri("bir", 20), gecmis_satiri("iki", 21)];
        let olcumler = Vec::new();
        assert_eq!(
            son_eslesenler(&gecmis, &test_raporu(&olcumler))["parse"].p95,
            21
        );
    }

    #[test]
    fn gecmis_semasi_yinelenen_ve_tutarsiz_satiri_reddeder() {
        let baslik = "# zee-performans-gecmisi-2\n# baslik\n";
        let satir = format!(
            "k\t{SHA}\tK-148\tfalse\tlinux-x86_64\tLinux test\tcpu\t1024\trust\trelease\t20\t2\tbagimsiz_tur\tparse\tns\t10\t20\n"
        );
        let yinelenen = format!("{baslik}{satir}{satir}");
        assert!(gecmis_metinini_ayristir(&yinelenen, Path::new("test.tsv")).is_err());

        let ters = format!(
            "{baslik}k\t{SHA}\tK-148\tfalse\tlinux-x86_64\tLinux test\tcpu\t1024\trust\trelease\t20\t2\tbagimsiz_tur\tparse\tns\t21\t20\n"
        );
        assert!(gecmis_metinini_ayristir(&ters, Path::new("test.tsv")).is_err());

        let sahte_sha = satir.replacen(SHA, "K-148", 1);
        assert!(
            gecmis_metinini_ayristir(&format!("{baslik}{sahte_sha}"), Path::new("test.tsv"))
                .is_err()
        );

        let yanlis_rss = format!(
            "{baslik}rss\t{SHA}\tK-148\tfalse\tlinux-x86_64\tLinux test\tcpu\t1024\trust\trelease\t25\t2\tsurec_tepe_anlik_goruntusu\ttepe_bellek\tKiB\t10\t20\n"
        );
        assert!(gecmis_metinini_ayristir(&yanlis_rss, Path::new("test.tsv")).is_err());
    }

    #[test]
    fn izlenen_k148_k153_ve_k154_gecmisi_exact_kaynaklari_tasir() {
        let yol = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/performans-gecmisi-v2.tsv");
        let (_, satirlar) = gecmisi_oku(Some(&yol)).expect("izlenen geçmiş geçerli olmalı");
        assert_eq!(satirlar.len(), 32);
        assert_eq!(
            satirlar
                .iter()
                .filter(|satir| satir.kayit == "K-148-m4pro" && satir.git_sha == SHA)
                .count(),
            9
        );
        assert_eq!(
            satirlar
                .iter()
                .filter(|satir| {
                    satir.kayit == "K-153-m4pro"
                        && satir.git_sha == "a2693d6ec98d52b8e2e882b43ecaae682fcce48a"
                        && satir.milestone == "K-153"
                })
                .count(),
            10
        );
        let rss = satirlar
            .iter()
            .find(|satir| satir.kimlik == "tepe_bellek")
            .expect("RSS kaydı olmalı");
        assert_eq!(rss.ornekleme, "surec_tepe_anlik_goruntusu");
        assert_eq!((rss.ornek_sayisi, rss.isinma_sayisi), (1, 0));
        let process = satirlar
            .iter()
            .find(|satir| satir.kayit == "K-153-m4pro" && satir.kimlik == "lsp_process_cold_start")
            .expect("K-153 process cold-start kaydı olmalı");
        assert_eq!((process.ornek_sayisi, process.isinma_sayisi), (25, 2));
        assert_eq!((process.p50, process.p95), (1_556_958, 1_996_667));
        let k154 = satirlar
            .iter()
            .filter(|satir| {
                satir.kayit == "K-154-m4pro"
                    && satir.git_sha == "59580cd0c0b2d66a1ff1f28e7e285bfa0858abab"
                    && satir.milestone == "K-154"
            })
            .collect::<Vec<_>>();
        assert_eq!(k154.len(), 13);
        let olcek = |kimlik| {
            k154.iter()
                .find(|satir| satir.kimlik == kimlik)
                .map(|satir| (satir.p50, satir.p95))
        };
        assert_eq!(olcek("lsp_degistir_2k"), Some((160_293_292, 167_280_708)));
        assert_eq!(
            olcek("lsp_degistir_5k"),
            Some((1_062_714_916, 1_081_745_917))
        );
        assert_eq!(
            olcek("lsp_degistir_10k"),
            Some((4_647_041_916, 4_669_371_959))
        );
        assert_eq!(
            olcek("lsp_degistir_20k"),
            Some((20_079_599_917, 20_159_635_833))
        );
    }

    #[test]
    fn shared_ci_esiksizdir_adanmis_kosu_acikca_ihlal_uretir() {
        let olcumler = vec![zaman_olcumu("parse", "parse", vec![20]).expect("ölçüm")];
        let gecmis = vec![gecmis_satiri("taban", 10)];
        let rapor = test_raporu(&olcumler);
        assert!(esik_ihlalleri(&rapor, &gecmis, None)
            .expect("eşiksiz")
            .is_empty());
        assert_eq!(
            esik_ihlalleri(&rapor, &gecmis, Some(50))
                .expect("eşikli")
                .len(),
            1
        );
        let mut baska_os = gecmis;
        baska_os[0].os = "başka Linux".into();
        assert!(esik_ihlalleri(&rapor, &baska_os, Some(50)).is_err());
    }

    #[test]
    fn gecmis_satiri_metadata_satir_sizmasini_reddeder() {
        let olcumler = vec![zaman_olcumu("parse", "parse", vec![10]).expect("ölçüm")];
        let rapor = Rapor {
            sema: SEMA,
            kayit: "bozuk\nkayıt",
            git_sha: SHA,
            milestone: "K-148",
            git_dirty: false,
            unix_zamani: 0,
            platform: "linux-x86_64",
            os: "Linux test",
            cpu: "cpu",
            ram_bayt: 1024,
            rustc: "rust",
            profil: "release",
            tur: 1,
            isinma_turu: 0,
            esik_uygulandi: false,
            olcumler: &olcumler,
        };
        assert!(gecmis_satirlarini_yaz("# zee-performans-gecmisi-2\n# baslik\n", &rapor).is_err());
        assert!(tarihce_provenance_denetle(true).is_err());
        assert!(tarihce_provenance_denetle(false).is_ok());
    }

    #[test]
    fn rapor_p50_p95_ve_shared_ci_politikasini_aciklar() {
        let olcumler = vec![zaman_olcumu("parse", "parse", vec![10, 20]).expect("ölçüm")];
        let rapor = Rapor {
            sema: SEMA,
            kayit: "kayıt",
            git_sha: SHA,
            milestone: "K-148",
            git_dirty: false,
            unix_zamani: 0,
            platform: "linux-x86_64",
            os: "Linux test",
            cpu: "cpu",
            ram_bayt: 1024,
            rustc: "rust",
            profil: "release",
            tur: 2,
            isinma_turu: 0,
            esik_uygulandi: false,
            olcumler: &olcumler,
        };
        let metin = markdown_raporu(&rapor, &[], &[]);
        assert!(metin.contains("| Ölçüm | p50 | p95"));
        assert!(metin.contains("shared CI hard gate değildir"));
        let (baslik, _) = gecmisi_oku(None).expect("boş v2 tarihçe başlığı");
        let yazilan = gecmis_satirlarini_yaz(&baslik, &rapor).expect("v2 satırı yazılmalı");
        let geri = gecmis_metinini_ayristir(&yazilan, Path::new("test.tsv"))
            .expect("üretilen v2 satırı yeniden okunmalı");
        assert_eq!(geri.len(), 1);
        assert_eq!(geri[0].git_sha, SHA);
    }

    #[test]
    fn lsp_olcek_raporu_esikleri_p95_uzerinden_bulur() {
        let p95ler = [200_000_000, 300_000_000, 700_000_000, 1_200_000_000];
        let olcumler = ["2k", "5k", "10k", "20k"]
            .iter()
            .zip(p95ler)
            .map(|(boyut, p95)| Olcum {
                kimlik: match *boyut {
                    "2k" => "lsp_degistir_2k",
                    "5k" => "lsp_degistir_5k",
                    "10k" => "lsp_degistir_10k",
                    _ => "lsp_degistir_20k",
                },
                aciklama: "test",
                birim: "ns",
                ornekleme: "bagimsiz_tur",
                ornek_sayisi: 1,
                isinma_turu: 0,
                ornekler: vec![p95],
                en_az: p95,
                p50: p95,
                p95,
                en_cok: p95,
            })
            .collect::<Vec<_>>();
        let metin = lsp_olcek_markdowni(&olcumler);
        assert!(metin.contains("| 250 ms | 5000 satır |"));
        assert!(metin.contains("| 500 ms | 10000 satır |"));
        assert!(metin.contains("| 1000 ms | 20000 satır |"));
        assert!(lsp_olcek_markdowni(&olcumler[..3]).is_empty());
    }
}
