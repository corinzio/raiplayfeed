# RaiPlayFeed - RSS Generator per RaiPlaySound

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

Generatore di feed RSS 2.0 (con estensioni iTunes) per i podcast di **RaiPlaySound.it**, che non li fornisce nativamente.

## 🚀 Modalità di utilizzo

RaiPlayFeed ora supporta **due modalità di utilizzo**:

### 1. Modalità Batch (consigliata) - Generazione locale di feed RSS

```bash
# Clona il repository
git clone https://github.com/tuo-utente/raiplayfeed
cd raiplayfeed

# Crea un file urls.txt con i programmi/playlist da processare
# Esempio: belve, erastava, il-ruggito-del-coniglio
nano urls.txt

# Esegui il generatore
cargo run

# I feed RSS verranno generati nella directory ./output/
# Apri index.html nel browser per vedere tutti i feed disponibili
```

### 2. Modalità Server Web (legacy)

La modalità server web è ancora disponibile ma non più consigliata. Per utilizzarla:

```bash
# Avvia il server web
cargo run -- --server
```

## 📖 Documentazione

| Documento | Descrizione |
|-----------|-------------|
| [docs/README.md](docs/README.md) | Documentazione completa |
| [docs/QUICKSTART.md](docs/QUICKSTART.md) | Guida rapida 5 minuti |
| [docs/FUTURE.md](docs/FUTURE.md) | Roadmap e sviluppi futuri |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Come contribuire |

## 🎯 Caratteristiche

- ✅ Feed RSS 2.0 standard + estensioni **iTunes/Apple Podcasts**
- ✅ Recupero automatico **tutte le stagioni** (non solo quella corrente)
- ✅ **Deduplicazione** episodi tramite GUID univoco
- ✅ Ordinamento **più recenti prima**
- ✅ Metadati completi: durata, stagione, episodio, copertina, descrizione
- ✅ **Generazione batch** di feed RSS locali
- ✅ **Index HTML** con tutti i feed disponibili (stile simile a raiplaysound.github.io)
- ✅ **Template engine** Tera (Jinja2-like) per la generazione HTML
- ✅ Configurazione flessibile via file YAML o variabili d'ambiente
- ✅ Zero dipendenze esterne (DB, Redis) per deploy semplice

## 📂 Output generato

Dopo l'esecuzione, nella directory `./output/` troverai:

- File `.xml` per ogni programma/playlist (es. `belve.xml`, `erastava.xml`)
- File `index.html` con l'elenco di tutti i feed disponibili
- Per ogni feed: titolo, descrizione, immagine, data ultimo episodio, numero episodi

## 🛠 Configurazione

La configurazione può essere personalizzata tramite:

1. **File `config.yaml`** (nella directory corrente)
2. **Variabili d'ambiente** (prefisso `RAIPLAYRSS_`)
3. **File `urls.txt`** (un URL per riga)

Esempio di configurazione (`config.yaml`):

```yaml
batch:
  output_dir: "./output"          # Directory di output
  urls_file: "urls.txt"           # File con lista URL
  index_file: "index.html"        # Nome file index HTML
  template_file: "templates/index.html.tera"  # File template

upstream:
  base_url: "https://www.raiplaysound.it"  # URL base RaiPlaySound
  timeout_secs: 10                # Timeout richieste
  user_agent: "RaiPlayFeed/1.0"  # User-Agent
```

## 🎧 Utilizzo nel Client Podcast

1. Esegui RaiPlayFeed per generare i feed RSS
2. Apri `index.html` nella directory di output
3. Copia il link RSS (pulsante RSS) nel tuo client preferito:

| Client | Come aggiungere |
|--------|-----------------|
| Apple Podcasts | File → Aggiungi feed URL |
| Spotify | Cerca "Aggiungi podcast" → Incolla URL |
| Pocket Casts | + → Aggiungi feed RSS |
| AntennaPod | + → Aggiungi feed |
| Podcast Addict | + → Aggiungi feed RSS |

## 📋 Requisiti

- Rust 1.70+ (`rustup default stable`)
- Connessione internet per API RaiPlaySound

## ⚠️ Limitazioni Note

- Alcuni episodi richiedono login (`login_required: true`)
- Rate limiting non implementato (usare con rispetto)
- Solo audio (nessun supporto video)
- La modalità server web è deprecata in favore della modalità batch

**Nota**: Questo progetto non è affiliato a Rai o RaiPlaySound. Utilizza API pubbliche non documentate. Usa con responsabilità e rispetto per i termini di servizio Rai.
