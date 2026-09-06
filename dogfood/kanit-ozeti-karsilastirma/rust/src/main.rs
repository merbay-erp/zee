//! K-165 karşılaştırma: `dogfood/kanit-ozeti` Zee ürününün Rust eşdeğeri.
//! Aynı dokuz kayıt defterini okur, aynı Markdown sayfasını bayt bayt üretir.
//! Kullanım: `kanit_ozeti_rs <depo-kökü>` → sayfa stdout'a yazılır.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;

fn satirlar(kok: &Path, goreli: &str) -> Vec<String> {
    let metin = std::fs::read_to_string(kok.join(goreli))
        .unwrap_or_else(|hata| panic!("{goreli} okunamadı: {hata}"));
    metin.lines().map(str::to_string).collect()
}

fn veri_satirlarini_ayikla(satirlar: &[String]) -> Vec<&String> {
    satirlar
        .iter()
        .filter(|s| {
            let temiz = s.trim();
            !temiz.is_empty() && !temiz.starts_with('#')
        })
        .collect()
}

fn baslik_satirini_bul(satirlar: &[String]) -> Vec<String> {
    let mut baslik = String::new();
    for satir in satirlar {
        if satir.starts_with("# ") && satir.contains('\t') {
            baslik = satir.replace("# ", "");
        }
    }
    if baslik.is_empty() {
        return Vec::new();
    }
    baslik.split('\t').map(str::to_string).collect()
}

fn sutunu_topla(satirlar: &[String], sutun: &str) -> Vec<String> {
    let basliklar = baslik_satirini_bul(satirlar);
    let indeks = basliklar.iter().position(|b| b == sutun);
    veri_satirlarini_ayikla(satirlar)
        .into_iter()
        .map(|veri| {
            let alanlar: Vec<&str> = veri.split('\t').collect();
            indeks
                .and_then(|i| alanlar.get(i))
                .map(|a| a.to_string())
                .unwrap_or_default()
        })
        .collect()
}

fn ciftleri_birlestir(birinciler: &[String], ikinciler: &[String], ayirici: &str) -> Vec<String> {
    birinciler
        .iter()
        .zip(ikinciler)
        .map(|(a, b)| format!("{a}{ayirici}{b}"))
        .collect()
}

fn turkce_harf_sirasi(k: char) -> (u8, u32) {
    const ALFABE: &str = "abcçdefgğhıijklmnoöprsştuüvyz";
    let kucuk = match k {
        'İ' => 'i',
        'I' => 'ı',
        _ => k.to_lowercase().next().unwrap_or(k),
    };
    match ALFABE.chars().position(|a| a == kucuk) {
        Some(sira) => (0, sira as u32),
        None => (1, k as u32),
    }
}

fn turkce_karsilastir(a: &str, b: &str) -> Ordering {
    a.chars()
        .map(turkce_harf_sirasi)
        .cmp(b.chars().map(turkce_harf_sirasi))
}

struct Sayaclar(HashMap<String, i64>);

impl Sayaclar {
    fn say(degerler: &[String]) -> Sayaclar {
        let mut sayaclar = HashMap::new();
        for deger in degerler {
            *sayaclar.entry(deger.clone()).or_insert(0) += 1;
        }
        Sayaclar(sayaclar)
    }
    fn oku(&self, anahtar: &str) -> i64 {
        self.0.get(anahtar).copied().unwrap_or(0)
    }
    fn sirali_anahtarlar(&self) -> Vec<String> {
        let mut anahtarlar: Vec<String> = self.0.keys().cloned().collect();
        anahtarlar.sort_by(|a, b| turkce_karsilastir(a, b));
        anahtarlar
    }
}

fn yuzde(bolunen: i64, bolen: i64) -> i64 {
    if bolen == 0 {
        0
    } else {
        bolunen * 100 / bolen
    }
}

fn hucreleri_birlestir(hucreler: &[String]) -> String {
    format!("| {} |", hucreler.join(" | "))
}

fn tablo_basligi(sutunlar: &[&str]) -> Vec<String> {
    let baslik: Vec<String> = sutunlar.iter().map(|s| s.to_string()).collect();
    let cizgi: Vec<String> = sutunlar.iter().map(|_| "---".to_string()).collect();
    vec![hucreleri_birlestir(&baslik), hucreleri_birlestir(&cizgi)]
}

fn sayim_tablosu(sayaclar: &Sayaclar, etiket: &str, olcu: &str) -> Vec<String> {
    let mut cikti = tablo_basligi(&[etiket, olcu]);
    for anahtar in sayaclar.sirali_anahtarlar() {
        cikti.push(hucreleri_birlestir(&[
            anahtar.clone(),
            sayaclar.oku(&anahtar).to_string(),
        ]));
    }
    cikti
}

fn bolum_basligi(ad: &str, yol: &str) -> Vec<String> {
    vec![
        String::new(),
        format!("## {ad}"),
        String::new(),
        format!("Kaynak: `{yol}`"),
        String::new(),
    ]
}

fn regresyon_bolumu(satirlar: &[String]) -> Vec<String> {
    let mut bolum = bolum_basligi("Semantic regresyon korpusu", "regression/v2.tsv");
    let fazlar = sutunu_topla(satirlar, "faz");
    bolum.extend(sayim_tablosu(&Sayaclar::say(&fazlar), "Faz", "Vaka"));
    bolum.push(String::new());
    let kodlar = sutunu_topla(satirlar, "tani");
    bolum.extend(sayim_tablosu(&Sayaclar::say(&kodlar), "Tanı", "Vaka"));
    let duzeltmeler = sutunu_topla(satirlar, "fixed_by");
    let tam = duzeltmeler.iter().filter(|d| d.chars().count() == 40).count();
    let toplam = fazlar.len();
    bolum.push(String::new());
    bolum.push(format!(
        "Toplam {toplam} vaka; `-` tanısız çalışma/eşzamanlılık vakasıdır. Tam 40 karakterlik `fixed_by` commit'i taşıyan vaka: {tam}/{toplam}."
    ));
    bolum
}

fn guvenlik_bolumu(satirlar: &[String]) -> Vec<String> {
    let mut bolum = bolum_basligi("Güvenlik bulguları", "docs/guvenlik-bulgulari-v1.tsv");
    let onemler = sutunu_topla(satirlar, "onem");
    let durumlar = sutunu_topla(satirlar, "durum");
    let sayaclar = Sayaclar::say(&ciftleri_birlestir(&onemler, &durumlar, "|"));
    bolum.extend(tablo_basligi(&["Önem", "Kapalı", "Kabul", "Açık", "Toplam"]));
    let mut acik_agir = 0;
    for onem in ["kritik", "yuksek", "orta", "dusuk"] {
        let kapali = sayaclar.oku(&format!("{onem}|kapali"));
        let kabul = sayaclar.oku(&format!("{onem}|kabul"));
        let acik = sayaclar.oku(&format!("{onem}|acik"));
        if onem == "kritik" || onem == "yuksek" {
            acik_agir += acik;
        }
        bolum.push(hucreleri_birlestir(&[
            onem.to_string(),
            kapali.to_string(),
            kabul.to_string(),
            acik.to_string(),
            (kapali + kabul + acik).to_string(),
        ]));
    }
    bolum.push(String::new());
    if acik_agir == 0 {
        bolum.push("Kapı: açık kritik/yüksek bulgu 0 → **GEÇTİ** (ADR-066 sürekli kapı koşulu).".into());
    } else {
        bolum.push(format!("Kapı: açık kritik/yüksek bulgu {acik_agir} → **KALDI**."));
    }
    bolum.push(format!("Toplam bulgu: {}.", onemler.len()));
    bolum
}

fn spec_bolumu(satirlar: &[String]) -> Vec<String> {
    let mut bolum = bolum_basligi("Spec maddeleri", "docs/spec-madde-kaniti-v1.tsv");
    let maddeler = sutunu_topla(satirlar, "madde");
    let durumlar = sutunu_topla(satirlar, "durum");
    let dosyalar: Vec<String> = maddeler
        .iter()
        .map(|m| m.split('#').next().unwrap_or("").to_string())
        .collect();
    let sayaclar = Sayaclar::say(&ciftleri_birlestir(&dosyalar, &durumlar, "|"));
    let dosya_sayaclari = Sayaclar::say(&dosyalar);
    bolum.extend(tablo_basligi(&["Bölüm", "Kanıtlı", "Kısmi", "Açık", "Toplam", "Kanıt %"]));
    let (mut tk, mut tp, mut ta) = (0, 0, 0);
    for dosya in dosya_sayaclari.sirali_anahtarlar() {
        let kanitli = sayaclar.oku(&format!("{dosya}|kanitli"));
        let kismi = sayaclar.oku(&format!("{dosya}|kismi"));
        let acik = sayaclar.oku(&format!("{dosya}|acik"));
        let toplam = kanitli + kismi + acik;
        tk += kanitli;
        tp += kismi;
        ta += acik;
        bolum.push(hucreleri_birlestir(&[
            dosya,
            kanitli.to_string(),
            kismi.to_string(),
            acik.to_string(),
            toplam.to_string(),
            yuzde(kanitli, toplam).to_string(),
        ]));
    }
    let genel = tk + tp + ta;
    bolum.push(hucreleri_birlestir(&[
        "**Toplam**".into(),
        tk.to_string(),
        tp.to_string(),
        ta.to_string(),
        genel.to_string(),
        yuzde(tk, genel).to_string(),
    ]));
    bolum
}

fn dogfood_bolumu(kayitlar: &[String], vakalar: &[String]) -> Vec<String> {
    let mut bolum = bolum_basligi(
        "Dogfood ürünleri ve korpusu",
        "docs/dogfood-projeleri-v1.tsv + dogfood/korpus-v1.tsv",
    );
    let urunler = sutunu_topla(kayitlar, "urun");
    let kokler = sutunu_topla(kayitlar, "kok");
    let durumlar = sutunu_topla(kayitlar, "durum");
    let urun_durumlari = ciftleri_birlestir(&urunler, &durumlar, " (");
    let cizgiler = ciftleri_birlestir(&urun_durumlari, &kokler, ") | ");
    bolum.extend(tablo_basligi(&["Ürün (durum)", "Kök"]));
    for cizgi in &cizgiler {
        bolum.push(format!("| {cizgi} |"));
    }
    bolum.push(String::new());
    let vaka_urunleri = sutunu_topla(vakalar, "urun");
    let kipler = sutunu_topla(vakalar, "kip");
    let beklentiler = sutunu_topla(vakalar, "beklenti");
    let kip_sayaclari = Sayaclar::say(&ciftleri_birlestir(&vaka_urunleri, &kipler, "|"));
    let beklenti_sayaclari = Sayaclar::say(&ciftleri_birlestir(&vaka_urunleri, &beklentiler, "|"));
    let urun_sayaclari = Sayaclar::say(&vaka_urunleri);
    bolum.extend(tablo_basligi(&[
        "Ürün", "denetle", "calistir", "proje", "basarili", "basarisiz", "Toplam",
    ]));
    for urun in urun_sayaclari.sirali_anahtarlar() {
        bolum.push(hucreleri_birlestir(&[
            urun.clone(),
            kip_sayaclari.oku(&format!("{urun}|denetle")).to_string(),
            kip_sayaclari.oku(&format!("{urun}|calistir")).to_string(),
            kip_sayaclari.oku(&format!("{urun}|proje")).to_string(),
            beklenti_sayaclari.oku(&format!("{urun}|basarili")).to_string(),
            beklenti_sayaclari.oku(&format!("{urun}|basarisiz")).to_string(),
            urun_sayaclari.oku(&urun).to_string(),
        ]));
    }
    bolum.push(String::new());
    bolum.push(format!("Toplam korpus vakası: {}.", vaka_urunleri.len()));
    bolum
}

fn deprecation_bolumu(satirlar: &[String]) -> Vec<String> {
    let mut bolum = bolum_basligi("Deprecation kayıtları", "docs/deprecation-kayitlari-v1.tsv");
    let durumlar = sutunu_topla(satirlar, "durum");
    bolum.extend(sayim_tablosu(&Sayaclar::say(&durumlar), "Durum", "Kayıt"));
    bolum.push(String::new());
    let yuzeyler = sutunu_topla(satirlar, "yuzey");
    let siniflar = sutunu_topla(satirlar, "sinif");
    let ciftler = ciftleri_birlestir(&yuzeyler, &siniflar, " / ");
    bolum.extend(sayim_tablosu(&Sayaclar::say(&ciftler), "Yüzey / sınıf", "Kayıt"));
    bolum.push(String::new());
    bolum.push(format!("Toplam kayıt: {}.", durumlar.len()));
    bolum
}

fn soak_bolumu(satirlar: &[String]) -> Vec<String> {
    let mut bolum = bolum_basligi("Uzun soak tarihçesi", "docs/soak-gecmisi-v1.tsv");
    let sonuclar = sutunu_topla(satirlar, "sonuc");
    bolum.extend(sayim_tablosu(&Sayaclar::say(&sonuclar), "Sonuç", "Koşu"));
    bolum.push(String::new());
    if sonuclar.is_empty() {
        bolum.push("Henüz soak koşusu yok.".into());
        return bolum;
    }
    let son = |sutun: &str| sutunu_topla(satirlar, sutun).last().cloned().unwrap_or_default();
    bolum.push(format!(
        "Son koşu: `{}` ({}, {}, {} sn): derleyici %{}, dillsp %{} RSS büyümesi → {}.",
        son("git_sha"),
        son("tarih"),
        son("platform"),
        son("sure_sn"),
        son("derleyici_buyume_yuzde"),
        son("dillsp_buyume_yuzde"),
        sonuclar.last().cloned().unwrap_or_default()
    ));
    bolum
}

fn beyan_bolumu(degisiklikler: &[String], donmalar: &[String]) -> Vec<String> {
    let mut bolum = bolum_basligi(
        "Compiler değişiklik ve core freeze beyanları",
        "docs/compiler-degisiklik-beyanlari-v1.tsv + docs/core-freeze-beyanlari-v1.tsv",
    );
    let degisiklik_siniflari = sutunu_topla(degisiklikler, "sinif");
    bolum.extend(sayim_tablosu(&Sayaclar::say(&degisiklik_siniflari), "Değişiklik sınıfı", "Beyan"));
    bolum.push(String::new());
    let freeze_siniflari = sutunu_topla(donmalar, "sinif");
    bolum.extend(sayim_tablosu(&Sayaclar::say(&freeze_siniflari), "Freeze sınıfı", "Beyan"));
    bolum.push(String::new());
    bolum.push(format!(
        "Toplam: {} compiler değişiklik beyanı, {} core freeze beyanı.",
        degisiklik_siniflari.len(),
        freeze_siniflari.len()
    ));
    bolum
}

pub fn sayfayi_uret(kok: &Path) -> String {
    let mut cikti: Vec<String> = vec![
        "# Kanıt özeti".into(),
        String::new(),
        "Bu sayfa `dogfood/kanit-ozeti` Zee ürünü tarafından depo kayıt defterlerinden üretilir (K-164/ADR-072) ve elle düzenlenmez. Yenilemek için: `cd compiler && cargo run --locked -- çalıştır ../dogfood/kanit-ozeti/kaynak/ana.dil`. CI aynı komutu koşar ve bu dosya bayatsa kırılır.".into(),
    ];
    cikti.extend(regresyon_bolumu(&satirlar(kok, "regression/v2.tsv")));
    cikti.extend(guvenlik_bolumu(&satirlar(kok, "docs/guvenlik-bulgulari-v1.tsv")));
    cikti.extend(spec_bolumu(&satirlar(kok, "docs/spec-madde-kaniti-v1.tsv")));
    cikti.extend(dogfood_bolumu(
        &satirlar(kok, "docs/dogfood-projeleri-v1.tsv"),
        &satirlar(kok, "dogfood/korpus-v1.tsv"),
    ));
    cikti.extend(deprecation_bolumu(&satirlar(kok, "docs/deprecation-kayitlari-v1.tsv")));
    cikti.extend(soak_bolumu(&satirlar(kok, "docs/soak-gecmisi-v1.tsv")));
    cikti.extend(beyan_bolumu(
        &satirlar(kok, "docs/compiler-degisiklik-beyanlari-v1.tsv"),
        &satirlar(kok, "docs/core-freeze-beyanlari-v1.tsv"),
    ));
    let mut sayfa = cikti.join("\n");
    sayfa.push('\n');
    sayfa
}

fn main() {
    let kok = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    print!("{}", sayfayi_uret(Path::new(&kok)));
}

#[cfg(test)]
mod testler {
    use super::*;

    fn l(satirlar: &[&str]) -> Vec<String> {
        satirlar.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn baslik_son_sekmeli_yorum_satiridir() {
        let s = l(&["# zee-x-1", "# enforcement_parent\tabc", "# kimlik\tdurum", "GB-1\tacik"]);
        assert_eq!(baslik_satirini_bul(&s), vec!["kimlik", "durum"]);
    }

    #[test]
    fn sutun_toplama_veri_sirasini_korur_ve_eksik_alani_bos_verir() {
        let s = l(&["# kimlik\tdurum", "GB-1\tacik", "", "GB-2"]);
        assert_eq!(sutunu_topla(&s, "durum"), vec!["acik", ""]);
    }

    #[test]
    fn turk_alfabesi_sirasi_c_k_den_once_ve_tire_sonda() {
        let sayaclar = Sayaclar::say(&l(&["kapali", "acik", "çok", "-"]));
        assert_eq!(sayaclar.sirali_anahtarlar(), vec!["acik", "çok", "kapali", "-"]);
    }

    #[test]
    fn guvenlik_kapisi_acik_agir_bulguda_kalir() {
        let s = l(&["# kimlik\tonem\tdurum", "GB-1\tyuksek\tacik"]);
        let bolum = guvenlik_bolumu(&s).join("\n");
        assert!(bolum.contains("Kapı: açık kritik/yüksek bulgu 1 → **KALDI**."));
    }

    #[test]
    fn spec_toplam_satiri_yuzdeyi_tam_sayi_verir() {
        let s = l(&["# madde\tdurum", "spec/02-dizim.md#a\tkanitli", "spec/02-dizim.md#b\tkismi", "spec/01-sozcukleme.md#c\tacik"]);
        let bolum = spec_bolumu(&s);
        assert_eq!(bolum.last().unwrap(), "| **Toplam** | 1 | 1 | 1 | 3 | 33 |");
    }

    #[test]
    fn bos_soak_tarihcesi_acikca_soylenir() {
        let s = l(&["# git_sha\tsonuc"]);
        assert_eq!(soak_bolumu(&s).last().unwrap(), "Henüz soak koşusu yok.");
    }
}
