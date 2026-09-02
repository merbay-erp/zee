//! Sürümlü morfoloji davranışının kanonik uyumluluk parmak izi.

use super::{cozumleri_bul, ek_zinciri_uydur, profil_dokumu, SoyutEk, MORFOLOJI_PROFILI};

const SEMANTIK_SEMA: &str = "zee-morfoloji-uyumluluk-v1";
const TEK_EKLER: &[SoyutEk] = &[
    SoyutEk::Belirtme,
    SoyutEk::Tamlayan,
    SoyutEk::Yonelme,
    SoyutEk::Ayrilma,
    SoyutEk::Bulunma,
    SoyutEk::Arac,
    SoyutEk::CogulYonelme,
];
const DIS_EKLER: &[SoyutEk] = &[
    SoyutEk::Belirtme,
    SoyutEk::Tamlayan,
    SoyutEk::Yonelme,
    SoyutEk::Ayrilma,
    SoyutEk::Bulunma,
    SoyutEk::Arac,
];
const HAM_YUZEYLER: &[&str] = &[
    "payı",
    "sayacı",
    "fiyatıyla",
    "zarından",
    "üssü",
    "affı",
    "reddi",
    "tıbbı",
    "şekli",
    "burnu",
    "oğlu",
];

/// `zee-tr-N` davranışını tablo, kanonik üretim ve bütün çözüm kümeleriyle
/// tek SHA-256 kaydına indirger. Kayıt değişirse eski profil fixture'ı
/// güncellenmez; yeni profil kimliği ve yeni fixture açılır.
pub fn profil_uyumluluk_kaydi() -> String {
    let semantik = profil_semantik_akisi();
    format!(
        "profil={}\nşema={}\nsha256={}\n",
        MORFOLOJI_PROFILI,
        SEMANTIK_SEMA,
        crate::guvenlik::sha256_hex(semantik.as_bytes())
    )
}

fn profil_semantik_akisi() -> String {
    let mut akis = format!("şema={}\n{}", SEMANTIK_SEMA, profil_dokumu());
    for yuzey in HAM_YUZEYLER {
        cozum_satiri_ekle(&mut akis, "ham", "-", yuzey);
    }
    for tohum in 0..4_096 {
        let kok = deterministik_kok(tohum);
        for &ek in TEK_EKLER {
            uretim_satiri_ekle(&mut akis, &kok, &[ek]);
        }
        for &dis in DIS_EKLER {
            uretim_satiri_ekle(&mut akis, &kok, &[SoyutEk::Iyelik, dis]);
        }
    }
    akis
}

fn uretim_satiri_ekle(akis: &mut String, kok: &str, ekler: &[SoyutEk]) {
    let ek_yolu = ekler
        .iter()
        .map(|ek| ek.adi())
        .collect::<Vec<_>>()
        .join("+");
    let yuzey = ek_zinciri_uydur(kok, ekler).unwrap_or_else(|| "<geçersiz>".into());
    cozum_satiri_ekle(akis, kok, &ek_yolu, &yuzey);
}

fn cozum_satiri_ekle(akis: &mut String, kok: &str, ek_yolu: &str, yuzey: &str) {
    let cozumler = cozumleri_bul(yuzey)
        .into_iter()
        .map(|cozum| {
            let ekler = cozum
                .ekler
                .iter()
                .map(|ek| ek.adi())
                .collect::<Vec<_>>()
                .join("+");
            format!("{}+{}", cozum.kok, ekler)
        })
        .collect::<Vec<_>>()
        .join("|");
    akis.push_str(&format!("{}\t{}\t{}\t{}\n", kok, ek_yolu, yuzey, cozumler));
}

fn deterministik_kok(tohum: u64) -> String {
    const ILK: &[char] = &[
        'a', 'b', 'c', 'ç', 'd', 'e', 'f', 'g', 'ğ', 'h', 'ı', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
        'ö', 'p', 'r', 's', 'ş', 't', 'u', 'ü', 'v', 'y', 'z', 'â', 'î', 'û', '_',
    ];
    const DEVAM: &[char] = &[
        'a', 'e', 'ı', 'i', 'o', 'ö', 'u', 'ü', 'b', 'c', 'ç', 'd', 'f', 'g', 'ğ', 'h', 'j', 'k',
        'l', 'm', 'n', 'p', 'r', 's', 'ş', 't', 'v', 'y', 'z', 'â', 'î', 'û', '_', '0', '1', '2',
        '7', '9',
    ];
    let mut durum = tohum.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let uzunluk = 2 + durum as usize % 31;
    durum = durum
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1);
    let mut kok = String::with_capacity(uzunluk);
    kok.push(ILK[durum as usize % ILK.len()]);
    for _ in 1..uzunluk {
        durum = durum
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        kok.push(DEVAM[durum as usize % DEVAM.len()]);
    }
    kok
}
