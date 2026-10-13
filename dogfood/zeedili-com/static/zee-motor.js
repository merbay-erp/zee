/* zee-motor.js — sitenin ortak Zee motoru.
   - Derleyicinin WASM C ABI v3 ikilisini aynı origin'den, tek kez yükler.
   - Kaynağı yalnız bu sekmede çalıştırır; ağ isteği yapmaz.
   - Yüklem-odaklı sözdizimi vurgusu ve tanı çıktısı biçimlendirmesi sağlar.
   DOM'a yalnız textContent ile yazar; kaynak hiçbir zaman HTML olarak yorumlanmaz. */
(() => {
  'use strict';

  const WASM_URL = '/assets/playground/dil.wasm?v=k188p2';
  let wasm = null;
  let limits = null;
  let loading = null;

  /* ---------- WASM ABI v3 ---------- */

  function utf8Length(text, name, maxBytes) {
    let total = 0;
    for (let index = 0; index < text.length; index++) {
      const code = text.charCodeAt(index);
      if (code <= 0x7f) total += 1;
      else if (code <= 0x7ff) total += 2;
      else if (code >= 0xd800 && code <= 0xdbff && index + 1 < text.length &&
               text.charCodeAt(index + 1) >= 0xdc00 && text.charCodeAt(index + 1) <= 0xdfff) {
        total += 4;
        index++;
      } else total += 3;
      if (total > maxBytes) throw new Error(`${name} ${maxBytes} UTF-8 bayt sınırını aşıyor.`);
    }
    return total;
  }

  function lineCount(text) {
    if (!text.length) return 0;
    let count = text.endsWith('\n') ? 0 : 1;
    for (let index = 0; index < text.length; index++) {
      if (text.charCodeAt(index) === 10) count++;
    }
    return count;
  }

  function writeMemory(text, name, maxBytes, maxLines = null) {
    const length = utf8Length(text, name, maxBytes);
    if (maxLines !== null && lineCount(text) > maxLines) {
      throw new Error(`${name} ${maxLines} satır sınırını aşıyor.`);
    }
    if (!length) return [0, 0];
    const pointer = wasm.dil_bellek_ayir(length) >>> 0;
    if (!pointer || pointer + length > wasm.memory.buffer.byteLength) {
      if (pointer) wasm.dil_bellek_birak(pointer, length);
      throw new Error('WASM girdi tamponu ayrılamadı.');
    }
    try {
      const encoded = new TextEncoder().encodeInto(
        text,
        new Uint8Array(wasm.memory.buffer, pointer, length)
      );
      if (encoded.read !== text.length || encoded.written !== length) {
        throw new Error('UTF-8 kodlama boyu hesaplanan bütçeyle uyuşmuyor.');
      }
      return [pointer, length];
    } catch (error) {
      wasm.dil_bellek_birak(pointer, length);
      throw error;
    }
  }

  async function instantiate() {
    const response = await fetch(WASM_URL, {cache: 'force-cache'});
    if (!response.ok) throw new Error(`WASM indirilemedi (${response.status}).`);
    let instance;
    try {
      ({instance} = await WebAssembly.instantiateStreaming(response.clone(), {}));
    } catch (_) {
      ({instance} = await WebAssembly.instantiate(await response.arrayBuffer(), {}));
    }
    const exports = instance.exports;
    const required = [
      'memory', 'dil_abi_surumu', 'dil_bellek_ayir', 'dil_bellek_birak',
      'dil_calistir', 'dil_sonuc_tamponu_uzunlugu',
      'dil_playground_kaynak_bayti', 'dil_playground_girdi_bayti',
      'dil_playground_girdi_satiri'
    ];
    if (required.some(name => !exports[name]) || exports.dil_abi_surumu() !== 3) {
      throw new Error('Site ile derleyici WASM ABI v3 sözleşmesi uyuşmuyor.');
    }
    const found = {
      sourceBytes: exports.dil_playground_kaynak_bayti() >>> 0,
      inputBytes: exports.dil_playground_girdi_bayti() >>> 0,
      inputLines: exports.dil_playground_girdi_satiri() >>> 0
    };
    if (!found.sourceBytes || !found.inputBytes || !found.inputLines) {
      throw new Error('Derleyici geçersiz playground sınırı bildirdi.');
    }
    wasm = exports;
    limits = found;
  }

  function load() {
    if (!loading) {
      loading = instantiate().catch(error => {
        loading = null;
        throw error;
      });
    }
    return loading;
  }

  function run(source, input = '', seed = 1) {
    if (!wasm || !limits) throw new Error('Derleyici henüz hazır değil.');
    const started = performance.now();
    let sourcePointer = 0;
    let sourceLength = 0;
    let inputPointer = 0;
    let inputLength = 0;
    let resultPointer = 0;
    let resultTotal = 0;
    let text = '';
    try {
      [sourcePointer, sourceLength] = writeMemory(source, 'Kaynak', limits.sourceBytes);
      [inputPointer, inputLength] = writeMemory(input, 'Soru girdisi', limits.inputBytes, limits.inputLines);
      const seedValue = BigInt(Math.max(1, Math.floor(Number(seed)) || 1));
      resultPointer = wasm.dil_calistir(sourcePointer, sourceLength, inputPointer, inputLength, seedValue) >>> 0;
      if (!resultPointer) throw new Error('WASM sonuç tamponu ayrılamadı.');
      resultTotal = wasm.dil_sonuc_tamponu_uzunlugu(resultPointer) >>> 0;
      const memoryLength = wasm.memory.buffer.byteLength;
      if (resultTotal < 4 || resultPointer + resultTotal > memoryLength) {
        throw new Error('WASM geçersiz bir sonuç tamponu döndürdü.');
      }
      const length = new DataView(wasm.memory.buffer).getUint32(resultPointer, true);
      if (length !== resultTotal - 4) throw new Error('WASM sonuç başlığı uyuşmuyor.');
      const bytes = new Uint8Array(wasm.memory.buffer, resultPointer + 4, length);
      text = new TextDecoder('utf-8', {fatal: true}).decode(bytes);
    } finally {
      const sourceReleased = !sourcePointer || wasm.dil_bellek_birak(sourcePointer, sourceLength) === 1;
      const inputReleased = !inputPointer || wasm.dil_bellek_birak(inputPointer, inputLength) === 1;
      const resultReleased = !resultPointer || (resultTotal && wasm.dil_bellek_birak(resultPointer, resultTotal) === 1);
      if (!sourceReleased || !inputReleased || !resultReleased) {
        throw new Error('WASM tampon sahipliği doğrulanamadı.');
      }
    }
    const error = /(^|\n)(HATA |WASM ABI HATASI:|PLAYGROUND SINIR HATASI:)/.test(text);
    return {text, error, ms: Math.max(1, Math.round(performance.now() - started))};
  }

  /* ---------- Sözdizimi vurgusu ---------- */

  // Zee'de ayrılmış anahtar kelime yoktur: cümlenin son kelimesi yüklemdir.
  // Vurgu bu kuralı izler; birkaç kalıp sözcüğü ve tür adları ayrıca renklenir.
  const TWO_WORD = new Set([
    'olarak al', 'yanıtını gönder', 'durumuyla gönder', 'diye sor', 'kez tekrarla',
    'kadar tekrarla', 'sunucu başlat', 'birimini kullan', 'eşit olmalı', 'eşit olmamalı',
    'istek geldiğinde', 'olduğu sürece', 'değer döndürmez', 'hepsini bekle', 'olarak eşzamanlı'
  ]);
  const LEADING = new Set(['işlem', 'yapı', 'test', 'her', 'değilse', 'eşzamanlı']);
  const CONNECTIVES = new Set([
    'ile', 've', 'veya', 'için', 'olarak', 'boş', 'yeni', 'liste', 'listesi', 'sözlük',
    'doğru', 'yanlış', 'yok', 'kadar', 'kez', 'daha', 'göre', 'ise', 'değilse', 'her'
  ]);
  const TOKEN = /("(?:\\.|[^"\\])*"?)|(#.*$)|(-?\d+(?:,\d+)?)|([\p{L}_][\p{L}\p{N}_]*)|(\s+)|(.)/gu;

  function span(cls, text) {
    const element = document.createElement('span');
    element.className = cls;
    element.textContent = text;
    return element;
  }

  function highlightLine(line, fragment) {
    const tokens = [];
    TOKEN.lastIndex = 0;
    let match;
    while ((match = TOKEN.exec(line)) !== null) {
      if (match[0] === '') { TOKEN.lastIndex++; continue; }
      let kind = 'plain';
      if (match[1] !== undefined) kind = 'metin';
      else if (match[2] !== undefined) kind = 'yorum';
      else if (match[3] !== undefined) kind = 'sayi';
      else if (match[4] !== undefined) kind = 'kelime';
      else if (match[5] !== undefined) kind = 'bosluk';
      tokens.push({kind, text: match[0], cls: ''});
    }
    const words = tokens.filter(token => token.kind === 'kelime');
    const meaningful = tokens.filter(token => token.kind !== 'bosluk' && token.kind !== 'yorum');
    const last = meaningful.at(-1);

    words.forEach(token => {
      const word = token.text;
      if (/^[A-ZÇĞİÖŞÜ]{2,}$/.test(word)) token.cls = 'z-kalip';
      else if (/^[A-ZÇĞİÖŞÜ]/.test(word)) token.cls = 'z-tur';
      else if (CONNECTIVES.has(word)) token.cls = 'z-kalip';
    });

    const first = words[0];
    if (first && meaningful[0] === first && LEADING.has(first.text)) {
      first.cls = 'z-yuklem';
      if (first.text === 'işlem') {
        words.slice(1).forEach(token => { token.cls = 'z-islem'; });
      } else if (first.text === 'yapı') {
        words.slice(1).forEach(token => { token.cls = 'z-tur'; });
      }
    }

    if (last && last.kind === 'kelime' && !(first && first.text === 'işlem' && last !== first)) {
      if (last.cls !== 'z-tur') last.cls = 'z-yuklem';
      const index = words.indexOf(last);
      const previous = words[index - 1];
      if (previous && TWO_WORD.has(`${previous.text} ${last.text}`)) previous.cls = 'z-yuklem';
    }

    tokens.forEach(token => {
      if (token.kind === 'metin') fragment.append(span('z-metin', token.text));
      else if (token.kind === 'yorum') fragment.append(span('z-yorum', token.text));
      else if (token.kind === 'sayi') fragment.append(span('z-sayi', token.text));
      else if (token.cls) fragment.append(span(token.cls, token.text));
      else fragment.append(token.text);
    });
  }

  function highlight(source) {
    const fragment = document.createDocumentFragment();
    source.split('\n').forEach((line, index) => {
      if (index) fragment.append('\n');
      highlightLine(line, fragment);
    });
    return fragment;
  }

  /* ---------- Tanı çıktısı ---------- */

  function formatOutput(text) {
    const fragment = document.createDocumentFragment();
    text.split('\n').forEach((line, index) => {
      if (index) fragment.append('\n');
      let gutter;
      if (/^HATA [A-Z]+\d+/.test(line) || /^(WASM ABI HATASI|PLAYGROUND SINIR HATASI):/.test(line)) {
        fragment.append(span('tani-hata', line));
      } else if ((gutter = /^(\s*\d+ \|)(.*)$/.exec(line))) {
        fragment.append(span('tani-satir', gutter[1]), gutter[2]);
      } else if (/^\s*\^[\^\s]*$/.test(line)) {
        fragment.append(span('tani-isaret', line));
      } else if (/^Öneri:/.test(line)) {
        fragment.append(span('tani-oneri', line));
      } else if (/^(Ayrıntı için:|Birim:)/.test(line)) {
        fragment.append(span('tani-ipucu', line));
      } else if (/^✓ /.test(line)) {
        fragment.append(span('tani-oneri', line));
      } else {
        fragment.append(line);
      }
    });
    return fragment;
  }

  /* ---------- Paylaşım kodlaması ---------- */

  function encode(text) {
    const bytes = new TextEncoder().encode(text);
    let binary = '';
    for (let start = 0; start < bytes.length; start += 8192) {
      binary += String.fromCharCode(...bytes.subarray(start, start + 8192));
    }
    return btoa(binary).replaceAll('+', '-').replaceAll('/', '_').replaceAll('=', '');
  }

  function decode(encoded) {
    const normalized = encoded.replaceAll('-', '+').replaceAll('_', '/');
    const padded = normalized + '='.repeat((4 - normalized.length % 4) % 4);
    const binary = atob(padded);
    const bytes = Uint8Array.from(binary, character => character.charCodeAt(0));
    return new TextDecoder('utf-8', {fatal: true}).decode(bytes);
  }

  function playgroundUrl(source) {
    utf8Length(source, 'Paylaşım kaynağı', 16384);
    return `/playground/#kod=${encode(source)}`;
  }

  /* ---------- Vurgulu düzenleyici ---------- */

  // Saydam textarea'nın arkasındaki <pre> aynı metni renkli gösterir.
  function attachEditor(wrapper, onChange) {
    const textarea = wrapper.querySelector('textarea');
    const pre = wrapper.querySelector('pre');
    const code = pre.querySelector('code') || pre;
    const paint = () => {
      code.replaceChildren(highlight(`${textarea.value}\n`));
      pre.scrollTop = textarea.scrollTop;
      pre.scrollLeft = textarea.scrollLeft;
    };
    textarea.addEventListener('input', () => { paint(); onChange?.(); });
    textarea.addEventListener('scroll', () => {
      pre.scrollTop = textarea.scrollTop;
      pre.scrollLeft = textarea.scrollLeft;
    });
    textarea.addEventListener('keydown', event => {
      if (event.key === 'Tab' && !event.shiftKey && !event.altKey && !event.ctrlKey && !event.metaKey) {
        event.preventDefault();
        textarea.setRangeText('    ', textarea.selectionStart, textarea.selectionEnd, 'end');
        textarea.dispatchEvent(new Event('input'));
      }
    });
    paint();
    return {
      set(value) {
        textarea.value = value;
        textarea.scrollTop = 0;
        paint();
        onChange?.();
      },
      get: () => textarea.value,
      paint,
      textarea
    };
  }

  window.ZeeMotor = Object.freeze({
    load,
    run,
    ready: () => Boolean(wasm),
    highlight,
    formatOutput,
    encode,
    decode,
    playgroundUrl,
    attachEditor,
    utf8Length
  });
})();
