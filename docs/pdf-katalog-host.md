# Sabit owned PDF host örneği

K-275/ADR-133, P11/B3 için mevcut Zee JSON çıktısını sabit Python backend'e
veren anonim teknik örnektir. PDF intrinsic/FFI/resmî imzalı eklenti veya
ürün katalog servisinin değiştirilmesi değildir. P11 ve gerçek benimseme
kısmi kalır. Kaynak [golden/pdf-host](../golden/pdf-host/README.md) altındadır.

```sh
dil çalıştır golden/pdf-host/ana.dil > job.json
/path/to/owned/python -I -B scripts/pdf-katalog-host.py \
  --fixture-root golden/pdf-host --job job.json --output /fresh/owned/output
```

İlk komut mevcut Metin sözlüğü/JSON serializer'ı kullanır. İkinci komut
operatörün ayrı sahip olduğu host komutudur; Zee shell başlatmaz. Python'da
exact ReportLab4.4.9/pypdf6.10.0/Pillow12.3.0 gerekir; araç indirme/kurulum
yapmaz, farklı/eksik sürüm fixedJSON/exit2 ile reddedilir. Codex runtime yolu
veya global PATH kurulum zorunluluğu yoktur.

Job8192B, title80/product100/description200 karakter; locale tr/en,
selection all/cover-back, placement after-cover. Font glyph/satır genişliği
kontrollüdür. Kaynak PDF/font/PNG adları ve hashleri sabit; kullanıcı path,
font, HTML, URL, shell veya keyfi PDF veremez. Her asset/output<=1MiB ve en
çok4sayfa. all temel3+üretilen1=4; cover-back temel2+üretilen1=3 üretir.
selected.pdf seçilen eski sayfalar, generated.pdf yeni sayfa, catalog.pdf
bileşimdir. Üç çıktı fresh0700 dizinde exclusive yazılır; çok dosyalı atomik
publication yoktur, hata sonrası kısmi yeni dizin kalabilir. Aynı dizine kör
retry yoktur.

Canvas düz metin çizer; HTML/JS/URI motoru yoktur. Korunan owned sayfa content
operators ve render piksel eşitliği teknik fixture'da doğrulandı; tüm PDF
baytları/metadata/imza koruma sözü yoktur. Byte cap hostile PDF parser sandbox'ı
değildir; yalnız önceden hash'i bilinen fixed kaynaklar kabul edilir. Şifreli/
imzalı PDF, form, annotation, PDF/A/UA, gerçek katalog ölçeği ve hostile aynı
UID/root operatör kapsam dışıdır. Hata/receipt raw body veya private path taşımaz.

DejaVu fontun orijinal telif ve tam Bitstream Vera/Arev notice'ı asset yanında
korunur; [FONT-PROVENANCE](../golden/pdf-host/FONT-PROVENANCE.json) taşınabilir
köken/hash zinciridir. Proje lisansı ertelenmiştir; üçüncü taraf notice'ların
korunması bundan ayrıdır. Ürünle dependency dağıtımı/onayı yapılmış sayılmaz.

Normal Cargo'da stdlib8/POSIX model wrapper'ı PDF paketleri istemez. Explicit
ignored backend8 için ZEE_PDF_BACKEND_PYTHON mutlak, operatörce seçilmiş runtime
ister. Bu host testleri kendi başına native Zee emitter kanıtı değildir.
Final ortak kampanyada temiz S ikilisiyle ana.dil ayrıca çalıştırılır; bounded
job.json eşitliği ve binary/build/helper/asset begin-end pinleri zorunludur,
sonra backend8 yürütülür. İki sayı faz toplamına tekrar eklenmez. Normal
wrapper IO sahibidir; çalıştırmadığı parser/CLI/proje fazları ek_kapsam'a yazılmaz.

B3'ün gerçek 'Zee'den katalog birleştirme çağrısı', resmi signed plugin veya
owned host invocation kararı, gerçek ürün/adoption/P08/platform/finalclean
kapıları açık kalır. [ADR-133](../adr/133-sabit-host-pdf-backend.md) kapsamı ve
neden compiler genişlemesi yapılmadığını açıklar.

Özgün PDF/font notice baytları Git üzerinden de korunur: base.pdf binary,
LICENSE-DejaVu.txt -text olarak işaretlidir. Yalnız özgün lisans dosyasında
blank-at-eol kontrolü kapalıdır; lisans metni biçimlendirilmez. Index blob
SHA-256 ile disk SHA-256 eşliği commit öncesi ayrıca doğrulanır.

## 15 Ekim 2026 — temiz S yerel kanıt güncellemesi

S üzerinde native Zee örneği gerçekten bounded job JSON üretti; equality kontrolünden sonra fixed backend8 koşusu geçti. [Son rapor](urunlesme-final-kabul-20261015.md) exact paket/asset/font notice kimliğini korur. Bu host kabulü Zee içinden resmî signed plugin invocation, keyfi PDF sandbox veya gerçek katalog entegrasyonu değildir.

## 16 Ekim 2026 — a57290f fixed backend kanıtı

[K-298 kabulünde](urunlesme-k298-kabul-20261016.md) aynı fixed host backend'in 8 iç vakasını
kaynakla pinlenmiş mevcut Cargo wrapper'ı doğruladı; saklanan PDF artefaktları
bağımsız incelendi. İç unittest stderr'i dışa aktarılmadı. Bu exact kaynakta
native Zee emitter veya signed PDF yeniden çalıştırılmadı; önceki native
kanıt kendi tarihsel kaynağında kalır. Ayrı K-299 C2 `8178920e03fed782a817d1864b8280fcb80812a3` kaynağında signed PDF9 kabulü daha sonra geçti; bu sonuç a57290f'ye aktarılmaz. Genel P11/gerçek katalog/FFI/yayın açıktır.
