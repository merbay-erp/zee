// dillsp için elle yazılmış LSP istemcisi — npm bağımlılığı YOKTUR
// (proje felsefesi: ADR-001'in ruhu editör katmanında da sürer).
// Kapsam: tanılar, hover, tanıma git, tamamlama. Belge eşitleme: tam metin.

const vscode = require("vscode");
const { spawn } = require("child_process");

let surec = null;
let tanilar = null;
let siradaki_kimlik = 1;
const bekleyenler = new Map(); // kimlik -> resolve

function baslat(baglam) {
  const ayar = vscode.workspace.getConfiguration("dil");
  const yol = ayar.get("lspYolu", "dillsp");

  try {
    surec = spawn(yol, [], { stdio: ["pipe", "pipe", "pipe"] });
  } catch (hata) {
    vscode.window.showWarningMessage(`dillsp başlatılamadı: ${hata}`);
    return false;
  }
  surec.on("error", () => {
    vscode.window.showWarningMessage(
      "dillsp bulunamadı. compiler/ içinde `cargo build --release` çalıştırıp " +
        "dillsp'yi PATH'e ekle ya da `dil.lspYolu` ayarını ver."
    );
    surec = null;
  });

  // Gelen akış: Content-Length çerçeveleri.
  let tampon = Buffer.alloc(0);
  surec.stdout.on("data", (parca) => {
    tampon = Buffer.concat([tampon, parca]);
    for (;;) {
      const ayrac = tampon.indexOf("\r\n\r\n");
      if (ayrac < 0) return;
      const baslik = tampon.slice(0, ayrac).toString("utf8");
      const eslesme = /Content-Length: (\d+)/i.exec(baslik);
      if (!eslesme) {
        tampon = tampon.slice(ayrac + 4);
        continue;
      }
      const uzunluk = parseInt(eslesme[1], 10);
      if (tampon.length < ayrac + 4 + uzunluk) return;
      const govde = tampon.slice(ayrac + 4, ayrac + 4 + uzunluk).toString("utf8");
      tampon = tampon.slice(ayrac + 4 + uzunluk);
      mesaj_isle(JSON.parse(govde));
    }
  });

  gonder({ jsonrpc: "2.0", id: kimlik_al(), method: "initialize", params: {} });
  return true;
}

function kimlik_al() {
  return siradaki_kimlik++;
}

function gonder(mesaj) {
  if (!surec) return;
  const govde = JSON.stringify(mesaj);
  surec.stdin.write(`Content-Length: ${Buffer.byteLength(govde, "utf8")}\r\n\r\n${govde}`);
}

function iste(method, params) {
  return new Promise((resolve) => {
    if (!surec) return resolve(null);
    const id = kimlik_al();
    bekleyenler.set(id, resolve);
    gonder({ jsonrpc: "2.0", id, method, params });
  });
}

function mesaj_isle(mesaj) {
  if (mesaj.id !== undefined && bekleyenler.has(mesaj.id)) {
    bekleyenler.get(mesaj.id)(mesaj.result ?? null);
    bekleyenler.delete(mesaj.id);
    return;
  }
  if (mesaj.method === "textDocument/publishDiagnostics") {
    const { uri, diagnostics } = mesaj.params;
    tanilar.set(
      vscode.Uri.parse(uri),
      diagnostics.map((t) => {
        const tani = new vscode.Diagnostic(
          alan_cevir(t.range),
          t.message,
          vscode.DiagnosticSeverity.Error
        );
        tani.code = t.code;
        tani.source = "dil";
        return tani;
      })
    );
  }
}

function alan_cevir(alan) {
  return new vscode.Range(
    alan.start.line,
    alan.start.character,
    alan.end.line,
    alan.end.character
  );
}

function belge_bildir(method, belge, tamMetin) {
  const params = { textDocument: { uri: belge.uri.toString() } };
  if (tamMetin) params.textDocument.text = belge.getText();
  if (method === "textDocument/didChange") {
    params.contentChanges = [{ text: belge.getText() }];
  }
  gonder({ jsonrpc: "2.0", method, params });
}

function konum_params(belge, konum) {
  return {
    textDocument: { uri: belge.uri.toString() },
    position: { line: konum.line, character: konum.character },
  };
}

function activate(baglam) {
  tanilar = vscode.languages.createDiagnosticCollection("dil");
  baglam.subscriptions.push(tanilar);
  if (!baslat(baglam)) return;

  const dil_mi = (belge) => belge.languageId === "dil";

  for (const belge of vscode.workspace.textDocuments) {
    if (dil_mi(belge)) belge_bildir("textDocument/didOpen", belge, true);
  }
  baglam.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((belge) => {
      if (dil_mi(belge)) belge_bildir("textDocument/didOpen", belge, true);
    }),
    vscode.workspace.onDidChangeTextDocument((olay) => {
      if (dil_mi(olay.document)) belge_bildir("textDocument/didChange", olay.document);
    }),
    vscode.workspace.onDidCloseTextDocument((belge) => {
      if (dil_mi(belge)) {
        belge_bildir("textDocument/didClose", belge);
        tanilar.delete(belge.uri);
      }
    }),
    vscode.languages.registerHoverProvider("dil", {
      async provideHover(belge, konum) {
        const sonuc = await iste("textDocument/hover", konum_params(belge, konum));
        if (!sonuc || !sonuc.contents) return null;
        return new vscode.Hover(new vscode.MarkdownString(sonuc.contents.value));
      },
    }),
    vscode.languages.registerDefinitionProvider("dil", {
      async provideDefinition(belge, konum) {
        const sonuc = await iste("textDocument/definition", konum_params(belge, konum));
        if (!sonuc || !sonuc.uri) return null;
        return new vscode.Location(vscode.Uri.parse(sonuc.uri), alan_cevir(sonuc.range));
      },
    }),
    vscode.languages.registerRenameProvider("dil", {
      async provideRenameEdits(belge, konum, yeniAd) {
        const params = konum_params(belge, konum);
        params.newName = yeniAd;
        const sonuc = await iste("textDocument/rename", params);
        if (!sonuc || !sonuc.changes) return null;
        const duzenleme = new vscode.WorkspaceEdit();
        for (const [uri, degisiklikler] of Object.entries(sonuc.changes)) {
          for (const d of degisiklikler) {
            duzenleme.replace(vscode.Uri.parse(uri), alan_cevir(d.range), d.newText);
          }
        }
        return duzenleme;
      },
    }),
    vscode.languages.registerCompletionItemProvider("dil", {
      async provideCompletionItems(belge, konum) {
        const sonuc = await iste("textDocument/completion", konum_params(belge, konum));
        if (!Array.isArray(sonuc)) return null;
        return sonuc.map(
          (oge) => new vscode.CompletionItem(oge.label, vscode.CompletionItemKind.Keyword)
        );
      },
    })
  );
}

function deactivate() {
  if (surec) {
    gonder({ jsonrpc: "2.0", id: kimlik_al(), method: "shutdown" });
    gonder({ jsonrpc: "2.0", method: "exit" });
    surec = null;
  }
}

module.exports = { activate, deactivate };
