package main

import (
	"strings"
	"testing"
)

func TestBaslikSonSekmeliYorumSatiridir(t *testing.T) {
	b := baslikSatiriniBul([]string{"# zee-x-1", "# enforcement_parent\tabc", "# kimlik\tdurum", "GB-1\tacik"})
	if strings.Join(b, ",") != "kimlik,durum" {
		t.Fatalf("başlık: %v", b)
	}
}

func TestSutunToplamaVeriSirasiniKorur(t *testing.T) {
	d := sutunuTopla([]string{"# kimlik\tdurum", "GB-1\tacik", "", "GB-2"}, "durum")
	if strings.Join(d, ",") != "acik," {
		t.Fatalf("değerler: %v", d)
	}
}

func TestTurkAlfabesiSirasi(t *testing.T) {
	a := say([]string{"kapali", "acik", "çok", "-"}).siraliAnahtarlar()
	if strings.Join(a, ",") != "acik,çok,kapali,-" {
		t.Fatalf("sıra: %v", a)
	}
}

func TestGuvenlikKapisiAcikAgirBulgudaKalir(t *testing.T) {
	b := strings.Join(guvenlikBolumu([]string{"# kimlik\tonem\tdurum", "GB-1\tyuksek\tacik"}), "\n")
	if !strings.Contains(b, "Kapı: açık kritik/yüksek bulgu 1 → **KALDI**.") {
		t.Fatal(b)
	}
}

func TestSpecToplamSatiri(t *testing.T) {
	b := specBolumu([]string{"# madde\tdurum", "spec/02-dizim.md#a\tkanitli", "spec/02-dizim.md#b\tkismi", "spec/01-sozcukleme.md#c\tacik"})
	if b[len(b)-1] != "| **Toplam** | 1 | 1 | 1 | 3 | 33 |" {
		t.Fatal(b[len(b)-1])
	}
}

func TestBosSoakTarihcesi(t *testing.T) {
	b := soakBolumu([]string{"# git_sha\tsonuc"})
	if b[len(b)-1] != "Henüz soak koşusu yok." {
		t.Fatal(b[len(b)-1])
	}
}
