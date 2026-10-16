# dene native web test köprüsü

Unix'te açık operator profili kullanılır. Önce olağan Zee testleri başarılı
olmalıdır. Altı rolün tümünü verin; output henüz var olmamalı ve kanonik
sahipli parent altında olmalıdır:

```sh
dil dene --web-suite /abs/fixtures/suite.json \
  --web-suite-sha256 SUITE_SHA256 --web-fixtures /abs/fixtures \
  --test-python /abs/trusted/python3 \
  --test-postgres-bin /abs/trusted/pg16.15/bin \
  --test-output /abs/owned/new-output /abs/project
```

Suite sözleşmesi [owned web kit](owned-web-test-kiti.md) ile aynıdır.
Python ve PostgreSQL16.15 binary'leri operator tarafından güvenilmiş roller;
suite bunları değiştiremez. Kit kaynakları compiler içine gömülüdür; proje
klasöründe Python aramaz. Credential/provider hesabı veya canlı hizmet
kullanmayın. Fixture SQL güvenilmiş test verisidir, SQL sandbox değildir.

`unit.json`, `runtime/scripts/` ve `results/proof.json` çıktı kökünde korunur.
Başarısız olağan test `native_started:false` bırakır, native suite çalışmaz.
Native rapor dört vaka/19HTTP isteği gibi gerçek sayıları kaydeder; güncel
suite'in sayıları dışında genelleme yapılmaz. Worker başına kaynak raporu
`results/coverage-<case>.json` altındadır. Unit ve native kaynak kapsamı ayrı
sonuçlardır; bağımsız worker sayaçları tek çalıştırmaymış gibi birleştirilmez.

İki PG vakasının cluster kimliği farklı olmalıdır; readonly observer commit1,
request rollback0 ve sonraki taze vaka0 durumunu doğrular. Bu, tüm HTTP
isteklerine ortak transaction rollback desteği değildir.

[ADR-144](../adr/144-dene-native-web-test-koprusu.md),
[kaynak kapsamı](kaynak-kapsami.md) ve
[ADR-142](../adr/142-zee-kaynak-dal-kapsami.md) güven sınırlarını açıklar.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.

## K-304 etki sınırı

K-304 bu köprüye gömülen kitin son iptal/deadline sonucunu korur; Python
bayt değişikliği yeni derlemede yer alır. [Direct kit kabulü](owned-web-test-kiti.md)
GB-113'ü dar kapsamda kapatır. `abdc5cf36b4d55021df5edad08cee3a7ad953ebd` native4 sonucu ayrı
`dene`/source-coverage sahibinin çalıştığı anlamına gelmez. S3 dört vaka/19HTTP
ve admission retleri kendi exact kaynağında kalır; yeni köprü kabulü veya
compiler grammar/API/coverage kapasitesi ileri sürülmez.


## K-305 — Negatif kabul koşucusunda iptal sahipliği

**KAPALI — GB-114 dar test-harness kapsamıyla kapalı.** İlk child öncesindeki
flag-only SIGTERM/INT handler, bekleme/terminal süre kontrolü ve final
cleanup/proof sonucunda gözlenen iptalin failed/nonzero korunması doğrulandı.
Her çocuk original Popen/WNOWAIT sahibi üzerinden kapanır; PID adoption yoktur.
Selector init yerleşimi tek başına kaçak kanıtı değildir; önceki main finally
cleanup'ı tarihsel değerlendirmede korunur. Gerçek bir süreç sızıntısı veya
üretim Rust/kit/Supervisor kusuru ileri sürülmez.

Hazırlık kaydı: Kontrollü hazırlıkta 15 Python metodu ve 27 senaryo geçti; eski sürümde 17 yapısal eksik, iki cleanup hatasında eksik makbuz ve bir terminal süre aşımında yanlış passed ayrıldı. Özgün dış owner eski exit1 ve yeni exit0 ile reaped; gerçek çocuk, OS sinyal teslimi, PG veya ağ kabulü değildir. `K305-PREPARATION.json`
SHA-256: `e445af8d0a82448488bd72d47e1360bba990751db0354f91061a93603c4918b1`.

Bu kontrollü hazırlık gerçek OS sinyal teslimi, PG/HTTP veya process-leak
ölçümü değildir. Mevcut normal CLI sahibinde 15 Python metodu korunur; eski üç
CLI guard ve ayrı ignored bridge sahibi değişmez. Python alt vakaları Cargo
toplamına eklenmez.

Tam kaynak/faz/Clippy/güvenlik kapıları: 15/15 geçti; faz 998/1023 başarılı, 0 başarısız ve 25 ignored; faz komutu 839,411 sn, toplam 1045,134 sn; Clippy ve sürekli güvenlik geçti, 194 kaynak pini korundu, özgün dış owner exit0/reaped.
Temiz `40788a3d0a9f0d43083130bb0f062871861787db` (tree `9c1ea8dffa61d91f5946eff32a9c4cf1c124759e`) üzerinde ayrı mevcut
`web_test_koprusu_unit_native_http_pg_ve_kaynak_kapsami_birlikte` Cargo sahibi
geçti. Unit kaynak kapsamı, ürün paydası14 ile kendi raporundadır:
Beş kaynak kapsamı raporu ham edge kayıtlarından yeniden hesaplandı: unit ürün 4/14; site_snapshot 0/12, panel_session_csrf 8/16, db_committed_and_request_rollback 3/6 ve db_fresh_case_isolation 0/6. Her raporda test paydası0 uygulanamaz; complete=true rapor bütünlüğüdür, yüzde100 kapsam değildir ve farklı raporların paydaları toplanmaz. Dört native vaka: Mevcut tek Cargo sahibi 1 başarılı/0 başarısız/0 ignored (4,63 sn); dört native senaryo, 19 HTTP isteği, altı SQL gözlemi ve iki ayrı fresh PG kümesi doğrulandı; native sürücü 20,169 sn, dış owner 21,076 sn.
Altı admission negatifi: Altı gerçek CLI admission negatifi geçti; installed=true, cancelled=false ve owned_groups_closed=true. Handler restoration ayrı kontrollü hazırlık kanıtıdır; gerçek OS iptal teslimi gözlemi ileri sürülmez.
Özgün Popen/WNOWAIT süreç sahipliği ve boş worker/PG/komut grubu makbuzları doğrulandı; dış owner exit0/reaped. Her PG child için bağımsız normal SIGINT/exit veya private runtime dizini kaldırma gözlemi ileri sürülmez; 205 kaynak ve 17 runtime pini korundu Bağımsız inceleme yedi özgün makbuzu, ham günlükleri, 12 public dosyayı ve beş kapsam raporunu doğruladı; E/k305-final-audits-1/INDEPENDENT-REVIEW.json SHA256=421facd52cc777ae0d34a480a3e971b2df6a6c63789eba64b05bcfb40d622d6e

Unit ve dört worker kapsam raporu ayrı tutulur; eksik dallar paydadan atılmaz,
sayaçlar birleştirilmez ve yüzde100 kapsam iddiası kurulmaz. Native makbuzdaki
handler kurulumu, hazırlığın ayrıca doğruladığı handler restoration testinin
yerine geçmez. Private PG dizininin silinmesi veya çocuk başına normal PG çıkışı
bağımsız gözlem olarak ileri sürülmez.

Son 11 belge/güvenlik kapısı kayıt commit önkoşuludur; gerçek sonuç E/k305-final-gates-1/PROOF.json ve E/k305-final-gates-controller-1.json ile izlenir. Bu metin kapı sonucu yerine geçmez.
K-304 C8 direct kit ve S3 köprü kayıtları kendi exact kaynaklarında korunur;
yeni köprü sonucu bu kanıtların aktarımı değildir. P22/P36, gerçek ürün/P08 ve
platform kapsamı kısmi kalır; C1/P33 reddedilen kapsamı yeniden açılmaz.
[ADR-144](../adr/144-dene-native-web-test-koprusu.md),
[ortak rapor](urunlesme-k298-kabul-20261016.md) exact makbuzları taşır.

Etki incelemesi: grammar/spec/RFC, kullanıcı tanısı, public API, production
katman/bağımlılık, sürüm artefaktı, performans iş yükü veya compiler/LSP kalıcı
durum değişmez. Bu test-harness düzeltmesi semantic `.dil` regression veya yeni
compiler/core beyanı gerektiren üretim değişikliği değildir; önceki RC/soak/IDE
tarihçesi yeni kaynak adına kabul kaydı olarak kullanılmaz.
