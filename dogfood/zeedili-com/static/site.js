/* site.js — gezinme, kod blokları (vurgu, kopyala, derslerde aç) ve belge araması. */
(() => {
  'use strict';

  const motor = window.ZeeMotor;

  /* ---------- Mobil menü ---------- */

  const menuButton = document.querySelector('.menu-dugmesi');
  const menu = document.querySelector('#ana-menu');
  const setMenu = open => {
    menuButton?.setAttribute('aria-expanded', String(open));
    menuButton?.setAttribute('aria-label', open ? 'Menüyü kapat' : 'Menüyü aç');
    menu?.classList.toggle('acik', open);
  };
  menuButton?.addEventListener('click', () => setMenu(menuButton.getAttribute('aria-expanded') !== 'true'));
  menu?.addEventListener('click', event => { if (event.target.closest('a')) setMenu(false); });

  /* ---------- Kod blokları ---------- */

  if (motor) {
    document.querySelectorAll('code.zee').forEach(code => {
      code.replaceChildren(motor.highlight(code.textContent));
    });
  }

  function copyText(button) {
    const holder = button.closest('figure, .kurulum-satiri');
    const code = holder?.querySelector('code');
    if (!code) return '';
    const text = code.textContent;
    if (holder.classList.contains('terminal')) {
      const commands = text.split('\n').map(line => /^(\$|PS>) (.*)$/.exec(line)?.[2]).filter(Boolean);
      if (commands.length) return commands.join('\n');
    }
    return text;
  }

  document.querySelectorAll('.kopyala').forEach(button => {
    button.addEventListener('click', async () => {
      const label = button.textContent;
      try {
        await navigator.clipboard.writeText(copyText(button));
        button.textContent = 'Kopyalandı';
      } catch (_) {
        button.textContent = 'Seçip kopyala';
      }
      setTimeout(() => { button.textContent = label; }, 1400);
    });
  });

  function openInPlayground(link, source) {
    try {
      link.href = motor ? motor.playgroundUrl(source) : '/playground/';
    } catch (_) {
      link.href = '/playground/';
    }
  }

  document.querySelectorAll('figure [data-alanda-ac]').forEach(link => {
    const code = link.closest('figure').querySelector('code');
    const update = () => openInPlayground(link, code.textContent);
    link.addEventListener('pointerenter', update);
    link.addEventListener('focus', update);
    link.addEventListener('click', update);
  });

  /* ---------- Ana sayfa örnek defteri ---------- */

  const defter = document.querySelector('[data-defter-kutusu]');
  if (defter) {
    const sekmeler = [...defter.querySelectorAll('[data-defter]')];
    const sayfalar = [...defter.querySelectorAll('[data-defter-sayfa]')];
    const goster = ad => {
      sekmeler.forEach(sekme => sekme.setAttribute('aria-selected', String(sekme.dataset.defter === ad)));
      sayfalar.forEach(sayfa => { sayfa.hidden = sayfa.dataset.defterSayfa !== ad; });
    };
    sekmeler.forEach((sekme, sira) => {
      sekme.addEventListener('click', () => goster(sekme.dataset.defter));
      sekme.addEventListener('keydown', olay => {
        const yon = olay.key === 'ArrowDown' || olay.key === 'ArrowRight' ? 1 : olay.key === 'ArrowUp' || olay.key === 'ArrowLeft' ? -1 : 0;
        if (!yon) return;
        olay.preventDefault();
        const hedef = sekmeler[(sira + yon + sekmeler.length) % sekmeler.length];
        hedef.focus();
        goster(hedef.dataset.defter);
      });
    });
    defter.dataset.hazir = '';
    if (sekmeler[0]) goster(sekmeler[0].dataset.defter);
  }

  /* ---------- Belge araması ---------- */

  const search = document.querySelector('#belge-arama');
  const results = document.querySelector('#arama-sonuclari');
  // Depo yolu → sitedeki belge adresi (kaynak/markdown.dil belgeAdi ile aynı kural).
  const belgeAdresi = yol => {
    const onekler = [['docs/', ''], ['spec/', 'spec-'], ['adr/', 'adr-'], ['rfcs/', 'rfc-'], ['editors/', 'editor-']];
    let ad = yol;
    for (const [onek, yeni] of onekler) {
      if (ad.startsWith(onek)) { ad = yeni + ad.slice(onek.length); break; }
    }
    ad = ad.replace('.md', '').toLocaleLowerCase('tr-TR').replace('-readme', '');
    return `/belge/${ad}/`;
  };
  const documents = [
    ['17 Ekim geliştirme güncellemesi', 'yeni yenilik 0.8.0-dev zee.ayar utf-16 emoji lsp biçimle denetle göçür kuru durum sql plan freeze', 'docs/zeedili-gelistirme-guncellemesi-20261017.md'],
    ['Başlangıç rehberi', 'kurulum ilk proje cli çalıştırma', 'docs/baslangic.md'],
    ['Dil turu', 'sözdizimi tür liste sözlük işlem yapı desen koşul döngü', 'docs/dil-turu.md'],
    ['Öğretmen rehberi', 'sınıf ders öğrenci çocuk proje', 'docs/ogretmen-rehberi.md'],
    ['Hata kataloğu', 'tanı kodu hata çözüm öneri T002 S015', 'docs/hata-katalogu.md'],
    ['Türler', 'tür sistemi tamsayı ondalık metin çıkarım imza', 'spec/04-turler.md'],
    ['Morfoloji profili', 'türkçe ek iyelik çoğul ünlü uyumu zee-tr-1', 'spec/13-surumlu-morfoloji-profili.md'],
    ['Hata değeri', 'sonuç hata zinciri kod neden veri', 'spec/15-yapilandirilmis-hata-degeri.md'],
    ['Ondalık', 'onluk aritmetik para hassasiyet', 'spec/16-keyfi-hassasiyetli-ondalik.md'],
    ['Deterministik IO', 'tohum rastgele saat dosya zee-io-1 test', 'docs/deterministik-io-profili.md'],
    ['Eşzamanlılık', 'görev bekle son tarih iptal', 'spec/14-yapilandirilmis-eszamanlilik.md'],
    ['Web üretim profili', 'web proxy nginx oturum csrf güvenlik sunucu', 'docs/web-production-profili.md'],
    ['PostgreSQL', 'veritabanı sql migration havuz transaction göçür', 'spec/25-postgresql-veri-erisimi.md'],
    ['Yetkinlik ve ağ', 'ağ izin outbound https güvenli mod', 'docs/yetkinlik-ve-ag-guvenligi.md'],
    ['Paketler ve tedarik', 'paket kilit registry güven imza ekle paketle', 'docs/tedarik-zinciri.md'],
    ['Editör ve LSP', 'vscode hover tanıma git yeniden adlandır', 'editors/README.md'],
    ['Sürüm notları', 'değişiklik göç sürüm', 'docs/surumler.md'],
    ['RFC dizini', 'karar gerekçe öneri', 'rfcs/README.md'],
    ['ADR dizini', 'mimari karar katman', 'adr/README.md']
  ];
  const normalize = value => value.toLocaleLowerCase('tr-TR').normalize('NFKD').replace(/[̀-ͯ]/g, '');

  search?.addEventListener('input', () => {
    if (!results) return;
    const query = normalize(search.value.trim());
    if (query.length < 2) { results.replaceChildren(); return; }
    const matches = documents.filter(item => normalize(`${item[0]} ${item[1]}`).includes(query)).slice(0, 6);
    if (!matches.length) {
      const empty = document.createElement('p');
      empty.textContent = 'Eşleşen belge bulunamadı.';
      results.replaceChildren(empty);
      return;
    }
    results.replaceChildren(...matches.map(item => {
      const link = document.createElement('a');
      link.href = belgeAdresi(item[2]);
      link.append(item[0]);
      const path = document.createElement('span');
      path.textContent = item[2];
      link.append(path);
      return link;
    }));
  });

  addEventListener('keydown', event => {
    const typing = event.target.closest?.('input, textarea, [contenteditable]');
    if (event.key === '/' && search && !typing) {
      event.preventDefault();
      search.focus();
    }
    if (event.key === 'Escape' && menu?.classList.contains('acik')) {
      setMenu(false);
      menuButton?.focus();
    }
  });
})();
