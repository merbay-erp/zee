# 27 — Uyumluluk ve sürümleme

Normatif kaynak: RFC-0028, ADR-064, ADR-025, ADR-058. Durum: **TANIMLI —
K-167**.

## Sürüm kimliği

- Derleyici sürümü SemVer `X.Y.Z`dir; etiket sonrası çalışma ağacı
  `X.(Y+1).0-dev` serisi taşımak **ZORUNDADIR** ve `-dev` aynı sayılı yayından
  önce sıralanır.
- `dil sürüm`, LSP `serverInfo.version` ve yayın SBOM'u aynı kimliği basmak
  **ZORUNDADIR**; yayımlanmış kimlik yeniden kullanılamaz (**YASAK**).

## Dondurulmuş yüzeyler

Kalıp kelimeleri, koşul yüklemleri, CLI komutları, tanı kodları, `proje.kilit`
v3, `ZEEZEP` v1, `zee-yayin-v1` zarfı, `zee-tr-1`/`zee-io-1`/`zee-esz-1`
profilleri, WASM ABI v3 ve `dil::api::v1` uyumluluk yüzeyidir.

- Her yüzey öğesi `dil-yuzeyi-v1.tsv` (kelime ve biçimler) ya da kendi
  sürümlü kaydında (tanı, profil, ABI, API) durum ve giriş sürümüyle
  listelenmek **ZORUNDADIR**.
- Kayıtsız öğe eklemek veya kayıtlı öğeyi kayıtsız kaldırmak **YASAKTIR**.

## Deprecation

- Kaldırma ve yeniden adlandırma `docs/deprecation-kayitlari-v1.tsv`
  içinde ardışık `DEP-NNN` kimliği, duyuru sürümü, kaldırma sürümü, göç yolu
  ve karar belgesiyle **ZORUNLU** olarak kayıtlıdır.
- Desteklenen yüzeyde kaldırma sürümü duyurudan en az bir alt sürüm serisi
  sonradır (**ZORUNLU**). Tanı mezar taşı ve `ic` yüzey süre şartı taşımaz;
  `ic` yüzey uyumluluk sözü değildir.
- Kaldırılan kelime fixture'da `kaldirildi` mezar taşı olarak kalır ve başka
  anlamda yeniden kullanılamaz (**YASAK**). Ayrılmış her tanı kodu kayıt
  taşır.
- Kaldırılan biçim sessizce kabul edilmez: derleyici tanı üretir ve önerisi
  göç yolunu adlandırır; regresyon korpusu bunu kalıcı kanıtlar
  (**TANIMLI**).

## Ana sürüm ve edition

1.0 sonrasında dondurulmuş yüzeylerde kırıcı değişiklik yalnız yeni ana sürüm
ya da `proje.dil` içinde açık edition seçimiyle olur; edition alanının
sözdizimi **AÇIK**tır. Morfoloji profili değişimi spec/13 gereği yeni profil
kimliği ister.

## Kanıt

- `compiler/tests/uyumluluk_testi.rs`: fixture↔kaynak birebirliği, exact
  biçim/profil/ABI/API kayıtları, deprecation şeması ve süre kuralı, tanı
  mezar taşı çaprazı, kaldırılan iç modülün yokluğu, sürüm kimliği ve sentetik
  bozuk girdilerin reddi.
- `compiler/tests/tani_kimligi_testi.rs`, `morfoloji_testi.rs`,
  `public_api_testi.rs` ve `playground_testi.rs` kendi yüzeylerini korur.
