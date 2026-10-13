/* playground.js — "30 dakikada Zee" dersleri. Motor: zee-motor.js (WASM ABI v3).
   Ders içerikleri Zee tarafından <template data-ders> olarak üretilir; burada
   yalnız okunur, kaynak kod hiçbir koşulda HTML olarak yorumlanmaz. */
(() => {
  'use strict';

  const workbench = document.querySelector('[data-zee-workbench]');
  const motor = window.ZeeMotor;
  if (!workbench || !motor) return;

  const $ = selector => workbench.querySelector(selector);
  const lines = $('#zee-lines');
  const output = $('#zee-output');
  const input = $('#zee-input');
  const seed = $('#zee-seed');
  const inputBox = $('#girdi-kutusu');
  const runButton = $('#zee-run');
  const engineStatus = $('#zee-engine-status');
  const position = $('#zee-position');
  const title = $('#zee-example-title');
  const body = $('#zee-example-summary');
  const counter = $('#ders-sayaci');
  const timing = $('#zee-timing');
  const path = $('#ders-yolu');
  const nextBottom = $('#ders-ileri-alt');
  const visited = new Set();
  const back = $('#ders-geri');
  const next = $('#ders-ileri');

  const lessons = [...workbench.querySelectorAll('template[data-ders]')].map(template => {
    const content = template.content.cloneNode(true);
    const givenInput = content.querySelector('.ders-girdi');
    givenInput?.remove();
    return {
      title: template.dataset.baslik,
      body: content.querySelector('.ders-govde'),
      source: content.querySelector('.ders-kaynak')?.textContent ?? '',
      input: givenInput?.textContent ?? ''
    };
  });
  if (!lessons.length) return;

  let current = 0;
  let downloadName = 'ders-1';

  function lineNumbers() {
    const value = editor.get();
    const count = Math.max(1, value.split('\n').length);
    lines.textContent = Array.from({length: count}, (_, index) => index + 1).join('\n');
    lines.scrollTop = editor.textarea.scrollTop;
    const pieces = value.slice(0, editor.textarea.selectionStart).split('\n');
    position.textContent = `Satır ${pieces.length}, Sütun ${pieces.at(-1).length + 1}`;
  }

  const editor = motor.attachEditor($('[data-editor]'), lineNumbers);

  function setEngineStatus(text, state = '') {
    engineStatus.textContent = text;
    engineStatus.dataset.state = state;
  }

  function hint(text) {
    const element = document.createElement('span');
    element.className = 'tani-ipucu';
    element.textContent = text;
    output.replaceChildren(element);
    timing.textContent = '';
  }

  function showLesson(index, {updateHash = true} = {}) {
    current = Math.max(0, Math.min(lessons.length - 1, index));
    const lesson = lessons[current];
    title.textContent = lesson.title;
    body.replaceChildren(lesson.body.cloneNode(true));
    counter.textContent = `Ders ${current + 1} / ${lessons.length}`;
    visited.add(current);
    path.querySelectorAll('button').forEach((star, index) => {
      star.classList.toggle('gecildi', visited.has(index));
      if (index === current) star.setAttribute('aria-current', 'step');
      else star.removeAttribute('aria-current');
    });
    back.disabled = current === 0;
    next.disabled = current === lessons.length - 1;
    nextBottom.hidden = current === lessons.length - 1;
    editor.set(lesson.source);
    input.value = lesson.input;
    seed.value = '1';
    inputBox.open = Boolean(lesson.input);
    downloadName = `ders-${current + 1}`;
    hint('▶ Çalıştır ile sonucu gör. Kısayol: Ctrl/⌘ + Enter');
    if (updateHash) history.replaceState(null, '', `#ders-${current + 1}`);
  }

  function run() {
    if (!motor.ready()) {
      hint('Derleyici henüz yükleniyor; birkaç saniye sonra yeniden dene.');
      return;
    }
    runButton.disabled = true;
    setEngineStatus('Çalıştırılıyor', 'loading');
    try {
      const result = motor.run(editor.get(), input.value, seed.value);
      output.replaceChildren(result.text ? motor.formatOutput(result.text) : '(çıktı yok)');
      timing.textContent = `${result.ms} ms`;
      setEngineStatus(result.error ? 'Tanı üretildi' : 'Çalıştı', result.error ? 'error' : 'ready');
    } catch (error) {
      output.textContent = `Çalıştırılamadı: ${error.message || error}`;
      setEngineStatus('Çalışma başarısız', 'error');
    } finally {
      runButton.disabled = false;
    }
  }

  async function share() {
    let url;
    try {
      url = new URL(motor.playgroundUrl(editor.get()), location.href);
    } catch (error) {
      setEngineStatus(error.message || 'Paylaşım kaynağı çok büyük', 'error');
      return;
    }
    history.replaceState(null, '', url);
    try {
      await navigator.clipboard.writeText(url.href);
      setEngineStatus('Bağlantı kopyalandı', 'ready');
    } catch (_) {
      setEngineStatus('Bağlantı adres çubuğunda', 'ready');
    }
  }

  function download() {
    const blob = new Blob([editor.get()], {type: 'text/plain;charset=utf-8'});
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${downloadName}.dil`;
    link.click();
    URL.revokeObjectURL(url);
  }

  path.replaceChildren(...lessons.map((lesson, index) => {
    const star = document.createElement('button');
    star.type = 'button';
    star.title = `${index + 1}. ${lesson.title}`;
    star.setAttribute('aria-label', `Ders ${index + 1}: ${lesson.title}`);
    star.addEventListener('click', () => showLesson(index));
    return star;
  }));
  nextBottom.addEventListener('click', () => showLesson(current + 1));
  back.addEventListener('click', () => showLesson(current - 1));
  next.addEventListener('click', () => showLesson(current + 1));
  editor.textarea.addEventListener('click', lineNumbers);
  editor.textarea.addEventListener('keyup', lineNumbers);
  editor.textarea.addEventListener('scroll', () => { lines.scrollTop = editor.textarea.scrollTop; });
  runButton.addEventListener('click', run);
  $('#zee-reset').addEventListener('click', () => showLesson(current));
  $('#zee-copy').addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(editor.get());
      setEngineStatus('Kod kopyalandı', 'ready');
    } catch (_) {
      editor.textarea.select();
      setEngineStatus('Kod seçildi; kopyalayabilirsin', 'ready');
    }
  });
  $('#zee-download').addEventListener('click', download);
  $('#zee-share').addEventListener('click', share);
  document.addEventListener('keydown', event => {
    if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
      event.preventDefault();
      run();
    }
  });

  // Açılış: #kod=… paylaşılan kodu, #ders-N ilgili dersi açar.
  let shared = null;
  if (location.hash.startsWith('#kod=')) {
    try { shared = motor.decode(location.hash.slice(5)); } catch (_) { shared = null; }
  }
  const lessonMatch = /^#ders-(\d+)$/.exec(location.hash);
  showLesson(lessonMatch ? Number(lessonMatch[1]) - 1 : 0, {updateHash: false});
  if (shared !== null) {
    title.textContent = 'Paylaşılan kod';
    const note = document.createElement('p');
    note.textContent = 'Bu kod bir bağlantıyla geldi. Çalıştır, değiştir ya da aşağıdan bir derse geç.';
    body.replaceChildren(note);
    counter.textContent = 'Serbest çalışma';
    editor.set(shared);
    input.value = '';
    inputBox.open = false;
    downloadName = 'program';
  }

  setEngineStatus('Derleyici yükleniyor', 'loading');
  motor.load()
    .then(() => {
      setEngineStatus('Derleyici hazır', 'ready');
      if (shared !== null) run();
    })
    .catch(error => {
      output.textContent = `Derleyici yüklenemedi: ${error.message || error}`;
      setEngineStatus('Derleyici yüklenemedi', 'error');
    });
})();
