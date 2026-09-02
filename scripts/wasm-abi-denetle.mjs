import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const wasmYolu = process.argv[2];
if (!wasmYolu) {
  throw new Error("kullanım: node scripts/wasm-abi-denetle.mjs <dil.wasm>");
}

const { instance } = await WebAssembly.instantiate(await readFile(wasmYolu), {});
const wasm = instance.exports;
for (const ad of [
  "memory",
  "dil_abi_surumu",
  "dil_bellek_ayir",
  "dil_bellek_birak",
  "dil_calistir",
  "dil_sonuc_tamponu_uzunlugu",
  "dil_playground_kaynak_bayti",
  "dil_playground_girdi_bayti",
  "dil_playground_girdi_satiri",
]) {
  assert.ok(wasm[ad], `WASM export'u eksik: ${ad}`);
}
assert.equal(wasm.dil_abi_surumu(), 3);
const kaynakSiniri = wasm.dil_playground_kaynak_bayti() >>> 0;
const girdiSiniri = wasm.dil_playground_girdi_bayti() >>> 0;
const girdiSatiriSiniri = wasm.dil_playground_girdi_satiri() >>> 0;
assert.equal(kaynakSiniri, 8 * 1024 * 1024);
assert.equal(girdiSiniri, 1024 * 1024);
assert.equal(girdiSatiriSiniri, 4096);

const kodlayici = new TextEncoder();
const cozucu = new TextDecoder("utf-8", { fatal: true });

function girdiAyir(baytlar) {
  if (baytlar.length === 0) return [0, 0];
  const ptr = wasm.dil_bellek_ayir(baytlar.length) >>> 0;
  assert.notEqual(ptr, 0);
  assert.ok(ptr + baytlar.length <= wasm.memory.buffer.byteLength);
  new Uint8Array(wasm.memory.buffer, ptr, baytlar.length).set(baytlar);
  return [ptr, baytlar.length];
}

function sonucuOkuVeBirak(ptr) {
  ptr >>>= 0;
  assert.notEqual(ptr, 0);
  const toplam = wasm.dil_sonuc_tamponu_uzunlugu(ptr) >>> 0;
  assert.ok(toplam >= 4);
  assert.ok(ptr + toplam <= wasm.memory.buffer.byteLength);
  const govdeUzunlugu = new DataView(wasm.memory.buffer).getUint32(ptr, true);
  assert.equal(govdeUzunlugu, toplam - 4);
  const metin = cozucu.decode(
    new Uint8Array(wasm.memory.buffer, ptr + 4, govdeUzunlugu),
  );
  assert.equal(wasm.dil_bellek_birak(ptr, toplam), 1);
  assert.equal(wasm.dil_bellek_birak(ptr, toplam), 0);
  return metin;
}

const [kaynakPtr, kaynakUzunlugu] = girdiAyir(
  kodlayici.encode('"wasm ayakta" yaz\n'),
);
const basarili = wasm.dil_calistir(kaynakPtr, kaynakUzunlugu, 0, 0, 7n);
assert.equal(sonucuOkuVeBirak(basarili), "wasm ayakta");
assert.equal(wasm.dil_bellek_birak(kaynakPtr, kaynakUzunlugu + 1), 0);
assert.equal(wasm.dil_bellek_birak(kaynakPtr, kaynakUzunlugu), 1);

const kayitDisi = wasm.dil_calistir(1, -1, 0, 0, 7n);
assert.match(sonucuOkuVeBirak(kayitDisi), /^WASM ABI HATASI: kaynak:/);

const [utf8Ptr, utf8Uzunlugu] = girdiAyir(Uint8Array.of(0xff, 0xfe));
const gecersizUtf8 = wasm.dil_calistir(utf8Ptr, utf8Uzunlugu, 0, 0, 7n);
assert.equal(
  sonucuOkuVeBirak(gecersizUtf8),
  "WASM ABI HATASI: kaynak geçerli UTF-8 değil",
);
assert.equal(wasm.dil_bellek_birak(utf8Ptr, utf8Uzunlugu), 1);

assert.equal(wasm.dil_bellek_ayir(0), 0);
assert.equal(wasm.dil_bellek_ayir(-1), 0);

const buyukKaynak = wasm.dil_bellek_ayir(kaynakSiniri + 1) >>> 0;
assert.notEqual(buyukKaynak, 0);
assert.match(
  sonucuOkuVeBirak(wasm.dil_calistir(buyukKaynak, kaynakSiniri + 1, 0, 0, 7n)),
  /^WASM ABI HATASI: kaynak: playground tür bütçesini aşıyor$/,
);
assert.equal(wasm.dil_bellek_birak(buyukKaynak, kaynakSiniri + 1), 1);

const cokGirdi = wasm.dil_bellek_ayir(girdiSiniri + 1) >>> 0;
assert.notEqual(cokGirdi, 0);
assert.match(
  sonucuOkuVeBirak(wasm.dil_calistir(0, 0, cokGirdi, girdiSiniri + 1, 7n)),
  /^WASM ABI HATASI: girdi: playground tür bütçesini aşıyor$/,
);
assert.equal(wasm.dil_bellek_birak(cokGirdi, girdiSiniri + 1), 1);

const cokSatir = wasm.dil_bellek_ayir(girdiSatiriSiniri + 1) >>> 0;
assert.notEqual(cokSatir, 0);
new Uint8Array(wasm.memory.buffer, cokSatir, girdiSatiriSiniri + 1).fill(10);
assert.match(
  sonucuOkuVeBirak(wasm.dil_calistir(0, 0, cokSatir, girdiSatiriSiniri + 1, 7n)),
  /^PLAYGROUND SINIR HATASI: soru girdisi 4097 satır;/,
);
assert.equal(wasm.dil_bellek_birak(cokSatir, girdiSatiriSiniri + 1), 1);

const canliTamponlar = Array.from({ length: 8 }, () => {
  const ptr = wasm.dil_bellek_ayir(1) >>> 0;
  assert.notEqual(ptr, 0);
  return ptr;
});
assert.equal(wasm.dil_bellek_ayir(1), 0);
for (const ptr of canliTamponlar) {
  assert.equal(wasm.dil_bellek_birak(ptr, 1), 1);
}

console.log("WASM ABI v3 doğrulandı: pointer/boy, UTF-8, sahiplik ve girdi bütçeleri.");
