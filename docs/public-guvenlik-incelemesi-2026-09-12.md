# Zee public depo güvenlik kontrolü — 12 Eylül 2026

## Kapsam ve sonuç

İnceleme tabanı `bb58b1e0bf3c6d0b69c9c317d8b42ec439be5d65` (public master).
Public görünürlük proje sahibinin bilinçli tercihidir ve korunmuştur.
Rapor gerçek veri hırsızlığının olmadığını kanıtlamaz; sunucu trafiği, silinmiş
Git nesneleri ve erişilemeyen kopyalar incelenmemiştir.

- Taban public master 265 commit içeriyor. Yerel erişilebilir bütün ref'lerin
  genişletilmiş Gitleaks 8.30.1 taraması 296 commit taradı: bulgu yok.
- Güncel dosyalar, commit mesajları ve tag açıklamaları ayrıca tarandı: bulgu yok.
- API ile listelenen mevcut 87 Actions artefaktının tamamı indirildi; açılmış
  içerik taramasında yaklaşık 73,2 MB işlendi: bulgu yok. İçerikler çalıştırılmadı.
- CI günlükleri: `34469873560` (üç platform dil), `34680457200` (tedarik),
  `34676467308` (fuzz). Sekiz adayın tamamı workflow ile doğrulanan
  `wasm_abi/morfoloji-fuzz-<run_id>` önbellek adı; erişim sırrı değil.
- GitHub secret scanning uyarı listesi boş. Secret scanning, push protection
  ve özel güvenlik bildirimi etkin; 12 Eylül yeniden doğrulandı.

## Düzeltmeler

K-184 kaynak commit'i `460473a`; provenance commit'i `e329c82`.

1. `dil iz kaydet` özel atomik yazıcı kullanıyor. Linux/Unix'te `0600`, macOS'ta
   ilk açılışta mirassız boş ACL, Windows'ta ilk açılışta korumalı owner-rights
   DACL uygulanıyor. Eski hedefin geniş izinleri aktarılmıyor. Genel atomik
   yazıcının metadata koruma sözleşmesi ve replay şeması değişmedi.
2. SECURITY.md public görünürlük ve çalışan özel bildirim kanalıyla düzeltildi.
3. `.env`, özel anahtar ve veri dökümü için ignore kuralları eklendi; örnek
   şablonlar için istisnalar var. İzlenen eski dosyalar ignore ile saklanmaz.
4. CI'a sürümü ve arşiv SHA-256'sı sabit Gitleaks eklendi. Tüm erişilebilir
   geçmiş ve güncel dosyalar ayrı taranıyor; çıktı maskeli, bulgu işi durduruyor.
   Yerel `scripts/sir-taramasi.sh --staged` sentetik tokenla reddetme sınavını geçti.
5. Demo panelin sabit parolayla canlıya çıkarılmaması gerektiği açıklandı.

## Korunan sınırlar

İz dosyaları şifreli ya da genel olarak anonim değildir; veritabanı/ağ içeriği
ve kullanıcı girdisi taşıyabilir. Paylaşım için sentetik veriden kayıt üretilir;
izler CI artefaktına konmaz. Önceden alınmış kopyalar geri çekilemez. Gömme
API'sinden alınan metnin saklanması uygulama sorumluluğudur. OS yöneticisi
ve düşman tarafından yönetilen dizinler bu dosya izinleriyle izole edilmez.

E-posta tercihi, Git geçmişi ve başka çalışma klasörlerinin commitlenmemiş
kodları değiştirilmedi. BRC'nin canlı derleyicisi bu depo çalışmasıyla yükseltilmedi.
Tarama bütün kodlanmış/ikili sırları bulma garantisi ve genel güvenlik sertifikası değildir.

## Doğrulama kanıtı

Kaynak/provenance toplamı `e329c823710fea2f1071e76529637d2dbbf3e7e9`:

| Ortam | Tam test matrisi | Sonuç |
|---|---:|---|
| Yerel macOS | 717/717 | Geçti |
| GitHub macOS | 717/717 | Geçti |
| GitHub Linux | 715/715 | Geçti |
| GitHub Windows | 706/706 | Geçti |

Platforma özgü testler nedeniyle toplamlar farklıdır; hiçbir test atlanmadı.
İzin testleri yeni ve mevcut dosyaları, açık Unix umask'ını, symlink reddini,
macOS miras ACL'sini ve Windows korumalı owner-rights DACL'sini kapsar.

[Üç platform CI koşusu](https://github.com/merbay-erp/zee/actions/runs/34690077116),
[başarılı tedarik ve sır taraması](https://github.com/merbay-erp/zee/actions/runs/34690077143).
Yerelde ayrıca Clippy (`-D warnings`), rustfmt, spec drift, mimari/bağımlılık,
core-freeze, regresyon provenance ve `guvenlik-kapisi.sh --surekli` geçti.

Platform kararları: [Apple openx/filesec ACL kullanımı](https://github.com/apple-oss-distributions/copyfile/blob/main/copyfile.c),
[Microsoft owner-rights SID](https://learn.microsoft.com/en-nz/windows-server/identity/ad-ds/manage/understand-security-identifiers).
Tarama kanıtları yerelde `/tmp/zee-*-gitleaks.json` ve `/tmp/zee-security-*.log`
altında tutuldu; ham CI günlükleri veya kişisel metadata bu depoya eklenmedi.

WASM release derlemesi ve gerçek Node hostunda ABI v3 kontrolü ayrıca yerelde
geçti (pointer/boy, UTF-8, sahiplik ve girdi bütçeleri). Rapor yayımlanırken
Linux uzak işindeki gözlemsel performans ölçümü sürüyordu; bütün CI işi
bitmiş/yeşil olarak sunulmaz. Önceki başarılı taban koşusunda bu ölçüm yaklaşık
56 dakika sürmüştür. Yeni bir performans tarihçesi veya ölçüm iddiası eklenmedi.
