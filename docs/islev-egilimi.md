# Kritik işlev boyutu ve karmaşıklık eğilimi

<!-- `cd compiler && cargo run --locked --bin islev_egilimi -- --rapor-yaz` üretir. Elle değiştirme. -->

Bu rapor K-144/ADR-041 makine kapısının güncel görünümüdür. Mutlak bir
"iyi işlev N satırdır" kuralı koymaz; incelenmiş tabana göre büyümeyi görünür
kılar. Clippy ölçümü sabit Rust araç zinciriyle üretim `lib` ve `dil` ikilisinde
çalışır; `allow` öznitelikleri `--force-warn` nedeniyle kapıyı atlayamaz.

- İncelenmiş taban: `K-185/K-186/K-187 inceleme: web_istegini_calistir dogrulanmis oturum ortami icin +15 satir; veritabani_bildirimini_al (K-186) ve migration_ust_seviye_kelimeleri (K-187) yeni kritik islev olarak tabana alindi; K-190 elmas birlesimi elmas_tanimlarini_kat yardimcisina ayrildi; K-240 dar inceleme: K-232 lowering/MIME/runtime ve K-234 config/CLI dispatcher kapsaminda yalniz 5 islev yeniden kabul edildi; sorumluluk/girdi/etki sinirlari ve mevcut olumlu-olumsuz testler depo disi inceleme notunda; global buyume payi ayni`
- İzlemeye giriş: en az 80 satır veya bilişsel karmaşıklık 12
- Satır gözden geçirme payı: tabanın %10'u; en az +8, en çok +24
- Karmaşıklık gözden geçirme payı: tabanın %20'si; en az +2, en çok +5

| İşlev | Satır taban→güncel (Δ/pay) | Karmaşıklık taban→güncel (Δ/pay) | Durum |
|---|---:|---:|---|
| `compiler/src/ayristirici.rs::aralik_ayristir#1` | 86→86 (+0/+9) | 3→3 (+0/+2) | SABİT |
| `compiler/src/ayristirici.rs::dondur_ayristir#1` | 105→105 (+0/+11) | 4→4 (+0/+2) | SABİT |
| `compiler/src/ayristirici.rs::gore_ayristir#1` | 109→109 (+0/+11) | 12→12 (+0/+3) | SABİT |
| `compiler/src/ayristirici.rs::islem_ayristir#1` | 146→146 (+0/+15) | 8→8 (+0/+2) | SABİT |
| `compiler/src/ayristirici.rs::kullanilan_birimler#1` | 46→46 (+0/+8) | 12→12 (+0/+3) | SABİT |
| `compiler/src/ayristirici.rs::yapi_ayristir#1` | 86→86 (+0/+9) | 7→7 (+0/+2) | SABİT |
| `compiler/src/ayristirici/cumle.rs::cumle_ayristir#1` | 485→496 (+11/+24) | 33→34 (+1/+5) | PAY İÇİNDE |
| `compiler/src/ayristirici/cumle/gorunum.rs::gorunum_dugumu_kur#1` | 91→91 (+0/+10) | 2→2 (+0/+2) | SABİT |
| `compiler/src/ayristirici/ifade.rs::kosul_atomu_ic#1` | 183→186 (+3/+19) | 12→12 (+0/+3) | PAY İÇİNDE |
| `compiler/src/ayristirici/ifade.rs::yapili_kalip_ic#1` | 427→424 (-3/+24) | 54→54 (+0/+5) | İYİLEŞTİ |
| `compiler/src/ayristirici/kimlik_iletisim.rs::yapili_kalip#1` | 92→92 (+0/+10) | 0→0 (+0/+2) | SABİT |
| `compiler/src/bicimleyici.rs::bicimle#1` | 112→112 (+0/+12) | 22→22 (+0/+5) | SABİT |
| `compiler/src/bicimleyici.rs::satiri_parcala#1` | 73→73 (+0/+8) | 15→15 (+0/+3) | SABİT |
| `compiler/src/cli/ayar.rs::cozumle#1` | 94→94 (+0/+10) | 8→8 (+0/+2) | SABİT |
| `compiler/src/cozumleyici.rs::denetle_coklu_hatali_tanimlarla#1` | 54→54 (+0/+8) | 15→15 (+0/+3) | SABİT |
| `compiler/src/cozumleyici/cagri.rs::cagri_denetle#1` | 301→301 (+0/+24) | 7→7 (+0/+2) | SABİT |
| `compiler/src/cozumleyici/cumle.rs::blok_denetle#1` | 1121→1123 (+2/+24) | 3→3 (+0/+2) | PAY İÇİNDE |
| `compiler/src/cozumleyici/donus.rs::donusleri_birlestir#1` | 87→87 (+0/+9) | 9→9 (+0/+2) | SABİT |
| `compiler/src/cozumleyici/etki.rs::cumle_bilgisi#1` | 178→183 (+5/+18) | 12→13 (+1/+3) | PAY İÇİNDE |
| `compiler/src/cozumleyici/etki.rs::denetle#1` | 133→133 (+0/+14) | 5→5 (+0/+2) | SABİT |
| `compiler/src/cozumleyici/etki.rs::ifade_bilgisi#1` | 119→120 (+1/+12) | 7→7 (+0/+2) | PAY İÇİNDE |
| `compiler/src/cozumleyici/ifade.rs::ifade_denetle_ic#1` | 964→966 (+2/+24) | 37→37 (+0/+5) | PAY İÇİNDE |
| `compiler/src/cozumleyici/yetkinlik.rs::cumle_bilgisi#1` | 173→176 (+3/+18) | 8→8 (+0/+2) | PAY İÇİNDE |
| `compiler/src/cozumleyici/yetkinlik.rs::ifade_bilgisi#1` | 112→112 (+0/+12) | 6→6 (+0/+2) | SABİT |
| `compiler/src/eposta/mime.rs::ileti_kur#1` | 93→93 (+0/+10) | 8→8 (+0/+2) | SABİT |
| `compiler/src/gorsel.rs::kucult#1` | 82→82 (+0/+9) | 2→2 (+0/+2) | SABİT |
| `compiler/src/invariant/cumle.rs::cumleyi_dogrula#1` | 296→306 (+10/+24) | 13→14 (+1/+3) | PAY İÇİNDE |
| `compiler/src/invariant/ifade.rs::ifadenin_faz_bagini_dogrula#1` | 126→126 (+0/+13) | 5→5 (+0/+2) | SABİT |
| `compiler/src/invariant/ifade.rs::ifadeyi_dogrula#1` | 142→142 (+0/+15) | 4→4 (+0/+2) | SABİT |
| `compiler/src/lib.rs::birim_ozeti#1` | 52→52 (+0/+8) | 14→14 (+0/+3) | SABİT |
| `compiler/src/lib.rs::dosyayi_coz#1` | 109→114 (+5/+11) | 6→7 (+1/+2) | PAY İÇİNDE |
| `compiler/src/lib.rs::kaynagi_tanilari_kokenlerle#1` | 100→105 (+5/+10) | 10→11 (+1/+2) | PAY İÇİNDE |
| `compiler/src/lsp.rs::mesaj_isle#1` | 195→197 (+2/+20) | 12→12 (+0/+3) | PAY İÇİNDE |
| `compiler/src/lsp.rs::yeniden_adlandirma_plani#1` | 81→81 (+0/+9) | 12→12 (+0/+3) | SABİT |
| `compiler/src/main.rs::cikar_komutu#1` | 99→99 (+0/+10) | 6→6 (+0/+2) | SABİT |
| `compiler/src/main.rs::ekle_komutu#1` | 135→135 (+0/+14) | 10→10 (+0/+2) | SABİT |
| `compiler/src/main.rs::govde#1` | 103→103 (+0/+11) | 10→10 (+0/+2) | SABİT |
| `compiler/src/main.rs::io_izi_komutu#1` | 108→108 (+0/+11) | 10→10 (+0/+2) | SABİT |
| `compiler/src/main.rs::istek_govdesini_oku#1` | 79→79 (+0/+8) | 12→12 (+0/+3) | SABİT |
| `compiler/src/main.rs::web_istegini_baslat#1` | 85→85 (+0/+9) | 6→6 (+0/+2) | SABİT |
| `compiler/src/main.rs::web_modunu_ayikla#1` | 88→89 (+1/+9) | 4→4 (+0/+2) | PAY İÇİNDE |
| `compiler/src/morfoloji.rs::eki_uydur#1` | 63→63 (+0/+8) | 13→13 (+0/+3) | SABİT |
| `compiler/src/paket.rs::kilit_metni#1` | 89→89 (+0/+9) | 9→9 (+0/+2) | SABİT |
| `compiler/src/paket.rs::ziyaret_et#1` | 186→186 (+0/+19) | 4→4 (+0/+2) | SABİT |
| `compiler/src/paket/uzak.rs::paketleri_hazirla#1` | 171→171 (+0/+18) | 4→4 (+0/+2) | SABİT |
| `compiler/src/postgresql.rs::migration_ust_seviye_kelimeleri#1` | 77→77 (+0/+8) | 19→19 (+0/+4) | SABİT |
| `compiler/src/proje.rs::bildirimi_oku#1` | 209→218 (+9/+21) | 4→4 (+0/+2) | PAY İÇİNDE |
| `compiler/src/proje.rs::uzak_bagimliliklari_guncelle#1` | 94→94 (+0/+10) | 12→12 (+0/+3) | SABİT |
| `compiler/src/proje.rs::veritabani_bildirimini_al#1` | 92→92 (+0/+10) | 12→12 (+0/+3) | SABİT |
| `compiler/src/registry.rs::targets_yapisini_dogrula#1` | 95→95 (+0/+10) | 5→5 (+0/+2) | SABİT |
| `compiler/src/registry.rs::zinciri_dogrula#1` | 92→92 (+0/+10) | 1→1 (+0/+2) | SABİT |
| `compiler/src/sozcukleyici.rs::sozcukle#1` | 320→320 (+0/+24) | 29→29 (+0/+5) | SABİT |
| `compiler/src/yorumlayici.rs::gorevleri_calistir#1` | 136→136 (+0/+14) | 2→2 (+0/+2) | SABİT |
| `compiler/src/yorumlayici.rs::json_nesnesi_ayristir#1` | 96→96 (+0/+10) | 8→8 (+0/+2) | SABİT |
| `compiler/src/yorumlayici/cumle.rs::blok_calistir_async#1` | 815→827 (+12/+24) | 1→1 (+0/+2) | PAY İÇİNDE |
| `compiler/src/yorumlayici/ifade.rs::degerlendir_async#1` | 850→834 (-16/+24) | 1→1 (+0/+2) | İYİLEŞTİ |
| `compiler/src/yorumlayici/io_izi.rs::olay_semasini_denetle#1` | 149→152 (+3/+15) | 20→20 (+0/+4) | PAY İÇİNDE |
| `compiler/src/yorumlayici/kaynak.rs::deger_heap_bayti#1` | 101→101 (+0/+11) | 8→8 (+0/+2) | SABİT |
| `compiler/src/yorumlayici/kimlik_iletisim.rs::intrinsic_degerlendir#1` | 125→125 (+0/+13) | 4→4 (+0/+2) | SABİT |
| `compiler/src/yorumlayici/web_istek.rs::web_istegini_calistir#1` | 113→113 (+0/+12) | 9→9 (+0/+2) | SABİT |

Eşik aşımı otomatik bir tasarım hükmü değildir; işi durdurup işlevi bölme veya
gerekçeli yeni tabanı aynı kod incelemesinde kabul etme zorunluluğudur. Düşüşler
tabanı kendiliğinden aşağı çekmez; böylece küçük artışlarla borç gizlenemez.
