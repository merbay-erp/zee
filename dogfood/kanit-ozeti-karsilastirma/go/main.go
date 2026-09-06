// K-165 karşılaştırma: dogfood/kanit-ozeti Zee ürününün Go eşdeğeri.
// Aynı dokuz kayıt defterini okur, aynı Markdown sayfasını bayt bayt üretir.
// Kullanım: kanit-ozeti-go <depo-kökü> → sayfa stdout'a yazılır.
package main

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"unicode"
)

func satirlar(kok, goreli string) []string {
	veri, err := os.ReadFile(filepath.Join(kok, goreli))
	if err != nil {
		panic(fmt.Sprintf("%s okunamadı: %v", goreli, err))
	}
	metin := strings.TrimSuffix(string(veri), "\n")
	if metin == "" {
		return nil
	}
	return strings.Split(metin, "\n")
}

func veriSatirlariniAyikla(s []string) []string {
	var secilenler []string
	for _, satir := range s {
		temiz := strings.TrimSpace(satir)
		if temiz == "" || strings.HasPrefix(temiz, "#") {
			continue
		}
		secilenler = append(secilenler, satir)
	}
	return secilenler
}

func baslikSatiriniBul(s []string) []string {
	baslik := ""
	for _, satir := range s {
		if strings.HasPrefix(satir, "# ") && strings.Contains(satir, "\t") {
			baslik = strings.ReplaceAll(satir, "# ", "")
		}
	}
	if baslik == "" {
		return nil
	}
	return strings.Split(baslik, "\t")
}

func sutunuTopla(s []string, sutun string) []string {
	basliklar := baslikSatiriniBul(s)
	indeks := -1
	for i, b := range basliklar {
		if b == sutun {
			indeks = i
			break
		}
	}
	var degerler []string
	for _, veri := range veriSatirlariniAyikla(s) {
		alanlar := strings.Split(veri, "\t")
		if indeks >= 0 && indeks < len(alanlar) {
			degerler = append(degerler, alanlar[indeks])
		} else {
			degerler = append(degerler, "")
		}
	}
	return degerler
}

func ciftleriBirlestir(birinciler, ikinciler []string, ayirici string) []string {
	var ciftler []string
	for i := range birinciler {
		if i < len(ikinciler) {
			ciftler = append(ciftler, birinciler[i]+ayirici+ikinciler[i])
		}
	}
	return ciftler
}

const alfabe = "abcçdefgğhıijklmnoöprsştuüvyz"

func turkceHarfSirasi(k rune) (int, int) {
	var kucuk rune
	switch k {
	case 'İ':
		kucuk = 'i'
	case 'I':
		kucuk = 'ı'
	default:
		kucuk = unicode.ToLower(k)
	}
	for i, a := range []rune(alfabe) {
		if a == kucuk {
			return 0, i
		}
	}
	return 1, int(k)
}

func turkceKucuk(a, b string) bool {
	ra, rb := []rune(a), []rune(b)
	for i := 0; i < len(ra) && i < len(rb); i++ {
		g1, s1 := turkceHarfSirasi(ra[i])
		g2, s2 := turkceHarfSirasi(rb[i])
		if g1 != g2 {
			return g1 < g2
		}
		if s1 != s2 {
			return s1 < s2
		}
	}
	return len(ra) < len(rb)
}

type sayaclar map[string]int

func say(degerler []string) sayaclar {
	s := sayaclar{}
	for _, d := range degerler {
		s[d]++
	}
	return s
}

func (s sayaclar) siraliAnahtarlar() []string {
	anahtarlar := make([]string, 0, len(s))
	for k := range s {
		anahtarlar = append(anahtarlar, k)
	}
	sort.SliceStable(anahtarlar, func(i, j int) bool { return turkceKucuk(anahtarlar[i], anahtarlar[j]) })
	return anahtarlar
}

func yuzde(bolunen, bolen int) int {
	if bolen == 0 {
		return 0
	}
	return bolunen * 100 / bolen
}

func hucre(h ...string) string { return "| " + strings.Join(h, " | ") + " |" }

func tabloBasligi(sutunlar ...string) []string {
	cizgi := make([]string, len(sutunlar))
	for i := range cizgi {
		cizgi[i] = "---"
	}
	return []string{hucre(sutunlar...), hucre(cizgi...)}
}

func sayimTablosu(s sayaclar, etiket, olcu string) []string {
	cikti := tabloBasligi(etiket, olcu)
	for _, k := range s.siraliAnahtarlar() {
		cikti = append(cikti, hucre(k, strconv.Itoa(s[k])))
	}
	return cikti
}

func bolumBasligi(ad, yol string) []string {
	return []string{"", "## " + ad, "", "Kaynak: `" + yol + "`", ""}
}

func i(n int) string { return strconv.Itoa(n) }

func regresyonBolumu(s []string) []string {
	b := bolumBasligi("Semantic regresyon korpusu", "regression/v2.tsv")
	fazlar := sutunuTopla(s, "faz")
	b = append(b, sayimTablosu(say(fazlar), "Faz", "Vaka")...)
	b = append(b, "")
	b = append(b, sayimTablosu(say(sutunuTopla(s, "tani")), "Tanı", "Vaka")...)
	tam := 0
	for _, d := range sutunuTopla(s, "fixed_by") {
		if len([]rune(d)) == 40 {
			tam++
		}
	}
	b = append(b, "", fmt.Sprintf("Toplam %d vaka; `-` tanısız çalışma/eşzamanlılık vakasıdır. Tam 40 karakterlik `fixed_by` commit'i taşıyan vaka: %d/%d.", len(fazlar), tam, len(fazlar)))
	return b
}

func guvenlikBolumu(s []string) []string {
	b := bolumBasligi("Güvenlik bulguları", "docs/guvenlik-bulgulari-v1.tsv")
	onemler := sutunuTopla(s, "onem")
	durumlar := sutunuTopla(s, "durum")
	sc := say(ciftleriBirlestir(onemler, durumlar, "|"))
	b = append(b, tabloBasligi("Önem", "Kapalı", "Kabul", "Açık", "Toplam")...)
	acikAgir := 0
	for _, onem := range []string{"kritik", "yuksek", "orta", "dusuk"} {
		kapali, kabul, acik := sc[onem+"|kapali"], sc[onem+"|kabul"], sc[onem+"|acik"]
		if onem == "kritik" || onem == "yuksek" {
			acikAgir += acik
		}
		b = append(b, hucre(onem, i(kapali), i(kabul), i(acik), i(kapali+kabul+acik)))
	}
	b = append(b, "")
	if acikAgir == 0 {
		b = append(b, "Kapı: açık kritik/yüksek bulgu 0 → **GEÇTİ** (ADR-066 sürekli kapı koşulu).")
	} else {
		b = append(b, fmt.Sprintf("Kapı: açık kritik/yüksek bulgu %d → **KALDI**.", acikAgir))
	}
	b = append(b, fmt.Sprintf("Toplam bulgu: %d.", len(onemler)))
	return b
}

func specBolumu(s []string) []string {
	b := bolumBasligi("Spec maddeleri", "docs/spec-madde-kaniti-v1.tsv")
	maddeler := sutunuTopla(s, "madde")
	durumlar := sutunuTopla(s, "durum")
	dosyalar := make([]string, len(maddeler))
	for k, m := range maddeler {
		dosyalar[k] = strings.Split(m, "#")[0]
	}
	sc := say(ciftleriBirlestir(dosyalar, durumlar, "|"))
	ds := say(dosyalar)
	b = append(b, tabloBasligi("Bölüm", "Kanıtlı", "Kısmi", "Açık", "Toplam", "Kanıt %")...)
	tk, tp, ta := 0, 0, 0
	for _, d := range ds.siraliAnahtarlar() {
		kanitli, kismi, acik := sc[d+"|kanitli"], sc[d+"|kismi"], sc[d+"|acik"]
		toplam := kanitli + kismi + acik
		tk += kanitli
		tp += kismi
		ta += acik
		b = append(b, hucre(d, i(kanitli), i(kismi), i(acik), i(toplam), i(yuzde(kanitli, toplam))))
	}
	genel := tk + tp + ta
	b = append(b, hucre("**Toplam**", i(tk), i(tp), i(ta), i(genel), i(yuzde(tk, genel))))
	return b
}

func dogfoodBolumu(kayitlar, vakalar []string) []string {
	b := bolumBasligi("Dogfood ürünleri ve korpusu", "docs/dogfood-projeleri-v1.tsv + dogfood/korpus-v1.tsv")
	urunler := sutunuTopla(kayitlar, "urun")
	kokler := sutunuTopla(kayitlar, "kok")
	durumlar := sutunuTopla(kayitlar, "durum")
	cizgiler := ciftleriBirlestir(ciftleriBirlestir(urunler, durumlar, " ("), kokler, ") | ")
	b = append(b, tabloBasligi("Ürün (durum)", "Kök")...)
	for _, c := range cizgiler {
		b = append(b, "| "+c+" |")
	}
	b = append(b, "")
	vu := sutunuTopla(vakalar, "urun")
	ks := say(ciftleriBirlestir(vu, sutunuTopla(vakalar, "kip"), "|"))
	bs := say(ciftleriBirlestir(vu, sutunuTopla(vakalar, "beklenti"), "|"))
	us := say(vu)
	b = append(b, tabloBasligi("Ürün", "denetle", "calistir", "proje", "basarili", "basarisiz", "Toplam")...)
	for _, u := range us.siraliAnahtarlar() {
		b = append(b, hucre(u, i(ks[u+"|denetle"]), i(ks[u+"|calistir"]), i(ks[u+"|proje"]), i(bs[u+"|basarili"]), i(bs[u+"|basarisiz"]), i(us[u])))
	}
	b = append(b, "", fmt.Sprintf("Toplam korpus vakası: %d.", len(vu)))
	return b
}

func deprecationBolumu(s []string) []string {
	b := bolumBasligi("Deprecation kayıtları", "docs/deprecation-kayitlari-v1.tsv")
	durumlar := sutunuTopla(s, "durum")
	b = append(b, sayimTablosu(say(durumlar), "Durum", "Kayıt")...)
	b = append(b, "")
	ciftler := ciftleriBirlestir(sutunuTopla(s, "yuzey"), sutunuTopla(s, "sinif"), " / ")
	b = append(b, sayimTablosu(say(ciftler), "Yüzey / sınıf", "Kayıt")...)
	b = append(b, "", fmt.Sprintf("Toplam kayıt: %d.", len(durumlar)))
	return b
}

func soakBolumu(s []string) []string {
	b := bolumBasligi("Uzun soak tarihçesi", "docs/soak-gecmisi-v1.tsv")
	sonuclar := sutunuTopla(s, "sonuc")
	b = append(b, sayimTablosu(say(sonuclar), "Sonuç", "Koşu")...)
	b = append(b, "")
	if len(sonuclar) == 0 {
		return append(b, "Henüz soak koşusu yok.")
	}
	son := func(sutun string) string {
		d := sutunuTopla(s, sutun)
		return d[len(d)-1]
	}
	return append(b, fmt.Sprintf("Son koşu: `%s` (%s, %s, %s sn): derleyici %%%s, dillsp %%%s RSS büyümesi → %s.",
		son("git_sha"), son("tarih"), son("platform"), son("sure_sn"), son("derleyici_buyume_yuzde"), son("dillsp_buyume_yuzde"), sonuclar[len(sonuclar)-1]))
}

func beyanBolumu(degisiklikler, donmalar []string) []string {
	b := bolumBasligi("Compiler değişiklik ve core freeze beyanları", "docs/compiler-degisiklik-beyanlari-v1.tsv + docs/core-freeze-beyanlari-v1.tsv")
	ds := sutunuTopla(degisiklikler, "sinif")
	b = append(b, sayimTablosu(say(ds), "Değişiklik sınıfı", "Beyan")...)
	b = append(b, "")
	fs := sutunuTopla(donmalar, "sinif")
	b = append(b, sayimTablosu(say(fs), "Freeze sınıfı", "Beyan")...)
	b = append(b, "", fmt.Sprintf("Toplam: %d compiler değişiklik beyanı, %d core freeze beyanı.", len(ds), len(fs)))
	return b
}

func sayfayiUret(kok string) string {
	cikti := []string{
		"# Kanıt özeti",
		"",
		"Bu sayfa `dogfood/kanit-ozeti` Zee ürünü tarafından depo kayıt defterlerinden üretilir (K-164/ADR-072) ve elle düzenlenmez. Yenilemek için: `cd compiler && cargo run --locked -- çalıştır ../dogfood/kanit-ozeti/kaynak/ana.dil`. CI aynı komutu koşar ve bu dosya bayatsa kırılır.",
	}
	cikti = append(cikti, regresyonBolumu(satirlar(kok, "regression/v2.tsv"))...)
	cikti = append(cikti, guvenlikBolumu(satirlar(kok, "docs/guvenlik-bulgulari-v1.tsv"))...)
	cikti = append(cikti, specBolumu(satirlar(kok, "docs/spec-madde-kaniti-v1.tsv"))...)
	cikti = append(cikti, dogfoodBolumu(satirlar(kok, "docs/dogfood-projeleri-v1.tsv"), satirlar(kok, "dogfood/korpus-v1.tsv"))...)
	cikti = append(cikti, deprecationBolumu(satirlar(kok, "docs/deprecation-kayitlari-v1.tsv"))...)
	cikti = append(cikti, soakBolumu(satirlar(kok, "docs/soak-gecmisi-v1.tsv"))...)
	cikti = append(cikti, beyanBolumu(satirlar(kok, "docs/compiler-degisiklik-beyanlari-v1.tsv"), satirlar(kok, "docs/core-freeze-beyanlari-v1.tsv"))...)
	return strings.Join(cikti, "\n") + "\n"
}

func main() {
	kok := "."
	if len(os.Args) > 1 {
		kok = os.Args[1]
	}
	fmt.Print(sayfayiUret(kok))
}
