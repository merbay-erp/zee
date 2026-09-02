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
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const SEMA: &str = "zee-performans-1";
const GECMIS_SEMASI: &str = "# zee-performans-gecmisi-1";
const VARSAYILAN_TUR: usize = 25;
const HIZLI_TUR: usize = 3;
const ISINMA_TURU: usize = 2;

#[derive(Debug)]
struct Ayarlar {
    tur: usize,
    json: Option<PathBuf>,
    rapor: Option<PathBuf>,
    gecmis: Option<PathBuf>,
    gecmis_cikti: Option<PathBuf>,
    kayit: String,
    revizyon: String,
    esik_yuzde: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
struct Olcum {
    kimlik: &'static str,
    aciklama: &'static str,
    birim: &'static str,
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
    revizyon: &'a str,
    unix_zamani: u64,
    platform: &'a str,
    cpu: &'a str,
    rustc: &'a str,
    profil: &'static str,
    tur: usize,
    isinma_turu: usize,
    esik_uygulandi: bool,
    olcumler: &'a [Olcum],
}

#[derive(Clone, Debug)]
struct GecmisSatiri {
    kayit: String,
    revizyon: String,
    platform: String,
    cpu: String,
    rustc: String,
    tur: usize,
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
    let mut revizyon = None;
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
            "--revizyon" => revizyon = Some(deger_iste(&mut argumanlar, "--revizyon")?),
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
                     [--revizyon ID] [--esik-yuzde N]"
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
        return Err("--esik-yuzde aynı platformdan --gecmis ister".into());
    }
    let unix_zamani = unix_zamani()?;
    Ok(Ayarlar {
        tur,
        json,
        rapor,
        gecmis,
        gecmis_cikti,
        kayit: kayit.unwrap_or_else(|| format!("yerel-{unix_zamani}")),
        revizyon: revizyon.unwrap_or_else(git_revizyonu),
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

fn git_revizyonu() -> String {
    komutun_tek_satiri("git", &["rev-parse", "--short=12", "HEAD"])
        .unwrap_or_else(|| "bilinmiyor".into())
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

fn rustc_surumu() -> String {
    komutun_tek_satiri("rustc", &["--version"]).unwrap_or_else(|| "bilinmiyor".into())
}

fn buyuk_kaynak() -> String {
    let mut kaynak = String::new();
    for sira in 0..500 {
        kaynak.push_str(&format!(
            "değer{sira} {sira} olsun\nkatı{sira} değer{sira} ile 3 ün çarpımı olsun\nkatı{sira} 0 dan büyükse\n    toplam{sira} katı{sira} ile 1 in toplamı olsun\n"
        ));
    }
    kaynak
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
    ornekler: Vec<u64>,
) -> Result<Olcum, String> {
    if ornekler.is_empty() {
        return Err(format!("{kimlik}: ölçüm örneği yok"));
    }
    let mut sirali = ornekler.clone();
    sirali.sort_unstable();
    Ok(Olcum {
        kimlik,
        aciklama,
        birim,
        en_az: sirali[0],
        p50: yuzdelik(&sirali, 50),
        p95: yuzdelik(&sirali, 95),
        en_cok: sirali[sirali.len() - 1],
        ornekler,
    })
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

fn olcumleri_al(tur: usize) -> Result<Vec<Olcum>, String> {
    let kaynak = buyuk_kaynak();
    let ham = ham_program(&kaynak)?;
    let bos_program = dil::kaynagi_fazli_derle("")
        .map_err(|tani| format!("boş runtime programı derlenemedi: {tani}"))?;
    let yurutme_programi = dil::kaynagi_fazli_derle(yurutme_kaynagi())
        .map_err(|tani| format!("runtime iş yükü derlenemedi: {tani}"))?;
    let ac = ac_mesaji(&kaynak);
    let degistir = degistir_mesaji(&kaynak);
    let mut sonuc = Vec::new();

    sonuc.push(olcum(
        "parse_gecikmesi",
        "2000 satırda lexer + parser",
        "ns",
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

    sonuc.push(olcum(
        "typecheck_gecikmesi",
        "hazır AST'de resolver + checker ve HIR kanıt toplama",
        "ns",
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

    sonuc.push(olcum(
        "hir_olusturma",
        "2000 satır kaynak -> bağlı typed HIR tam ön uç",
        "ns",
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

    sonuc.push(olcum(
        "runtime_baslangici",
        "önceden derlenmiş boş typed HIR runtime dispatch",
        "ns",
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

    sonuc.push(olcum(
        "yurutme_gecikmesi",
        "önceden derlenmiş 100 bin turluk sayaç",
        "ns",
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

    sonuc.push(olcum(
        "lsp_soguk",
        "sunucu kurma + initialize isteği",
        "ns",
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

    sonuc.push(olcum(
        "lsp_ac",
        "initialize edilmiş sunucuda 2000 satır didOpen",
        "ns",
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

    sonuc.push(olcum(
        "lsp_degistir",
        "açık 2000 satır belgede tam metin didChange",
        "ns",
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
    )?);

    if let Some(kib) = tepe_bellek_kib() {
        sonuc.push(olcum(
            "tepe_bellek",
            "koşucu süreç tepe resident set'i",
            "KiB",
            vec![kib],
        )?);
    }
    Ok(sonuc)
}

fn gecmisi_oku(yol: Option<&Path>) -> Result<(String, Vec<GecmisSatiri>), String> {
    let Some(yol) = yol else {
        return Ok((format!("{GECMIS_SEMASI}\n# kayit\trevizyon\tplatform\tcpu\trustc\ttur\tolcum\tbirim\tp50\tp95\n"), Vec::new()));
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
        let [kayit, revizyon, platform, cpu, rustc, tur, kimlik, birim, p50, p95] =
            alanlar.as_slice()
        else {
            return Err(format!("geçmiş satırı {} on alan taşımalı", sira + 1));
        };
        if [kayit, revizyon, platform, cpu, rustc, kimlik, birim]
            .iter()
            .any(|alan| alan.is_empty())
        {
            return Err(format!("geçmiş satırı {} boş alan taşıyor", sira + 1));
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
            (*revizyon).to_string(),
            (*platform).to_string(),
            (*cpu).to_string(),
            (*rustc).to_string(),
            (*tur).to_string(),
        );
        if let Some(onceki) = kayit_metadatasi.insert((*kayit).to_string(), metadata.clone()) {
            if onceki != metadata {
                return Err(format!(
                    "geçmiş satırı {} aynı kayıt için farklı metadata taşıyor: {kayit}",
                    sira + 1
                ));
            }
        }
        let tur = tur
            .parse()
            .map_err(|_| format!("geçmiş satırı {} tur sayısı bozuk", sira + 1))?;
        if tur == 0 {
            return Err(format!("geçmiş satırı {} sıfır tur taşıyor", sira + 1));
        }
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
        satirlar.push(GecmisSatiri {
            kayit: (*kayit).into(),
            revizyon: (*revizyon).into(),
            platform: (*platform).into(),
            cpu: (*cpu).into(),
            rustc: (*rustc).into(),
            tur,
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
    platform: &str,
) -> BTreeMap<&'a str, &'a GecmisSatiri> {
    let mut son = BTreeMap::new();
    for satir in gecmis.iter().filter(|satir| satir.platform == platform) {
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

fn markdown_raporu(rapor: &Rapor<'_>, gecmis: &[GecmisSatiri], ihlaller: &[String]) -> String {
    let onceki = son_eslesenler(gecmis, rapor.platform);
    let taban = onceki.values().next().map_or_else(
        || "yok; bu platform için ilk gözlem".to_string(),
        |satir| {
            format!(
                "`{}` / `{}`; CPU `{}`; Rust `{}`; {} tur",
                satir.kayit, satir.revizyon, satir.cpu, satir.rustc, satir.tur
            )
        },
    );
    let mut metin = format!(
        "# Zee performans gözlemi\n\n\
         - Şema: `{}`\n\
         - Kayıt: `{}`\n\
         - Revizyon: `{}`\n\
         - Platform: `{}`\n\
         - CPU: `{}`\n\
         - Rust: `{}`\n\
         - Profil: **{}**, ölçüm turu: **{}**, ısınma: **{}**\n\
         - Aynı platform karşılaştırma tabanı: {}\n\
         - Politika: {}\n\n\
         | Ölçüm | p50 | p95 | En az | En çok | Önceki p50 | Önceki p95 | Eğilim |\n\
         |---|---:|---:|---:|---:|---:|---:|---:|\n",
        rapor.sema,
        rapor.kayit,
        rapor.revizyon,
        rapor.platform,
        rapor.cpu,
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
    if !ihlaller.is_empty() {
        metin.push_str("\n## Eşik ihlalleri\n\n");
        for ihlal in ihlaller {
            metin.push_str(&format!("- {ihlal}\n"));
        }
    }
    metin.push_str(
        "\nShared CI sayıları makine seçimi ve komşu iş yüklerinden etkilenir; \
         doğruluk başarısı veya sürüm engeli değildir. Eşik yalnız aynı platformda \
         sabitlenmiş adanmış benchmark koşucusunda açıkça istenir.\n",
    );
    metin
}

fn gecmis_satirlarini_yaz(onceki_metin: &str, rapor: &Rapor<'_>) -> Result<String, String> {
    for deger in [
        rapor.kayit,
        rapor.revizyon,
        rapor.platform,
        rapor.cpu,
        rapor.rustc,
    ] {
        if deger.contains(['\t', '\r', '\n']) {
            return Err("geçmiş metadata'sı sekme veya satır sonu taşıyamaz".into());
        }
    }
    let mut metin = onceki_metin.to_string();
    for olcum in rapor.olcumler {
        metin.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            rapor.kayit,
            rapor.revizyon,
            rapor.platform,
            rapor.cpu,
            rapor.rustc,
            rapor.tur,
            olcum.kimlik,
            olcum.birim,
            olcum.p50,
            olcum.p95
        ));
    }
    Ok(metin)
}

fn esik_ihlalleri(
    olcumler: &[Olcum],
    gecmis: &[GecmisSatiri],
    platform: &str,
    esik_yuzde: Option<u64>,
) -> Result<Vec<String>, String> {
    let Some(esik) = esik_yuzde else {
        return Ok(Vec::new());
    };
    let onceki = son_eslesenler(gecmis, platform);
    let mut ihlaller = Vec::new();
    let mut karsilastirilan = 0usize;
    for olcum in olcumler {
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
            "--esik-yuzde için {platform} geçmişinde karşılaştırılabilir ölçüm yok"
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
    let platform = platform();
    let cpu = cpu_adi();
    let rustc = rustc_surumu();
    let profil = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    if profil != "release" && (ayarlar.json.is_some() || ayarlar.gecmis_cikti.is_some()) {
        return Err("arşiv/artefakt yalnız --release profiliyle üretilebilir".into());
    }

    let olcumler = olcumleri_al(ayarlar.tur)?;
    let (gecmis_metni, gecmis) = gecmisi_oku(ayarlar.gecmis.as_deref())?;
    let ihlaller = esik_ihlalleri(&olcumler, &gecmis, &platform, ayarlar.esik_yuzde)?;
    let rapor = Rapor {
        sema: SEMA,
        kayit: &ayarlar.kayit,
        revizyon: &ayarlar.revizyon,
        unix_zamani: unix_zamani()?,
        platform: &platform,
        cpu: &cpu,
        rustc: &rustc,
        profil,
        tur: ayarlar.tur,
        isinma_turu: ISINMA_TURU,
        esik_uygulandi: ayarlar.esik_yuzde.is_some(),
        olcumler: &olcumler,
    };
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

    fn gecmis_satiri(kayit: &str, p95: u64) -> GecmisSatiri {
        GecmisSatiri {
            kayit: kayit.into(),
            revizyon: "a".into(),
            platform: "linux-x86_64".into(),
            cpu: "cpu".into(),
            rustc: "rust".into(),
            tur: 20,
            kimlik: "parse".into(),
            birim: "ns".into(),
            p50: 10,
            p95,
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
        assert_eq!(son_eslesenler(&gecmis, "linux-x86_64")["parse"].p95, 21);
    }

    #[test]
    fn gecmis_semasi_yinelenen_ve_tutarsiz_satiri_reddeder() {
        let baslik = "# zee-performans-gecmisi-1\n# baslik\n";
        let satir = "k\ta\tlinux-x86_64\tcpu\trust\t20\tparse\tns\t10\t20\n";
        let yinelenen = format!("{baslik}{satir}{satir}");
        assert!(gecmis_metinini_ayristir(&yinelenen, Path::new("test.tsv")).is_err());

        let ters = format!("{baslik}k\ta\tlinux-x86_64\tcpu\trust\t20\tparse\tns\t21\t20\n");
        assert!(gecmis_metinini_ayristir(&ters, Path::new("test.tsv")).is_err());
    }

    #[test]
    fn izlenen_k148_gecmisi_sema_ve_dokuz_olcumu_tasir() {
        let yol = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/performans-gecmisi-v1.tsv");
        let (_, satirlar) = gecmisi_oku(Some(&yol)).expect("izlenen geçmiş geçerli olmalı");
        assert_eq!(satirlar.len(), 9);
        assert!(satirlar.iter().all(|satir| satir.kayit == "K-148-m4pro"));
    }

    #[test]
    fn shared_ci_esiksizdir_adanmis_kosu_acikca_ihlal_uretir() {
        let olcumler = vec![olcum("parse", "parse", "ns", vec![20]).expect("ölçüm")];
        let gecmis = vec![gecmis_satiri("taban", 10)];
        assert!(esik_ihlalleri(&olcumler, &gecmis, "linux-x86_64", None)
            .expect("eşiksiz")
            .is_empty());
        assert_eq!(
            esik_ihlalleri(&olcumler, &gecmis, "linux-x86_64", Some(50))
                .expect("eşikli")
                .len(),
            1
        );
    }

    #[test]
    fn gecmis_satiri_metadata_satir_sizmasini_reddeder() {
        let olcumler = vec![olcum("parse", "parse", "ns", vec![10]).expect("ölçüm")];
        let rapor = Rapor {
            sema: SEMA,
            kayit: "bozuk\nkayıt",
            revizyon: "a",
            unix_zamani: 0,
            platform: "linux-x86_64",
            cpu: "cpu",
            rustc: "rust",
            profil: "release",
            tur: 1,
            isinma_turu: 0,
            esik_uygulandi: false,
            olcumler: &olcumler,
        };
        assert!(gecmis_satirlarini_yaz("# zee-performans-gecmisi-1\n# baslik\n", &rapor).is_err());
    }

    #[test]
    fn rapor_p50_p95_ve_shared_ci_politikasini_aciklar() {
        let olcumler = vec![olcum("parse", "parse", "ns", vec![10, 20]).expect("ölçüm")];
        let rapor = Rapor {
            sema: SEMA,
            kayit: "kayıt",
            revizyon: "a",
            unix_zamani: 0,
            platform: "linux-x86_64",
            cpu: "cpu",
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
    }
}
