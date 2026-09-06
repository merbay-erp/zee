//! K-168/ADR-068 tekrar üretilebilir sürüm artefaktı: SHA256SUMS, SPDX 3.0.1
//! SBOM, in-toto/SLSA v1 provenance ve `zee-surum-imza-v1` Ed25519 imzası.
//! Derleme orkestrasyonu `scripts/surum-artefakti.sh` içindedir; bu ikili
//! yalnız deterministik metin/JSON üretir ve doğrular.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const IMZA_SEMASI: &str = "zee-surum-imza-v1";
const IMZA_ALANI: &[u8] = b"zee-surum-v1\0";
const ANAHTAR_BASLIGI: &str = "zee-ed25519-private-v1";
const BUILD_TYPE: &str = "https://zee-lang.dev/surum/tekrar-uretilebilir/v1";
const BUILDER_ID: &str = "https://zee-lang.dev/surum/surum-artefakti";

fn hex_yaz(baytlar: &[u8]) -> String {
    baytlar.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_coz<const N: usize>(metin: &str, ad: &str) -> Result<[u8; N], String> {
    if metin.len() != N * 2
        || !metin
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(format!("{ad} {} haneli küçük hex olmalı", N * 2));
    }
    let mut sonuc = [0u8; N];
    for (i, parca) in sonuc.iter_mut().enumerate() {
        *parca = u8::from_str_radix(&metin[i * 2..i * 2 + 2], 16)
            .map_err(|_| format!("{ad} hex değil"))?;
    }
    Ok(sonuc)
}

fn sha256_hex(baytlar: &[u8]) -> String {
    hex_yaz(&Sha256::digest(baytlar))
}

fn dosya_oku(yol: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(yol).map_err(|hata| format!("{} okunamadı: {hata}", yol.display()))
}

/// `SHA256SUMS` metni: `<hex>  <ad>` satırları ad sırasıyla.
fn ozet_metni(klasor: &Path, adlar: &[String]) -> Result<String, String> {
    let mut sirali = adlar.to_vec();
    sirali.sort();
    sirali.dedup();
    let mut sonuc = String::new();
    for ad in sirali {
        if ad.contains('/') || ad.contains('\\') || ad.is_empty() {
            return Err(format!("artefakt adı yalın dosya adı olmalı: {ad}"));
        }
        let ozet = sha256_hex(&dosya_oku(&klasor.join(&ad))?);
        sonuc.push_str(&format!("{ozet}  {ad}\n"));
    }
    Ok(sonuc)
}

fn ozetleri_coz(metin: &str) -> Result<BTreeMap<String, String>, String> {
    let mut sonuc = BTreeMap::new();
    for satir in metin.lines().filter(|s| !s.is_empty()) {
        let (ozet, ad) = satir
            .split_once("  ")
            .ok_or_else(|| format!("SHA256SUMS satırı `<hex>  <ad>` olmalı: {satir:?}"))?;
        hex_coz::<32>(ozet, "sha256")?;
        if sonuc.insert(ad.to_string(), ozet.to_string()).is_some() {
            return Err(format!("SHA256SUMS içinde yinelenen ad: {ad}"));
        }
    }
    if sonuc.is_empty() {
        return Err("SHA256SUMS boş".into());
    }
    Ok(sonuc)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KilitPaketi {
    ad: String,
    surum: String,
    ozet: Option<String>,
}

/// `Cargo.lock` `[[package]]` bloklarını (kök paket dahil) sıralı okur.
fn kilit_paketleri(metin: &str) -> Result<Vec<KilitPaketi>, String> {
    let mut paketler = Vec::new();
    let mut acik: Option<(Option<String>, Option<String>, Option<String>)> = None;
    let kapat = |acik: &mut Option<(Option<String>, Option<String>, Option<String>)>,
                 paketler: &mut Vec<KilitPaketi>|
     -> Result<(), String> {
        if let Some((ad, surum, ozet)) = acik.take() {
            paketler.push(KilitPaketi {
                ad: ad.ok_or("Cargo.lock paketinde name eksik")?,
                surum: surum.ok_or("Cargo.lock paketinde version eksik")?,
                ozet,
            });
        }
        Ok(())
    };
    for satir in metin.lines() {
        let satir = satir.trim();
        if satir == "[[package]]" {
            kapat(&mut acik, &mut paketler)?;
            acik = Some((None, None, None));
            continue;
        }
        if satir.starts_with('[') {
            kapat(&mut acik, &mut paketler)?;
            continue;
        }
        let Some(blok) = acik.as_mut() else { continue };
        let Some((anahtar, deger)) = satir.split_once(" = ") else {
            continue;
        };
        let deger = deger.trim_matches('"').to_string();
        match anahtar {
            "name" => blok.0 = Some(deger),
            "version" => blok.1 = Some(deger),
            "checksum" => blok.2 = Some(deger),
            _ => {}
        }
    }
    kapat(&mut acik, &mut paketler)?;
    paketler.sort_by(|a, b| (&a.ad, &a.surum).cmp(&(&b.ad, &b.surum)));
    Ok(paketler)
}

/// Unix saniyesini RFC 3339 UTC'ye çevirir (Howard Hinnant civil-from-days).
fn rfc3339(epoch: i64) -> String {
    let gun = epoch.div_euclid(86_400);
    let saniye = epoch.rem_euclid(86_400);
    let z = gun + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        saniye / 3600,
        (saniye % 3600) / 60,
        saniye % 60
    )
}

fn json_metni(deger: &Value) -> Result<String, String> {
    serde_json::to_string_pretty(deger)
        .map(|mut metin| {
            metin.push('\n');
            metin
        })
        .map_err(|hata| format!("JSON üretilemedi: {hata}"))
}

fn sbom_belgesi(
    git_sha: &str,
    surum: &str,
    zaman: &str,
    ozetler: &BTreeMap<String, String>,
    kilit: &[KilitPaketi],
) -> Value {
    let taban = format!("https://zee-lang.dev/spdx/surum/{git_sha}");
    let kok = format!("{taban}/package/dil");
    let mut ogeler = vec![kok.clone()];
    let mut graph = vec![
        json!({
            "type": "CreationInfo",
            "@id": "_:creationinfo",
            "specVersion": "3.0.1",
            "createdBy": [format!("{taban}/agent")],
            "createdUsing": [format!("{taban}/tool")],
            "created": zaman
        }),
        json!({"type": "Tool", "spdxId": format!("{taban}/tool"), "creationInfo": "_:creationinfo", "name": "surum_artefakti", "comment": format!("zee sürüm artefaktı üreticisi {surum}")}),
        json!({"type": "Organization", "spdxId": format!("{taban}/agent"), "creationInfo": "_:creationinfo", "name": "zee release"}),
    ];
    let mut dosya_ogeleri = Vec::new();
    for (ad, ozet) in ozetler {
        let kimlik = format!("{taban}/file/{ad}");
        ogeler.push(kimlik.clone());
        dosya_ogeleri.push(json!({
            "type": "software_File",
            "spdxId": kimlik,
            "creationInfo": "_:creationinfo",
            "name": ad,
            "software_primaryPurpose": "executable",
            "verifiedUsing": [{"type": "Hash", "algorithm": "sha256", "hashValue": ozet}]
        }));
    }
    let mut bagimlilik_ogeleri = Vec::new();
    let mut iliskiler = Vec::new();
    for paket in kilit.iter().filter(|p| p.ad != "dil") {
        let kimlik = format!("{taban}/package/{}/{}", paket.ad, paket.surum);
        ogeler.push(kimlik.clone());
        let mut oge = json!({
            "type": "software_Package",
            "spdxId": kimlik,
            "creationInfo": "_:creationinfo",
            "name": paket.ad,
            "software_packageVersion": paket.surum,
            "software_downloadLocation": format!("https://crates.io/crates/{}/{}", paket.ad, paket.surum),
            "software_copyrightText": "NOASSERTION"
        });
        if let Some(ozet) = &paket.ozet {
            oge["verifiedUsing"] =
                json!([{"type": "Hash", "algorithm": "sha256", "hashValue": ozet}]);
        }
        bagimlilik_ogeleri.push(oge);
        iliskiler.push(json!({
            "type": "Relationship",
            "spdxId": format!("{taban}/relationship/{}/{}", paket.ad, paket.surum),
            "creationInfo": "_:creationinfo",
            "from": kok,
            "relationshipType": "dependsOn",
            "to": [kimlik]
        }));
    }
    graph.push(json!({
        "type": "SpdxDocument",
        "spdxId": format!("{taban}/document"),
        "creationInfo": "_:creationinfo",
        "profileConformance": ["core", "software"],
        "rootElement": [format!("{taban}/sbom")],
        "element": ogeler
    }));
    graph.push(json!({
        "type": "software_Sbom",
        "spdxId": format!("{taban}/sbom"),
        "creationInfo": "_:creationinfo",
        "profileConformance": ["core", "software"],
        "rootElement": [kok],
        "software_sbomType": ["build"]
    }));
    graph.push(json!({
        "type": "software_Package",
        "spdxId": kok,
        "creationInfo": "_:creationinfo",
        "name": "dil",
        "software_packageVersion": surum,
        "software_downloadLocation": format!("git+https://github.com/merbay-erp/zee@{git_sha}"),
        "software_copyrightText": "NOASSERTION",
        "suppliedBy": format!("{taban}/agent"),
        "verifiedUsing": [{"type": "Hash", "algorithm": "sha256", "hashValue": sha256_hex(git_sha.as_bytes())}]
    }));
    graph.extend(dosya_ogeleri);
    graph.extend(bagimlilik_ogeleri);
    graph.extend(iliskiler);
    json!({"@context": "https://spdx.org/rdf/3.0.1/spdx-context.jsonld", "@graph": graph})
}

#[allow(clippy::too_many_arguments)]
fn provenance_belgesi(
    git_sha: &str,
    surum: &str,
    zaman: &str,
    epoch: i64,
    rustc: &str,
    platform: &str,
    ozetler: &BTreeMap<String, String>,
    ikinci_ozet: &str,
    ilk_ozet: &str,
) -> Value {
    let subject = ozetler
        .iter()
        .map(|(ad, ozet)| json!({"name": ad, "digest": {"sha256": ozet}}))
        .collect::<Vec<_>>();
    json!({
        "_type": "https://in-toto.io/Statement/v1",
        "subject": subject,
        "predicateType": "https://slsa.dev/provenance/v1",
        "predicate": {
            "buildDefinition": {
                "buildType": BUILD_TYPE,
                "externalParameters": {
                    "gitCommit": git_sha,
                    "surum": surum,
                    "rustc": rustc,
                    "platform": platform,
                    "sourceDateEpoch": epoch,
                    "cargoArgs": "cargo build --locked --release --bin dil --bin dillsp",
                    "remapPathPrefix": ["/zee", "/cargo", "/rust"]
                },
                "resolvedDependencies": [{
                    "uri": format!("git+https://github.com/merbay-erp/zee@{git_sha}"),
                    "digest": {"gitCommit": git_sha}
                }]
            },
            "runDetails": {
                "builder": {"id": BUILDER_ID},
                "metadata": {"invocationId": format!("surum-{git_sha}"), "startedOn": zaman, "finishedOn": zaman},
                "byproducts": [
                    {"name": "SHA256SUMS", "digest": {"sha256": ilk_ozet}},
                    {"name": "SHA256SUMS.ikinci-klon", "digest": {"sha256": ikinci_ozet}}
                ]
            },
            "zeeTekrarUretim": {
                "ikiTemizKlonEsit": ilk_ozet == ikinci_ozet,
                "yontem": "iki bağımsız git klonu, aynı toolchain, --remap-path-prefix, SOURCE_DATE_EPOCH"
            }
        }
    })
}

fn anahtari_oku(yol: &Path) -> Result<SigningKey, String> {
    let metin = String::from_utf8(dosya_oku(yol)?)
        .map_err(|_| "anahtar dosyası UTF-8 değil".to_string())?;
    let satirlar = metin.lines().collect::<Vec<_>>();
    if satirlar.len() != 4 || satirlar[0] != ANAHTAR_BASLIGI {
        return Err(format!("özel anahtar {ANAHTAR_BASLIGI} biçiminde değil"));
    }
    let gizli = hex_coz::<32>(
        satirlar[1]
            .strip_prefix("gizli ")
            .ok_or("gizli alanı eksik")?,
        "özel anahtar",
    )?;
    let acik = hex_coz::<32>(
        satirlar[2]
            .strip_prefix("açık ")
            .ok_or("açık alanı eksik")?,
        "açık anahtar",
    )?;
    let kimlik = satirlar[3]
        .strip_prefix("kimlik sha256:")
        .ok_or("kimlik alanı eksik")?;
    let anahtar = SigningKey::from_bytes(&gizli);
    let uretilen = anahtar.verifying_key().to_bytes();
    if uretilen != acik || sha256_hex(&uretilen) != kimlik {
        return Err("özel anahtarın açık anahtarı veya kimliği uyuşmuyor".into());
    }
    Ok(anahtar)
}

fn imza_belgesi(anahtar: &SigningKey, dosya_adi: &str, icerik: &[u8]) -> Value {
    let mut girdi = IMZA_ALANI.to_vec();
    girdi.extend_from_slice(icerik);
    let imza = anahtar.sign(&girdi);
    let acik = anahtar.verifying_key().to_bytes();
    json!({
        "sema": IMZA_SEMASI,
        "dosya": dosya_adi,
        "sha256": sha256_hex(icerik),
        "anahtar_kimligi": format!("sha256:{}", sha256_hex(&acik)),
        "acik_anahtar": hex_yaz(&acik),
        "ed25519": hex_yaz(&imza.to_bytes())
    })
}

fn imzayi_dogrula(
    belge: &Value,
    icerik: &[u8],
    beklenen_kimlik: Option<&str>,
) -> Result<String, String> {
    let alan = |ad: &str| {
        belge
            .get(ad)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("imza belgesinde {ad} eksik"))
    };
    if alan("sema")? != IMZA_SEMASI {
        return Err("imza şeması bilinmiyor".into());
    }
    if alan("sha256")? != sha256_hex(icerik) {
        return Err("dosya özeti imza belgesiyle uyuşmuyor".into());
    }
    let acik = hex_coz::<32>(alan("acik_anahtar")?, "açık anahtar")?;
    let kimlik = format!("sha256:{}", sha256_hex(&acik));
    if alan("anahtar_kimligi")? != kimlik {
        return Err("anahtar kimliği açık anahtarla uyuşmuyor".into());
    }
    if let Some(beklenen) = beklenen_kimlik {
        if beklenen != kimlik {
            return Err(format!(
                "imza beklenen {beklenen} yerine {kimlik} anahtarıyla atılmış"
            ));
        }
    }
    let imza = Signature::from_bytes(&hex_coz::<64>(alan("ed25519")?, "imza")?);
    let dogrulayici =
        VerifyingKey::from_bytes(&acik).map_err(|_| "açık anahtar geçersiz".to_string())?;
    let mut girdi = IMZA_ALANI.to_vec();
    girdi.extend_from_slice(icerik);
    dogrulayici
        .verify(&girdi, &imza)
        .map_err(|_| "Ed25519 imzası doğrulanamadı".to_string())?;
    Ok(kimlik)
}

fn secenek(argumanlar: &[String], ad: &str) -> Result<String, String> {
    argumanlar
        .iter()
        .position(|a| a == ad)
        .and_then(|i| argumanlar.get(i + 1))
        .cloned()
        .ok_or_else(|| format!("{ad} <değer> gerekli"))
}

fn calistir(argumanlar: &[String]) -> Result<String, String> {
    let komut = argumanlar.first().map(String::as_str).unwrap_or("");
    let kalan = &argumanlar[1.min(argumanlar.len())..];
    match komut {
        "ozet" => {
            let (klasor, adlar) = kalan.split_first().ok_or("ozet <klasör> <ad>... gerekli")?;
            ozet_metni(Path::new(klasor), adlar)
        }
        "zaman" => {
            let epoch = kalan
                .first()
                .ok_or("zaman <epoch> gerekli")?
                .parse::<i64>()
                .map_err(|_| "epoch tam sayı olmalı")?;
            Ok(format!("{}\n", rfc3339(epoch)))
        }
        "sbom" => {
            let ozetler = ozetleri_coz(
                &String::from_utf8(dosya_oku(Path::new(&secenek(kalan, "--ozet")?))?)
                    .map_err(|_| "SHA256SUMS UTF-8 değil")?,
            )?;
            let kilit = kilit_paketleri(
                &String::from_utf8(dosya_oku(Path::new(&secenek(kalan, "--kilit")?))?)
                    .map_err(|_| "Cargo.lock UTF-8 değil")?,
            )?;
            let epoch = secenek(kalan, "--epoch")?
                .parse::<i64>()
                .map_err(|_| "--epoch tam sayı olmalı")?;
            json_metni(&sbom_belgesi(
                &secenek(kalan, "--sha")?,
                env!("CARGO_PKG_VERSION"),
                &rfc3339(epoch),
                &ozetler,
                &kilit,
            ))
        }
        "provenance" => {
            let ilk = dosya_oku(Path::new(&secenek(kalan, "--ozet")?))?;
            let ikinci = dosya_oku(Path::new(&secenek(kalan, "--ikinci-ozet")?))?;
            let ozetler = ozetleri_coz(
                &String::from_utf8(ilk.clone()).map_err(|_| "SHA256SUMS UTF-8 değil")?,
            )?;
            let epoch = secenek(kalan, "--epoch")?
                .parse::<i64>()
                .map_err(|_| "--epoch tam sayı olmalı")?;
            json_metni(&provenance_belgesi(
                &secenek(kalan, "--sha")?,
                env!("CARGO_PKG_VERSION"),
                &rfc3339(epoch),
                epoch,
                &secenek(kalan, "--rustc")?,
                &secenek(kalan, "--platform")?,
                &ozetler,
                &sha256_hex(&ikinci),
                &sha256_hex(&ilk),
            ))
        }
        "imzala" => {
            let anahtar = anahtari_oku(Path::new(&secenek(kalan, "--anahtar")?))?;
            let dosya = PathBuf::from(kalan.last().ok_or("imzala --anahtar <yol> <dosya>")?);
            let ad = dosya
                .file_name()
                .and_then(|a| a.to_str())
                .ok_or("dosya adı")?
                .to_string();
            json_metni(&imza_belgesi(&anahtar, &ad, &dosya_oku(&dosya)?))
        }
        "dogrula" => {
            let dosya = PathBuf::from(secenek(kalan, "--dosya")?);
            let imza: Value =
                serde_json::from_slice(&dosya_oku(Path::new(&secenek(kalan, "--imza")?))?)
                    .map_err(|hata| format!("imza belgesi JSON değil: {hata}"))?;
            let beklenen = secenek(kalan, "--kimlik").ok();
            let kimlik = imzayi_dogrula(&imza, &dosya_oku(&dosya)?, beklenen.as_deref())?;
            Ok(format!("imza geçerli: {} ({kimlik})\n", dosya.display()))
        }
        _ => Err("kullanım: surum_artefakti ozet|zaman|sbom|provenance|imzala|dogrula ...".into()),
    }
}

fn main() -> ExitCode {
    let argumanlar = std::env::args().skip(1).collect::<Vec<_>>();
    match calistir(&argumanlar) {
        Ok(cikti) => {
            print!("{cikti}");
            ExitCode::SUCCESS
        }
        Err(hata) => {
            eprintln!("HATA: {hata}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    const KILIT: &str = "# lock\nversion = 4\n\n[[package]]\nname = \"dil\"\nversion = \"0.8.0-dev\"\ndependencies = [\n \"sha2\",\n]\n\n[[package]]\nname = \"sha2\"\nversion = \"0.11.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"abc\"\n";

    #[test]
    fn kilit_paketleri_sirali_ve_kok_dahil_okunur() {
        let paketler = kilit_paketleri(KILIT).unwrap();
        assert_eq!(paketler.len(), 2);
        assert_eq!(paketler[0].ad, "dil");
        assert_eq!(paketler[0].ozet, None);
        assert_eq!(paketler[1].ozet.as_deref(), Some("abc"));
        assert!(kilit_paketleri("[[package]]\nversion = \"1\"\n").is_err());
    }

    #[test]
    fn rfc3339_utc_donusumu_bilinen_anlari_verir() {
        assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(rfc3339(1_788_912_000), "2026-09-09T00:00:00Z");
        assert_eq!(rfc3339(-1), "1969-12-31T23:59:59Z");
    }

    #[test]
    fn sbom_ve_provenance_ayni_girdide_byte_byte_ayni_ve_ozetleri_tasir() {
        let ozetler = ozetleri_coz("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  dil\nbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb  dillsp\n").unwrap();
        let kilit = kilit_paketleri(KILIT).unwrap();
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let a = json_metni(&sbom_belgesi(
            sha,
            "0.8.0-dev",
            &rfc3339(10),
            &ozetler,
            &kilit,
        ))
        .unwrap();
        let b = json_metni(&sbom_belgesi(
            sha,
            "0.8.0-dev",
            &rfc3339(10),
            &ozetler,
            &kilit,
        ))
        .unwrap();
        assert_eq!(a, b);
        assert!(a.contains(
            "\"hashValue\": \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\""
        ));
        assert!(a.contains("https://crates.io/crates/sha2/0.11.0"));
        assert!(
            !a.contains("/package/dil/0.8.0-dev"),
            "kök paket bağımlılık olarak listelenmez"
        );
        let p = provenance_belgesi(
            sha,
            "0.8.0-dev",
            &rfc3339(10),
            10,
            "rustc 1.93.1",
            "x86_64-unknown-linux-gnu",
            &ozetler,
            "x",
            "x",
        );
        assert_eq!(
            p["predicate"]["zeeTekrarUretim"]["ikiTemizKlonEsit"],
            json!(true)
        );
        assert_eq!(p["subject"].as_array().unwrap().len(), 2);
        let farkli = provenance_belgesi(
            sha,
            "0.8.0-dev",
            &rfc3339(10),
            10,
            "r",
            "p",
            &ozetler,
            "x",
            "y",
        );
        assert_eq!(
            farkli["predicate"]["zeeTekrarUretim"]["ikiTemizKlonEsit"],
            json!(false)
        );
        assert!(ozetleri_coz("kısa  dil\n").is_err());
        assert!(ozetleri_coz("").is_err());
    }

    #[test]
    fn imza_gidis_donus_ve_oynama_reddi() {
        let gecici = std::env::temp_dir().join(format!("zee-surum-imza-{}", std::process::id()));
        std::fs::create_dir_all(&gecici).unwrap();
        let anahtar_yolu = gecici.join("anahtar.zee-anahtar");
        dil::artefakt_dogrulama::anahtar_uret(&anahtar_yolu).unwrap();
        let anahtar = anahtari_oku(&anahtar_yolu).unwrap();
        let icerik = b"aaaa  dil\n";
        let belge = imza_belgesi(&anahtar, "SHA256SUMS", icerik);
        let kimlik = imzayi_dogrula(&belge, icerik, None).unwrap();
        assert!(kimlik.starts_with("sha256:"));
        assert!(imzayi_dogrula(&belge, b"aaaa  dil\nekstra\n", None).is_err());
        assert!(imzayi_dogrula(&belge, icerik, Some("sha256:yanlis")).is_err());
        let mut bozuk = belge.clone();
        bozuk["ed25519"] = json!(format!(
            "{}{}",
            "00",
            &belge["ed25519"].as_str().unwrap()[2..]
        ));
        assert!(imzayi_dogrula(&bozuk, icerik, None).is_err());
        std::fs::write(&anahtar_yolu, "bozuk\n").unwrap();
        assert!(anahtari_oku(&anahtar_yolu).is_err());
        let _ = std::fs::remove_dir_all(&gecici);
    }

    #[test]
    fn ozet_metni_adlari_siralar_ve_yol_ayracini_reddeder() {
        let gecici = std::env::temp_dir().join(format!("zee-surum-ozet-{}", std::process::id()));
        std::fs::create_dir_all(&gecici).unwrap();
        std::fs::write(gecici.join("b"), b"2").unwrap();
        std::fs::write(gecici.join("a"), b"1").unwrap();
        let metin = ozet_metni(&gecici, &["b".into(), "a".into()]).unwrap();
        assert!(metin.starts_with(&format!("{}  a\n", sha256_hex(b"1"))));
        assert!(ozet_metni(&gecici, &["../a".into()]).is_err());
        let _ = std::fs::remove_dir_all(&gecici);
    }
}
