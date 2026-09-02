# Kritik işlev boyutu ve karmaşıklık eğilimi

<!-- `cd compiler && cargo run --locked --bin islev_egilimi -- --rapor-yaz` üretir. Elle değiştirme. -->

Bu rapor K-144/ADR-041 makine kapısının güncel görünümüdür. Mutlak bir
"iyi işlev N satırdır" kuralı koymaz; incelenmiş tabana göre büyümeyi görünür
kılar. Clippy ölçümü sabit Rust araç zinciriyle üretim `lib` ve `dil` ikilisinde
çalışır; `allow` öznitelikleri `--force-warn` nedeniyle kapıyı atlayamaz.

- İncelenmiş taban: `K-144/ADR-041`
- İzlemeye giriş: en az 80 satır veya bilişsel karmaşıklık 12
- Satır gözden geçirme payı: tabanın %10'u; en az +8, en çok +24
- Karmaşıklık gözden geçirme payı: tabanın %20'si; en az +2, en çok +5

| İşlev | Satır taban→güncel (Δ/pay) | Karmaşıklık taban→güncel (Δ/pay) | Durum |
|---|---:|---:|---|
| `compiler/src/ayristirici.rs::aralik_ayristir#1` | 86→86 (+0/+9) | 3→3 (+0/+2) | SABİT |
| `compiler/src/ayristirici.rs::dondur_ayristir#1` | 94→94 (+0/+10) | 4→4 (+0/+2) | SABİT |
| `compiler/src/ayristirici.rs::gore_ayristir#1` | 103→103 (+0/+11) | 12→12 (+0/+3) | SABİT |
| `compiler/src/ayristirici.rs::islem_ayristir#1` | 145→145 (+0/+15) | 8→8 (+0/+2) | SABİT |
| `compiler/src/ayristirici/cumle.rs::cumle_ayristir#1` | 452→452 (+0/+24) | 35→35 (+0/+5) | SABİT |
| `compiler/src/ayristirici/ifade.rs::kosul_atomu_ic#1` | 185→185 (+0/+19) | 12→12 (+0/+3) | SABİT |
| `compiler/src/ayristirici/ifade.rs::yapili_kalip_ic#1` | 379→379 (+0/+24) | 52→52 (+0/+5) | SABİT |
| `compiler/src/bicimleyici.rs::bicimle#1` | 112→112 (+0/+12) | 22→22 (+0/+5) | SABİT |
| `compiler/src/bicimleyici.rs::satiri_parcala#1` | 73→73 (+0/+8) | 15→15 (+0/+3) | SABİT |
| `compiler/src/cozumleyici.rs::denetle_coklu#1` | 50→50 (+0/+8) | 14→14 (+0/+3) | SABİT |
| `compiler/src/cozumleyici/cagri.rs::cagri_denetle#1` | 296→296 (+0/+24) | 7→7 (+0/+2) | SABİT |
| `compiler/src/cozumleyici/cumle.rs::blok_denetle#1` | 975→975 (+0/+24) | 3→3 (+0/+2) | SABİT |
| `compiler/src/cozumleyici/donus.rs::donusleri_birlestir#1` | 81→81 (+0/+9) | 9→9 (+0/+2) | SABİT |
| `compiler/src/cozumleyici/etki.rs::cumle_bilgisi#1` | 176→176 (+0/+18) | 12→12 (+0/+3) | SABİT |
| `compiler/src/cozumleyici/etki.rs::denetle#1` | 133→133 (+0/+14) | 5→5 (+0/+2) | SABİT |
| `compiler/src/cozumleyici/etki.rs::ifade_bilgisi#1` | 114→114 (+0/+12) | 6→6 (+0/+2) | SABİT |
| `compiler/src/cozumleyici/ifade.rs::ifade_denetle_ic#1` | 877→877 (+0/+24) | 37→37 (+0/+5) | SABİT |
| `compiler/src/cozumleyici/yetkinlik.rs::cumle_bilgisi#1` | 173→173 (+0/+18) | 8→8 (+0/+2) | SABİT |
| `compiler/src/cozumleyici/yetkinlik.rs::ifade_bilgisi#1` | 112→112 (+0/+12) | 6→6 (+0/+2) | SABİT |
| `compiler/src/invariant/cumle.rs::cumleyi_dogrula#1` | 296→296 (+0/+24) | 13→13 (+0/+3) | SABİT |
| `compiler/src/invariant/ifade.rs::ifadenin_faz_bagini_dogrula#1` | 126→126 (+0/+13) | 5→5 (+0/+2) | SABİT |
| `compiler/src/invariant/ifade.rs::ifadeyi_dogrula#1` | 146→146 (+0/+15) | 4→4 (+0/+2) | SABİT |
| `compiler/src/lib.rs::birim_ozeti#1` | 52→52 (+0/+8) | 14→14 (+0/+3) | SABİT |
| `compiler/src/lib.rs::dosyayi_coz#1` | 125→125 (+0/+13) | 12→12 (+0/+3) | SABİT |
| `compiler/src/lib.rs::kaynagi_tanilari_kokenlerle#1` | 84→84 (+0/+9) | 10→10 (+0/+2) | SABİT |
| `compiler/src/lsp.rs::mesaj_isle#1` | 195→195 (+0/+20) | 12→12 (+0/+3) | SABİT |
| `compiler/src/lsp.rs::yeniden_adlandirma_plani#1` | 81→81 (+0/+9) | 12→12 (+0/+3) | SABİT |
| `compiler/src/main.rs::cikar_komutu#1` | 99→99 (+0/+10) | 6→6 (+0/+2) | SABİT |
| `compiler/src/main.rs::ekle_komutu#1` | 135→135 (+0/+14) | 10→10 (+0/+2) | SABİT |
| `compiler/src/main.rs::io_izi_komutu#1` | 108→108 (+0/+11) | 10→10 (+0/+2) | SABİT |
| `compiler/src/main.rs::istek_al#1` | 282→282 (+0/+24) | 25→25 (+0/+5) | SABİT |
| `compiler/src/morfoloji.rs::eki_uydur#1` | 63→63 (+0/+8) | 13→13 (+0/+3) | SABİT |
| `compiler/src/paket.rs::kilit_metni#1` | 89→89 (+0/+9) | 9→9 (+0/+2) | SABİT |
| `compiler/src/paket.rs::ziyaret_et#1` | 186→186 (+0/+19) | 4→4 (+0/+2) | SABİT |
| `compiler/src/paket/uzak.rs::paketleri_hazirla#1` | 169→169 (+0/+17) | 4→4 (+0/+2) | SABİT |
| `compiler/src/proje.rs::bildirimi_oku#1` | 203→203 (+0/+21) | 4→4 (+0/+2) | SABİT |
| `compiler/src/proje.rs::uzak_bagimliliklari_guncelle#1` | 94→94 (+0/+10) | 12→12 (+0/+3) | SABİT |
| `compiler/src/registry.rs::targets_yapisini_dogrula#1` | 95→95 (+0/+10) | 5→5 (+0/+2) | SABİT |
| `compiler/src/registry.rs::zinciri_dogrula#1` | 92→92 (+0/+10) | 1→1 (+0/+2) | SABİT |
| `compiler/src/sozcukleyici.rs::sozcukle#1` | 314→314 (+0/+24) | 29→29 (+0/+5) | SABİT |
| `compiler/src/tedarik.rs::paketle_zamanla#1` | 85→85 (+0/+9) | 2→2 (+0/+2) | SABİT |
| `compiler/src/yorumlayici.rs::gorevleri_calistir#1` | 142→142 (+0/+15) | 2→2 (+0/+2) | SABİT |
| `compiler/src/yorumlayici.rs::json_nesnesi_ayristir#1` | 90→90 (+0/+9) | 8→8 (+0/+2) | SABİT |
| `compiler/src/yorumlayici/cumle.rs::blok_calistir_async#1` | 596→596 (+0/+24) | 1→1 (+0/+2) | SABİT |
| `compiler/src/yorumlayici/ifade.rs::degerlendir_async#1` | 734→734 (+0/+24) | 1→1 (+0/+2) | SABİT |
| `compiler/src/yorumlayici/io_izi.rs::olay_semasini_denetle#1` | 139→139 (+0/+14) | 21→21 (+0/+5) | SABİT |
| `compiler/src/yorumlayici/kaynak.rs::deger_heap_bayti#1` | 101→101 (+0/+11) | 8→8 (+0/+2) | SABİT |
| `compiler/src/yorumlayici/web_istek.rs::web_istegini_calistir#1` | 92→92 (+0/+10) | 9→9 (+0/+2) | SABİT |

Eşik aşımı otomatik bir tasarım hükmü değildir; işi durdurup işlevi bölme veya
gerekçeli yeni tabanı aynı kod incelemesinde kabul etme zorunluluğudur. Düşüşler
tabanı kendiliğinden aşağı çekmez; böylece küçük artışlarla borç gizlenemez.
