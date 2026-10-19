// K-320: zee kullanılabilirlik oturum yürütücüsü (docs/usability-kiti.md, protokol K016-1).
// Veriler yalnız bu tarayıcıda (localStorage) tutulur; sunucuya hiçbir şey gönderilmez.
// Çıktı docs/usability-sonuclari/katilimci-sablonu.md biçimindeki anonim Markdown dosyasıdır.
(function () {
  "use strict";

  var KIMLIKLER = ["C01", "C02", "C03", "C04", "C05", "C06", "C07", "C08", "C09", "C10", "P01", "P02", "P03", "P04", "P05"];
  // Dengeli atama: her kart sırası beşer kişiye; profesyoneller üç sıraya dağılır.
  var SIRALAR = {
    C01: "A-B-C", C02: "B-C-A", C03: "C-A-B", C04: "A-B-C", C05: "B-C-A", C06: "C-A-B", C07: "A-B-C", C08: "B-C-A",
    C09: "C-A-B", C10: "A-B-C", P01: "B-C-A", P02: "C-A-B", P03: "A-B-C", P04: "B-C-A", P05: "C-A-B"
  };
  var KARTLAR = {
    A: { satir: "ortalama notlar için ortalamayı hesapla olsun", kural: "Sonuç adı başta, çağrı ortada; satır `olsun` ile biter." },
    B: { satir: "notlar için ortalamayı hesapla, sonucu ortalama olsun", kural: "Önce çağrı yazılır; `sonucu ... olsun` bölümü dönen değere ad verir." },
    C: { satir: "ortalama (notlar için ortalamayı hesapla) olsun", kural: "Sonuç adı başta; çağrı parantezin içinde, satır `olsun` ile biter." }
  };
  var K180_SATIRLAR = ["kelimelerin 3. öğesi yaz", "kelimelerin üçüncü öğesi yaz", "kelimelerin sıra. öğesi yaz"];
  var G1 = "yapı Kutu\n    adet TamSayı\n\nkutular boş liste olsun\nbir yeni Kutu olsun\nbirin adedi 1 olsun\nkutulara biri ekle\nher kutu için\n    kutunun adedi 9 olsun\nkutuların json metni yaz";
  var G2 = "sayılar 1, 2, 3 listesi olsun\nher sayı için\n    sayı 7 olsun\nsayıların json metni yaz";
  var G3 = "sayılar 1, 2 listesi olsun\nyedek sayılar olsun\nher sayı için\n    sayı 7 olsun\nyedeğin json metni yaz";
  var ADIMLAR = [
    { id: "kurulum", ad: "Katılımcı", dk: 0 },
    { id: "isinma", ad: "1 Isınma", dk: 2 },
    { id: "serbest", ad: "2 K-016 serbest", dk: 3 },
    { id: "kartlar", ad: "3 K-016 kartlar", dk: 8 },
    { id: "okuma", ad: "4 Sesli okuma", dk: 7 },
    { id: "k180", ad: "5 K-180 sıra", dk: 3 },
    { id: "k093", ad: "6 K-093 gezme", dk: 7 },
    { id: "yazma", ad: "7 Yazma görevi", dk: 5 },
    { id: "kapanis", ad: "8 Kapanış", dk: 2 },
    { id: "bitir", ad: "Bitir", dk: 0 }
  ];

  var kok = document.getElementById("kok");
  var veri = yeniVeri("");
  var adim = 0;
  var sureBaslangic = null;
  var sureHedef = 0;

  function yeniVeri(kimlik) {
    return { protokol: "K016-1", kimlik: kimlik, tarih: new Date().toISOString().slice(0, 10), yas: "", onam: "", k016: {}, okuma: {}, k180: {}, k093: {}, yazma: {}, kapanis: {} };
  }
  function anahtar() { return "zee-oturum-" + veri.kimlik; }
  function kaydet() { if (!veri.kimlik) return; try { localStorage.setItem(anahtar(), JSON.stringify(veri)); } catch (e) { /* özel pencere */ } }
  function yukle(kimlik) {
    try { var s = localStorage.getItem("zee-oturum-" + kimlik); if (s) return JSON.parse(s); } catch (e) { /* yok */ }
    return null;
  }
  function kac(s) { return String(s == null ? "" : s).replace(/[&<>"']/g, function (c) { return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]; }); }
  function al(yol) { return yol.split(".").reduce(function (o, k) { return o == null ? undefined : o[k]; }, veri); }
  function koy(yol, deger) {
    var p = yol.split("."), o = veri;
    for (var i = 0; i < p.length - 1; i++) { if (o[p[i]] == null || typeof o[p[i]] !== "object") o[p[i]] = {}; o = o[p[i]]; }
    o[p[p.length - 1]] = deger; kaydet();
  }
  function cocukMu() { return veri.kimlik.charAt(0) === "C"; }

  // Alan üreticileri (değerler daima kaçışlı; kimlik dışı serbest metin kişisel bilgi içermemeli).
  function metin(yol, etiket, ipucu) {
    return '<label>' + kac(etiket) + '</label><textarea data-k="' + yol + '" placeholder="' + kac(ipucu || "Birebir yaz; kişisel bilgi yazma") + '">' + kac(al(yol)) + '</textarea>';
  }
  function satir(yol, etiket) { return '<label>' + kac(etiket) + '</label><input type="text" data-k="' + yol + '" value="' + kac(al(yol)) + '">'; }
  function secim(yol, etiket, secenekler) {
    var d = al(yol), h = '<label>' + kac(etiket) + '</label><div class="secenekler">';
    secenekler.forEach(function (s) {
      h += '<label><input type="radio" name="' + yol + '" data-k="' + yol + '" value="' + kac(s) + '"' + (d === s ? " checked" : "") + '>' + kac(s) + '</label>';
    });
    return h + '</div>';
  }
  function puan(yol, etiket) { return secim(yol, etiket, ["1", "2", "3", "4", "5"]); }
  function oku(baslik, metinler) { return '<div class="oku"><b>' + kac(baslik) + '</b>' + metinler + '</div>'; }
  function goster(m, kod) { return '<div class="goster' + (kod ? " kod" : "") + '">' + kac(m) + '</div>'; }
  function programKodu(kod) {
    var satirlar = kod.split("\n").filter(function (s) { return !/^\s*#/.test(s); });
    while (satirlar.length && !satirlar[0].trim()) satirlar.shift();
    return satirlar.join("\n").replace(/\s+$/, "");
  }
  function karistir(dizi, tohum) {
    var h = 0; for (var i = 0; i < tohum.length; i++) h = (h * 31 + tohum.charCodeAt(i)) >>> 0;
    var a = dizi.slice();
    for (var j = a.length - 1; j > 0; j--) { h = (h * 1103515245 + 12345) >>> 0; var k = h % (j + 1); var t = a[j]; a[j] = a[k]; a[k] = t; }
    return a;
  }

  function adimHtml() {
    var a = ADIMLAR[adim].id;
    if (a === "kurulum") {
      var sec = '<label>Anonim kimlik</label><select id="kimlik"><option value="">— seç —</option>';
      KIMLIKLER.forEach(function (k) { sec += '<option' + (veri.kimlik === k ? " selected" : "") + '>' + k + '</option>'; });
      sec += '</select>';
      return '<div class="kart"><h2>Katılımcı</h2><p class="not">C01–C10: 8–14 yaş öğrenci/çocuk · P01–P05: profesyonel. İsim, okul, e-posta, ses/video kaydı yok. Kart sırası kimlikten otomatik atanır.</p>' + sec +
        (veri.kimlik ? '<p><b>Kart sırası:</b> ' + SIRALAR[veri.kimlik] + '</p>' + satir("tarih", "Tarih (YYYY-AA-GG)") +
          satir("yas", cocukMu() ? "Yaş grubu (ör. 8–10, 11–14)" : "Rol (ör. yazılımcı, öğretmen; kişisel ayrıntı yok)") +
          secim("onam", cocukMu() ? "Veli onamı ve çocuğun sözlü onayı alındı" : "Katılımcı onayı alındı", ["evet", "hayır", "uygulanamaz"]) +
          '<p class="not">Onam yoksa oturumu yapma. Onam metni: <a href="onam.html">yazdırılabilir onam formu</a>.</p>' : "") + '</div>';
    }
    if (a === "isinma") {
      return '<div class="kart"><h2>Isınma · 2 dk</h2>' + oku("Oku", '“Bilgisayara Türkçe komut veren bir dil deniyoruz. Doğru cevap yok; takıldığın her yer bizim hatamız, senin değil.”') + '</div>';
    }
    if (a === "serbest") {
      return '<div class="kart"><h2>K-016 serbest üretim · 3 dk</h2><p class="not">Hiç çağrı örneği göstermeden yazdır. Satırı düzeltme, Zee kuralı öğretme.</p>' +
        oku("Görev 1", '“<code>notlar</code> listesinin ortalamasını hesaplayan işlemi çağır ve sonucu <code>ortalama</code> adlı değere bağla.”') +
        metin("k016.serbestBag", "Değere bağlama için birebir yazdığı satır") +
        oku("Görev 2", '“<code>selamla</code> işlemini <code>"Ayşe"</code> ve <code>10</code> ile çağır; bu işlem değer döndürmüyor.”') +
        metin("k016.serbestCumle", "İki argümanlı cümle çağrısı için birebir yazdığı satır") +
        metin("k016.serbestNot", "Uygulayıcı notu (yönlendirme yapmadan)") + '</div>';
    }
    if (a === "kartlar") {
      var h = '<div class="kart"><h2>K-016 kör kartlar · 8 dk</h2><p class="not">Sıra: <b>' + SIRALAR[veri.kimlik] + '</b>. Kartta “mevcut/önerilen” yazmaz. Önce “Bu ne yapar?”, sonra “Bu çağrının sonucunu başka bir işleme doğrudan vermek istersen nasıl yazarsın?” Ardından yalnız o kartın kural cümlesini oku ve yeni görevi sor.</p></div>';
      SIRALAR[veri.kimlik].split("-").forEach(function (k) {
        var y = "k016.kart." + k;
        h += '<div class="kart"><h2>Kart ' + k + '</h2>' + goster(KARTLAR[k].satir, true) +
          metin(y + ".ilk", "“Bu ne yapar?” — birebir açıklama") +
          secim(y + ".ilkDogru", "İlk açıklama doğru zihinsel model mi?", ["doğru", "yanlış", "kısmen"]) +
          metin(y + ".icice", "İç içe kullanım için yazdığı (birebir)") +
          secim(y + ".iciceOldu", "İç içe kullanım yazılabildi mi?", ["evet", "hayır"]) +
          oku("Kural cümlesi (yalnız bunu oku)", kac(KARTLAR[k].kural)) +
          oku("Yeni görev", '“<code>fiyatlar</code> listesinin medyanını hesapla ve sonucu <code>medyan</code> adına bağla.”') +
          metin(y + ".sonra", "Öğretim sonrası yazdığı/açıkladığı (birebir)") +
          secim(y + ".sonraDogru", "Öğretim sonrası doğru zihinsel model mi? (çağrılan işlem + bağlanan ad)", ["doğru", "yanlış", "kısmen"]) +
          '<div class="izgara"><div>' + puan(y + ".dogal", "Doğallık 1–5") + '</div><div>' + puan(y + ".anlasilir", "Anlaşılırlık 1–5") + '</div><div>' + puan(y + ".secerim", "Yazarken seçerim 1–5") + '</div></div>' +
          metin(y + ".gerekce", "Birebir gerekçe") + '</div>';
      });
      h += '<div class="kart"><h2>Son tercihler</h2>' + secim("k016.sonDogal", "En doğal", ["A", "B", "C", "hiçbiri"]) + secim("k016.sonAnlasilir", "En anlaşılır", ["A", "B", "C", "hiçbiri"]) +
        secim("k016.sonYazarken", "Yazarken seçerim", ["A", "B", "C", "hiçbiri"]) + metin("k016.kendi", "“Hiçbiri” ise kendi yazımı (birebir)") + '</div>';
      return h;
    }
    if (a === "okuma") {
      var o = '<div class="kart"><h2>Sesli okuma · 7 dk</h2><p class="not">Satırı önce okusun, sonra “sence bu ne yapar?” Cevabı yaz. Yorum satırları gösterilmez.</p></div>';
      (window.OTURUM_PROGRAMLARI || []).forEach(function (p) {
        if (p.yalniz_profesyonel && cocukMu()) return;
        var y = "okuma.p" + p.no;
        o += '<div class="kart"><h2>' + kac(p.dosya) + '</h2>' + goster(programKodu(p.kod), true) +
          metin(y + ".yanit", "“Sence bu ne yapar?” (birebir)") + metin(y + ".takildi", "Takıldığı satırlar / yanlış tahmin (birebir)") + '</div>';
      });
      return o;
    }
    if (a === "k180") {
      var sira = karistir(K180_SATIRLAR, veri.kimlik);
      if (al("k180.gosterimSirasi") !== sira.join(" | ")) koy("k180.gosterimSirasi", sira.join(" | "));
      return '<div class="kart"><h2>K-180 sıra erişimi · 3 dk</h2>' +
        oku("1 · Serbest üretim (öğretmeden)", '“Elinde <code>kelimeler</code> diye bir liste var; üçüncü kelimeyi ekrana yazdıran tek satırı yaz.”') +
        metin("k180.serbest", "Birebir yazdığı satır") +
        oku("2 · Kör tercih", "“Sana en doğal geleni seç, neden?”") + goster(sira.join("\n"), true) +
        secim("k180.kor", "Seçtiği satır", sira) + metin("k180.neden", "Nedeni (birebir)") +
        oku("3 · Taban sorusu", '“<code>elma, armut, kiraz</code> listesinde <code>2. öğesi</code> sence hangisi?”') +
        secim("k180.taban", "Cevabı", ["armut", "kiraz", "diğer"]) + satir("k180.tabanNot", "“Diğer” ise birebir") + '</div>';
    }
    if (a === "k093") {
      function kartG(ad, kod, a1, a2, yol) {
        return '<div class="kart"><h2>Kart ' + ad + '</h2>' + goster(kod, true) + '<p>Seçenekler: <b>A</b> <code>' + kac(a1) + '</code> · <b>B</b> <code>' + kac(a2) + '</code></p>' +
          secim(yol + ".ilk", "İlk tahmin (öğretmeden)", ["A", "B", "diğer"]) + metin(yol + ".ilkNeden", "“Neden?” (birebir; doğru/yanlış deme)") +
          secim(yol + ".sonra", "Tek cümleden sonra tahmin", ["A", "B", "diğer"]) + metin(yol + ".sonraNeden", "Gerekçe (birebir)") + '</div>';
      }
      return '<div class="kart"><h2>K-093 gezme/değer kartları · 7 dk</h2><p class="not">Önce hiçbir kural öğretmeden G1 ve G2’nin son satır çıktısını tahmin ettir. Sonra yalnız cümleyi oku ve yeniden tahmin ettir.</p>' +
        oku("Tek cümle (ilk tahminlerden sonra)", "“Her öğe için dediğinde, döngü adının tur sonundaki değeri listedeki aynı yere geri konur; listenin bir kopyası ayrı değerdir.”") + '</div>' +
        kartG("G1 — alan yazma", G1, '[{"adet":1}]', '[{"adet":9}]', "k093.G1") +
        kartG("G2 — yeniden bağlama", G2, "[1,2,3]", "[7,7,7]", "k093.G2") +
        '<div class="kart"><h2>Kart G3 — kopya</h2><p class="not">Yalnız profesyoneller; çocuk isterse göster.</p>' + goster(G3, true) + '<p>Seçenekler: <b>A</b> <code>[1,2]</code> · <b>B</b> <code>[7,7]</code></p>' +
        secim("k093.G3.gosterildi", "Gösterildi mi?", ["evet", "hayır"]) + secim("k093.G3.cevap", "Cevabı", ["A", "B", "diğer"]) + metin("k093.G3.neden", "Gerekçe (birebir)") + '</div>';
    }
    if (a === "yazma") {
      var gorev = cocukMu() ? "“Yaşını soran, 10'dan büyükse 'abisin/ablasın', değilse 'kardeşsin' diyen programı yaz.”"
        : "“Bir liste sayının ortalamasını alan işlemi tanımla ve çağır; sıfır bölme durumunu hatasını döndür ile ele al.”";
      if (al("yazma.gorev") !== (cocukMu() ? "cocuk" : "profesyonel")) koy("yazma.gorev", cocukMu() ? "cocuk" : "profesyonel");
      return '<div class="kart"><h2>Genel yazma görevi · 5 dk</h2>' + oku("Görev", gorev) +
        metin("yazma.metin", "Yazdığı program / icat ettiği sözdizimi (ALTIN DEĞERİNDE — birebir)", "Satır satır, olduğu gibi") + '</div>';
    }
    if (a === "kapanis") {
      return '<div class="kart"><h2>Kapanış · 2 dk</h2>' + oku("Sor", "“En garip gelen satır hangisiydi?”") +
        metin("kapanis.garip", "En garip satır ve birebir nedeni") +
        '<div class="izgara"><div>' + puan("kapanis.dogal", "Doğal 1–5") + '</div><div>' + puan("kapanis.anlasilir", "Anlaşılır 1–5") + '</div><div>' + puan("kapanis.tekrar", "Tekrar ister mi 1–5") + '</div></div>' +
        metin("kapanis.ekNot", "Serbest ek not (kişisel bilgi temizlenmiş)") + '</div>';
    }
    return '<div class="kart"><h2>Bitir</h2><p>Formu indir ve dosyayı (<code>' + kac(dosyaAdi()) + '</code>) zee projesi sorumlusuna ilet. İndirdikten sonra bu cihazdaki kaydı silebilirsin.</p>' +
      '<div class="ust"><button id="indir2">Formu indir (.md)</button><button class="tehlike" id="sil">Bu cihazdaki kaydı sil</button></div>' +
      '<p class="not">Eksik alanlar formda boş kalır; eksik veri “eşik sağlandı” lehine sayılmaz.</p></div>';
  }

  function ciz() {
    var nav = '<div class="adimlar yazdirma-yok">';
    ADIMLAR.forEach(function (s, i) { nav += '<button data-adim="' + i + '"' + (i === adim ? ' class="aktif"' : "") + (i > 0 && !veri.kimlik ? " disabled" : "") + '>' + kac(s.ad) + '</button>'; });
    nav += '</div>';
    var alt = '<div class="ust yay yazdirma-yok">' +
      '<button class="ikincil" id="geri"' + (adim === 0 ? " disabled" : "") + '>← Geri</button>' +
      (ADIMLAR[adim].dk ? '<button class="ikincil" id="sureBaslat">Süreyi başlat (' + ADIMLAR[adim].dk + ' dk)</button>' : "") +
      '<button id="ileri"' + (adim === ADIMLAR.length - 1 || !veri.kimlik ? " disabled" : "") + '>İleri →</button></div>';
    kok.innerHTML = nav + adimHtml() + alt;
    document.getElementById("durum").textContent = (veri.kimlik ? veri.kimlik + " · sıra " + SIRALAR[veri.kimlik] + " · " : "") + "Protokol K016-1 · veriler yalnız bu cihazda";
    window.scrollTo(0, 0);
  }

  kok.addEventListener("input", function (e) {
    var k = e.target.getAttribute("data-k"); if (!k) return;
    if (e.target.type === "radio") { if (e.target.checked) koy(k, e.target.value); } else koy(k, e.target.value);
  });
  kok.addEventListener("change", function (e) {
    if (e.target.id === "kimlik") {
      var k = e.target.value; if (!k) return;
      veri = yukle(k) || yeniVeri(k); kaydet(); ciz(); return;
    }
    var yol = e.target.getAttribute("data-k");
    if (yol && e.target.type === "radio" && e.target.checked) koy(yol, e.target.value);
  });
  kok.addEventListener("click", function (e) {
    var t = e.target;
    if (t.getAttribute("data-adim") != null) { adim = +t.getAttribute("data-adim"); ciz(); return; }
    if (t.id === "ileri" && adim < ADIMLAR.length - 1) { adim++; ciz(); }
    else if (t.id === "geri" && adim > 0) { adim--; ciz(); }
    else if (t.id === "sureBaslat") { sureBaslangic = Date.now(); sureHedef = ADIMLAR[adim].dk * 60; }
    else if (t.id === "indir2") { indir(); }
    else if (t.id === "sil" && confirm(veri.kimlik + " kaydı bu cihazdan silinsin mi?")) { try { localStorage.removeItem(anahtar()); } catch (x) { /* yok */ } veri = yeniVeri(""); adim = 0; ciz(); }
  });
  document.getElementById("disaAktar").addEventListener("click", indir);

  setInterval(function () {
    var el = document.getElementById("sure");
    if (!sureBaslangic) { el.textContent = "--:--"; return; }
    var gecen = Math.floor((Date.now() - sureBaslangic) / 1000), kalan = sureHedef - gecen;
    var m = Math.floor(Math.abs(kalan) / 60), s = Math.abs(kalan) % 60;
    el.textContent = (kalan < 0 ? "+" : "") + m + ":" + (s < 10 ? "0" : "") + s;
    el.className = "sure" + (kalan < 0 ? " asti" : "");
  }, 500);

  function dosyaAdi() { return (veri.tarih || "YYYY-AA-GG") + "-" + (veri.kimlik || "C__") + ".md"; }
  function d(yol) { var v = al(yol); return v == null || v === "" ? "" : String(v).replace(/\r/g, ""); }
  function blok(v) { return v.indexOf("\n") >= 0 ? "\n\n```text\n" + v + "\n```\n" : " " + v; }

  function markdown() {
    var L = [];
    L.push("# Anonim usability katılımcı formu", "");
    L.push("- Katılımcı kimliği: `" + veri.kimlik + "`", "- Tarih: `" + d("tarih") + "`", "- Yaş grubu/rol (kişisel ayrıntı yok): " + d("yas"),
      "- Kart sırası: `" + SIRALAR[veri.kimlik] + "`", "- Protokol sürümü: `K016-1`", "- Onam/izin süreci uygulandı: `" + (d("onam") || "boş") + "`",
      "- Kayıt aracı: zeedili.com oturum yürütücüsü (K-320)", "");
    L.push("## K-016 — gösterim öncesi serbest üretim", "");
    L.push("- Değere bağlama için birebir yazdığı satır:" + blok(d("k016.serbestBag")));
    L.push("- İki argümanlı cümle çağrısı için birebir yazdığı satır:" + blok(d("k016.serbestCumle")));
    L.push("- Uygulayıcı notu (yönlendirme yapmadan):" + blok(d("k016.serbestNot")), "");
    L.push("## K-016 — kör kart sonuçları", "");
    ["A", "B", "C"].forEach(function (k) {
      var y = "k016.kart." + k;
      L.push("### Kart " + k, "");
      L.push("- İlk açıklama (birebir):" + blok(d(y + ".ilk")));
      L.push("- İlk / öğretim sonrası: " + (d(y + ".ilkDogru") || "boş") + " / " + (d(y + ".sonraDogru") || "boş"));
      L.push("- Öğretim sonrası yazdığı (birebir):" + blok(d(y + ".sonra")));
      L.push("- İç içe kullanım: " + (d(y + ".iciceOldu") || "boş") + " —" + blok(d(y + ".icice")));
      L.push("- Doğal / anlaşılır / yazarken seçerim: " + (d(y + ".dogal") || "-") + " / " + (d(y + ".anlasilir") || "-") + " / " + (d(y + ".secerim") || "-"));
      L.push("- Birebir gerekçe:" + blok(d(y + ".gerekce")), "");
    });
    L.push("- Son tercihler — doğal / anlaşılır / yazarken: " + (d("k016.sonDogal") || "-") + " / " + (d("k016.sonAnlasilir") || "-") + " / " + (d("k016.sonYazarken") || "-"));
    L.push("- “Hiçbiri” ise kendi yazımı:" + blok(d("k016.kendi")), "");
    L.push("## K-093 — gezme kartları", "");
    ["G1", "G2"].forEach(function (g) {
      L.push("- " + g + " ilk / öğretim sonrası — gerekçe: " + (d("k093." + g + ".ilk") || "-") + " / " + (d("k093." + g + ".sonra") || "-") +
        " — ilk:" + blok(d("k093." + g + ".ilkNeden")) + " sonra:" + blok(d("k093." + g + ".sonraNeden")));
    });
    L.push("- G3 (gösterildiyse) — gerekçe: " + (d("k093.G3.gosterildi") === "evet" ? (d("k093.G3.cevap") || "-") : "gösterilmedi") + " —" + blok(d("k093.G3.neden")), "");
    L.push("## K-180 — sıra erişimi", "");
    L.push("- Serbest üretim (birebir):" + blok(d("k180.serbest")));
    L.push("- Kör tercih gösterim sırası: " + d("k180.gosterimSirasi"));
    L.push("- Kör tercih: " + (d("k180.kor") || "-") + " — nedeni:" + blok(d("k180.neden")));
    L.push("- Taban sorusu: " + (d("k180.taban") || "-") + (d("k180.tabanNot") ? " (" + d("k180.tabanNot") + ")" : ""), "");
    L.push("## Sesli okuma", "");
    (window.OTURUM_PROGRAMLARI || []).forEach(function (p) {
      if (p.yalniz_profesyonel && cocukMu()) return;
      L.push("- `" + p.dosya + "` — yanıt:" + blok(d("okuma.p" + p.no + ".yanit")) + " takıldığı:" + blok(d("okuma.p" + p.no + ".takildi")));
    });
    L.push("", "## Yazma görevi (" + (cocukMu() ? "çocuk" : "profesyonel") + ")", "");
    L.push("```text", d("yazma.metin"), "```", "");
    L.push("## Genel", "");
    L.push("- En garip satır ve birebir nedeni:" + blok(d("kapanis.garip")));
    L.push("- Dört soru puanı — doğal / anlaşılır / tekrar ister mi: " + (d("kapanis.dogal") || "-") + " / " + (d("kapanis.anlasilir") || "-") + " / " + (d("kapanis.tekrar") || "-"));
    L.push("- Serbest ek not (kişisel bilgi temizlenmiş):" + blok(d("kapanis.ekNot")), "");
    return L.join("\n");
  }

  function indir() {
    if (!veri.kimlik) { alert("Önce anonim kimlik seç."); return; }
    var b = new Blob([markdown()], { type: "text/markdown;charset=utf-8" });
    var a = document.createElement("a"); a.href = URL.createObjectURL(b); a.download = dosyaAdi();
    document.body.appendChild(a); a.click(); setTimeout(function () { URL.revokeObjectURL(a.href); a.remove(); }, 500);
  }

  window.ZEE_OTURUM = { markdown: markdown, veri: function () { return veri; }, SIRALAR: SIRALAR };
  ciz();
})();
