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
    pub(super) imzalar: HashMap<String, Imza>,
    pub(super) yapilar: Vec<Yapi>,
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
}

impl Baglam {
    pub(super) fn yeni(islemler: HashMap<String, Islem>, yapilar: Vec<Yapi>) -> Self {
        Self {
            islemler,
            imzalar: HashMap::new(),
            yapilar,
            bekleyen_gorevler: std::collections::HashSet::new(),
            denetim_yigini: Vec::new(),
            dolu_secenekler: std::collections::HashSet::new(),
            basarili_sonuclar: std::collections::HashSet::new(),
            basarisiz_sonuclar: std::collections::HashSet::new(),
            gezilen_koleksiyonlar: std::collections::HashSet::new(),
        }
    }
}

pub(super) struct ImzaKaydi {
    pub(super) ad: String,
    pub(super) parametre_turleri: Vec<Tur>,
    pub(super) donusler: Vec<Tur>,
    /// Özyinelemeli kullanımlara verilen tür (son birleşimle doğrulanır).
    pub(super) verilen_ozyineleme: Option<Tur>,
}
