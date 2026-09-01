use super::*;

/// Çıkarımlı ya da açık işlem imzası. Yalnız çıkarımlı imza sayısal
/// genişlemeyle değişebilir.
pub(super) struct Imza {
    pub(super) parametre_turleri: Vec<Tur>,
    pub(super) donus: Option<Tur>,
    /// K-083 açık parametre sözleşmesi çağrılarla terfi ettirilemez.
    pub(super) acik: bool,
}

pub(super) struct Baglam {
    pub(super) islemler: HashMap<String, Islem>,
    pub(super) imzalar: HashMap<IslemId, Imza>,
    pub(super) yapilar: Vec<Yapi>,
    yapi_kimlikleri: HashMap<String, YapiId>,
    yapi_konumlari: HashMap<YapiId, usize>,
    islem_kimlikleri: HashMap<String, IslemId>,
    /// `eşzamanlı olarak` görev adları; `hepsini bekle`ye dek erişilemez (T033).
    pub(super) bekleyen_gorevler: std::collections::HashSet<String>,
    /// Denetimi süren işlemler (özyineleme desteği): ad + parametre türleri +
    /// o ana dek görülen dönüş türleri. `döndür` en üsttekine yazar.
    pub(super) denetim_yigini: Vec<ImzaKaydi>,
    /// Akış-duyarlı daraltma (RFC-0008 §4.2): "X varsa" dalında X'in değeri
    /// güvenlidir; "X başarılıysa" dalında değeri, "başarısızsa" dalında hatası.
    pub(super) dolu_secenekler: std::collections::HashSet<String>,
    pub(super) basarili_sonuclar: std::collections::HashSet<String>,
    pub(super) basarisiz_sonuclar: std::collections::HashSet<String>,
    /// K-093: gezme boyunca biçimi sabit kalan kaynak listeler/sözlükler.
    /// Aynı kaynağı yeniden bağlama, ekleme, silme ve iç içe gezme T053'tür.
    pub(super) gezilen_koleksiyonlar: std::collections::HashSet<String>,
    /// Checker'ın HIR lowering'e devrettiği ifade türü ve semantic bağları.
    pub(super) hir_ifadeleri: HashMap<usize, crate::hir::HirIfadeBilgisi>,
    hir_sembol_adlari: HashMap<SymbolId, String>,
    hir_sembol_tanimlari: HashMap<SymbolId, crate::hir::HirKaynakAraligi>,
    hir_sembol_yazimlari: Vec<crate::hir::HirSembolKullanimi>,
}

impl Baglam {
    pub(super) fn yeni(islemler: HashMap<String, Islem>, yapilar: Vec<Yapi>) -> Self {
        let mut yapi_adlari = yapilar
            .iter()
            .map(|yapi| yapi.ad.clone())
            .collect::<Vec<_>>();
        yapi_adlari.sort();
        let yapi_kimlikleri = yapi_adlari
            .into_iter()
            .enumerate()
            .map(|(sira, ad)| (ad, YapiId::yeni(sira)))
            .collect::<HashMap<_, _>>();
        let yapi_konumlari = yapilar
            .iter()
            .enumerate()
            .map(|(konum, yapi)| (yapi_kimlikleri[&yapi.ad], konum))
            .collect();

        let mut islem_adlari = islemler.keys().cloned().collect::<Vec<_>>();
        islem_adlari.sort();
        let islem_kimlikleri = islem_adlari
            .into_iter()
            .enumerate()
            .map(|(sira, ad)| (ad, IslemId::yeni(sira)))
            .collect();

        Self {
            islemler,
            imzalar: HashMap::new(),
            yapilar,
            yapi_kimlikleri,
            yapi_konumlari,
            islem_kimlikleri,
            bekleyen_gorevler: std::collections::HashSet::new(),
            denetim_yigini: Vec::new(),
            dolu_secenekler: std::collections::HashSet::new(),
            basarili_sonuclar: std::collections::HashSet::new(),
            basarisiz_sonuclar: std::collections::HashSet::new(),
            gezilen_koleksiyonlar: std::collections::HashSet::new(),
            hir_ifadeleri: HashMap::new(),
            hir_sembol_adlari: HashMap::new(),
            hir_sembol_tanimlari: HashMap::new(),
            hir_sembol_yazimlari: Vec::new(),
        }
    }

    pub(super) fn hir_bilgisi(&self) -> crate::hir::HirOlusturmaBilgisi {
        crate::hir::HirOlusturmaBilgisi {
            ifadeler: self.hir_ifadeleri.clone(),
            sembol_adlari: self.hir_sembol_adlari.clone(),
            sembol_tanimlari: self.hir_sembol_tanimlari.clone(),
            sembol_yazimlari: self.hir_sembol_yazimlari.clone(),
            islem_adlari: self
                .islem_kimlikleri
                .iter()
                .map(|(ad, kimlik)| (*kimlik, ad.clone()))
                .collect(),
            yapi_konumlari: self.yapi_konumlari.clone(),
        }
    }

    pub(super) fn hir_sembol_adi_ekle(&mut self, kimlik: SymbolId, ad: String) {
        self.hir_sembol_adlari.entry(kimlik).or_insert(ad);
    }

    pub(super) fn hir_sembol_yazimi_ekle(
        &mut self,
        kimlik: SymbolId,
        kaynak_araligi: crate::hir::HirKaynakAraligi,
    ) {
        self.hir_sembol_tanimlari.entry(kimlik).or_insert(kaynak_araligi);
        let kullanim = crate::hir::HirSembolKullanimi::yeni(kimlik, kaynak_araligi);
        if !self.hir_sembol_yazimlari.contains(&kullanim) {
            self.hir_sembol_yazimlari.push(kullanim);
        }
    }

    pub(super) fn yapi_kimligi(&self, ad: &str) -> Option<YapiId> {
        self.yapi_kimlikleri.get(ad).copied()
    }

    pub(super) fn yapi(&self, kimlik: YapiId) -> Option<&Yapi> {
        self.yapi_konumlari
            .get(&kimlik)
            .and_then(|konum| self.yapilar.get(*konum))
    }

    pub(super) fn islem_kimligi(&self, ad: &str) -> Option<IslemId> {
        self.islem_kimlikleri.get(ad).copied()
    }

    pub(super) fn islem_kapsami(&self, kimlik: IslemId) -> usize {
        kimlik.sirasi() + 1
    }

    pub(super) fn test_kapsami(&self, test_indeksi: usize) -> usize {
        (1usize << 30) + test_indeksi
    }

    pub(super) fn istek_kapsami(&self, satir: usize) -> usize {
        (1usize << 31) + satir
    }
}

pub(super) struct ImzaKaydi {
    pub(super) kimlik: IslemId,
    pub(super) parametre_turleri: Vec<Tur>,
    pub(super) donusler: Vec<Tur>,
    /// Özyinelemeli kullanımlara verilen tür (son birleşimle doğrulanır).
    pub(super) verilen_ozyineleme: Option<Tur>,
}
