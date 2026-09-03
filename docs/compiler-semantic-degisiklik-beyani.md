# Compiler semantic değişiklik beyanı

`compiler/src` altındaki her commit, başlığından bağımsız olarak semantic
etkisini açıkça bildirir. Kapının makine-okunur kaynağı
[`compiler-degisiklik-beyanlari-v1.tsv`](compiler-degisiklik-beyanlari-v1.tsv),
mimari kararı [ADR-053](../adr/053-compiler-semantic-degisiklik-beyani.md)'tür.

## Satır sözleşmesi

Her veri satırı dört sekmeli alan taşır:

1. kaynak değişikliğinin tam 40 haneli Git SHA'sı;
2. `semantic-bugfix`, `semantic-change` veya `maintenance` sınıfı;
3. kanıt;
4. en az 40 karakterlik, incelemede anlamlı gerekçe.

| Sınıf | Kanıt | Anlam |
|---|---|---|
| `semantic-bugfix` | `regression/v2.tsv` vaka kimliği | Vakanın `fixed_by` alanı commit SHA'sıyla exact eşleşir. |
| `semantic-change` | `spec/`, `rfcs/` veya `adr/` Markdown yolu | Kullanıcıya görünen semantic değişiklik normatif karara bağlıdır. |
| `maintenance` | `-` | Semantic davranış değişmemiştir; gerekçe bunun nedenini açıklar. |

## İki commitlik akış

1. Kaynak düzeltmesini testleri ve ilgili davranış belgeleriyle commit et.
2. Tam SHA'yı al.
3. Bug fix ise minimal regression fixture'ını ve `fixed_by` satırını ekle.
4. Beyan TSV'sine SHA+sınıf+kanıt+gerekçe satırını ekle ve ikinci commit'i al.
5. Koruk ile tam faz matrisini çalıştır.

```bash
bash scripts/semantic-regresyon-korugu.sh HEAD~2
cd compiler
cargo run --locked --bin faz_test_matrisi -- --denetle \
  --rapor target/faz-test-matrisi.md
```

`maintenance` kolay kaçış değildir: her commit yine exact ve tekil kayıttır;
gerekçe review edilir. Yanlış sınıflandırma, kod incelemesinde semantic bug
sayılır. Koruk; eksik/yinelenen satırı, bozuk kanıtı ve kaynak commit'i olmayan
sahipsiz beyanı otomatik reddeder.
