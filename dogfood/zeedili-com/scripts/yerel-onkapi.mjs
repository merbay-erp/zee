import { createReadStream, promises as fs } from 'node:fs';
import { createServer, request as httpRequest } from 'node:http';
import { dirname, extname, join, normalize } from 'node:path';
import { fileURLToPath } from 'node:url';

const kok = dirname(dirname(fileURLToPath(import.meta.url)));
const statikKoku = join(kok, 'static');
const belgeKoku = join(kok, 'uretilen', 'belge');
const kapi = Number.parseInt(process.env.ZEE_ONKAPI_PORT ?? '8125', 10);
const isciKapisi = Number.parseInt(process.env.ZEE_ISCI_PORT ?? '8126', 10);
const yerelCsp = [
  "default-src 'self'",
  "base-uri 'self'",
  "connect-src 'self'",
  "font-src 'self'",
  "form-action 'self'",
  "frame-ancestors 'none'",
  "img-src 'self' data:",
  "object-src 'none'",
  "script-src 'self' 'wasm-unsafe-eval' 'sha256-kyVhpOBDLAFXvWuAHP25qpX4BKf4cHqN2aWbDNT1wXg='",
  "style-src 'self'",
].join('; ');

const mime = new Map([
  ['.css', 'text/css; charset=utf-8'],
  ['.js', 'text/javascript; charset=utf-8'],
  ['.json', 'application/json; charset=utf-8'],
  ['.png', 'image/png'],
  ['.svg', 'image/svg+xml'],
  ['.txt', 'text/plain; charset=utf-8'],
  ['.webmanifest', 'application/manifest+json; charset=utf-8'],
  ['.xml', 'application/xml; charset=utf-8'],
  ['.wasm', 'application/wasm'],
  ['.woff2', 'font/woff2'],
]);

const kokDosyalari = new Map([
  ['/robots.txt', 'robots.txt'],
  ['/sitemap.xml', 'sitemap.xml'],
  ['/manifest.webmanifest', 'manifest.webmanifest'],
]);

const slashGerektirenler = new Set([
  '/playground', '/dil', '/ogren', '/ornekler', '/belgeler', '/manifesto',
  '/yol-haritasi', '/topluluk', '/indir',
]);

function guvenlikBasliklari(yanit) {
  yanit.setHeader('X-Content-Type-Options', 'nosniff');
  yanit.setHeader('Referrer-Policy', 'strict-origin-when-cross-origin');
  yanit.setHeader('Permissions-Policy', 'camera=(), microphone=(), geolocation=()');
  yanit.setHeader('Cross-Origin-Opener-Policy', 'same-origin');
  yanit.setHeader('X-Frame-Options', 'DENY');
}

async function dosyayiSun(dosyaAdi, istek, yanit) {
  const guvenli = normalize(dosyaAdi).replace(/^(\.\.(\/|\\|$))+/, '');
  const dosyaYolu = join(statikKoku, guvenli);
  if (!dosyaYolu.startsWith(`${statikKoku}/`)) return false;

  try {
    const bilgi = await fs.stat(dosyaYolu);
    if (!bilgi.isFile()) return false;
    yanit.statusCode = 200;
    yanit.setHeader('Content-Type', mime.get(extname(dosyaYolu)) ?? 'application/octet-stream');
    yanit.setHeader('Content-Length', bilgi.size);
    yanit.setHeader('Cache-Control', 'public, max-age=3600');
    guvenlikBasliklari(yanit);
    if (istek.method === 'HEAD') yanit.end();
    else createReadStream(dosyaYolu).pipe(yanit);
    return true;
  } catch {
    return false;
  }
}

async function belgeyiSun(dosyaAdi, istek, yanit) {
  const dosyaYolu = join(belgeKoku, dosyaAdi);
  if (!dosyaYolu.startsWith(`${belgeKoku}/`)) return false;
  try {
    const bilgi = await fs.stat(dosyaYolu);
    if (!bilgi.isFile()) return false;
    yanit.statusCode = 200;
    yanit.setHeader('Content-Type', 'text/html; charset=utf-8');
    yanit.setHeader('Content-Length', bilgi.size);
    yanit.setHeader('Content-Security-Policy', yerelCsp);
    guvenlikBasliklari(yanit);
    if (istek.method === 'HEAD') yanit.end();
    else createReadStream(dosyaYolu).pipe(yanit);
    return true;
  } catch {
    return false;
  }
}

function isciyeAktar(istek, yanit) {
  const aktarim = httpRequest({
    hostname: '127.0.0.1',
    port: isciKapisi,
    method: istek.method === 'HEAD' ? 'GET' : istek.method,
    path: istek.url,
    headers: {
      ...istek.headers,
      host: `127.0.0.1:${isciKapisi}`,
      'x-forwarded-for': istek.socket.remoteAddress ?? '127.0.0.1',
      'x-forwarded-proto': 'http',
    },
  }, (isciYaniti) => {
    for (const [ad, deger] of Object.entries(isciYaniti.headers)) {
      if (deger !== undefined) yanit.setHeader(ad, deger);
    }
    const yol = new URL(istek.url ?? '/', 'http://127.0.0.1').pathname;
    if (!['/canli', '/hazir'].includes(yol)) {
      yanit.setHeader('Content-Type', 'text/html; charset=utf-8');
      yanit.setHeader('Content-Security-Policy', yerelCsp);
    }
    guvenlikBasliklari(yanit);
    yanit.writeHead(isciYaniti.statusCode ?? 502);
    if (istek.method === 'HEAD') {
      isciYaniti.resume();
      yanit.end();
    } else {
      isciYaniti.pipe(yanit);
    }
  });
  aktarim.on('error', () => {
    yanit.writeHead(502, { 'Content-Type': 'text/plain; charset=utf-8' });
    yanit.end('Zee worker hazır değil.');
  });
  istek.pipe(aktarim);
}

const sunucu = createServer(async (istek, yanit) => {
  const adres = new URL(istek.url ?? '/', `http://${istek.headers.host ?? '127.0.0.1'}`);
  if (!['GET', 'HEAD'].includes(istek.method ?? '')) {
    isciyeAktar(istek, yanit);
    return;
  }

  if (slashGerektirenler.has(adres.pathname)) {
    yanit.writeHead(308, { Location: `${adres.pathname}/${adres.search}` });
    yanit.end();
    return;
  }

  if (adres.pathname === '/belge') {
    yanit.writeHead(308, { Location: `/belge/${adres.search}` });
    yanit.end();
    return;
  }
  const belge = /^\/belge\/(?:([a-z0-9-]+)(\/)?)?$/.exec(adres.pathname);
  if (belge) {
    if (belge[1] && !belge[2]) {
      yanit.writeHead(308, { Location: `/belge/${belge[1]}/${adres.search}` });
      yanit.end();
      return;
    }
    if (await belgeyiSun(belge[1] ? `${belge[1]}.html` : 'index.html', istek, yanit)) return;
    yanit.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' });
    yanit.end('Belge bulunamadı. Önce scripts/belgeleri-uret.sh çalıştır.');
    return;
  }

  const kokDosyasi = kokDosyalari.get(adres.pathname);
  if (kokDosyasi && await dosyayiSun(kokDosyasi, istek, yanit)) return;

  if (adres.pathname.startsWith('/assets/')) {
    if (await dosyayiSun(adres.pathname.slice('/assets/'.length), istek, yanit)) return;
    yanit.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' });
    yanit.end('Bulunamadı.');
    return;
  }

  isciyeAktar(istek, yanit);
});

sunucu.listen(kapi, '127.0.0.1', () => {
  process.stdout.write(`zeedili.com yerel ön kapı: http://127.0.0.1:${kapi}\n`);
});
