# Quickstart - RaiPlayFeed RSS Generator

## Prerequisiti

- **Rust 1.75+** (installare via [rustup](https://rustup.rs/))
- Accesso internet per API RaiPlaySound

## 5 Minuti per il Primo Feed

### Modalità Batch (consigliata)

```bash
# 1. Clone e build
git clone https://github.com/tuo-utente/raiplayfeed
cd raiplayfeed

# 2. Crea un file urls.txt con i programmi da processare
# Formati accettati:
#   - belve (default: programmi)
#   - programmi/belve
#   - playlist/erastava
#   - https://www.raiplaysound.it/programmi/belve

echo "belve" > urls.txt

# 3. Esegui il generatore
cargo run
# Output atteso:
# INFO raiplayfeed: RaiPlayFeed - Generatore batch di feed RSS
# INFO raiplayfeed:   Upstream: https://www.raiplaysound.it
# INFO raiplayfeed:   Output dir: ./output
# INFO raiplayfeed: Trovati 1 feed da processare
# INFO raiplayfeed: Processando: programmi/belve
# INFO raiplayfeed:   Scritto: ./output/belve.xml
# INFO raiplayfeed:   OK: 124 episodi, ultimo: 28/04/2026
# INFO raiplayfeed: Index generato: ./output/index.html

# 4. Visualizza i risultati
ls -la ./output/
# -rw-r--r-- 1 user user 129034 lug 22 21:15 belve.xml
# -rw-r--r-- 1 user user  9599 lug 22 21:15 index.html

# 5. Apri index.html nel browser per vedere tutti i feed disponibili
xdg-open ./output/index.html
```

### Modalità Server Web (legacy)

La modalità server web è ancora disponibile ma non più consigliata:

```bash
# Avvia il server web
cargo run -- --server

# Test in nuovo terminale
curl http://localhost:3000/programmi/belve.xml | head -30
```

## Aggiungere al Tuo Client Podcast

Copia l'URL del feed e incollalo nel tuo client:

| Client | Come aggiungere |
|--------|-----------------|
| **Apple Podcasts** | File → Aggiungi URL → `http://TUO-SERVER:3000/programmi/belve.xml` |
| **Pocket Casts** | ➕ → Aggiungi tramite URL → Incolla URL |
| **Spotify for Podcasters** | Aggiungi show → RSS Feed → Incolla URL |
| **Overcast** | ➕ → Add Podcast URL → Incolla URL |
| **AntennaPod (Android)** | ➕ → Inserisci URL feed → Incolla URL |
| **gPodder** | Subscription → Add → URL → Incolla URL |

## Trovare Altri Programmi e Playlist

1. Vai su [raiplaysound.it](https://www.raiplaysound.it)
2. Cerca il programma o la playlist desiderata
3. Copia lo **slug** dall'URL:
   - Programma: `https://www.raiplaysound.it/programmi/**belve**`
   - Playlist:  `https://www.raiplaysound.it/playlist/**erastava**`
4. Usa rispettivamente:
   - `http://localhost:3000/programmi/belve.xml`
   - `http://localhost:3000/playlist/erastava.xml`

> **Nota:** per le playlist usa lo slug completo presente nell'URL della pagina
> (es. `erastava`), non una versione abbreviata.

Esempi popolari (programmi):
```
http://localhost:3000/programmi/il-ruggito-del-coniglio.xml
http://localhost:3000/programmi/la-zanzara.xml
http://localhost:3000/programmi/caterpillar.xml
http://localhost:3000/programmi/prime-time.xml
http://localhost:3000/programmi/voci-dalla-notte.xml
```

Esempi playlist:
```
http://localhost:3000/playlist/erastava.xml
```

## Risoluzione Problemi

| Problema | Soluzione |
|----------|-----------|
| `Program not found` | Verifica slug esatto su raiplaysound.it |
| `Upstream request failed` | Controlla connessione internet / API RaiPlaySound down |
| Feed vuoto | Alcuni programmi potrebbero non avere episodi pubblici |
| Errore SSL | Verifica certificati ca-certificates installati |
| Timeout | Aumenta `REQUEST_TIMEOUT` in main.rs |

## Log Debug

```bash
# Log dettagliati
RUST_LOG=debug ./target/release/raiplayrss

# Solo errori
RUST_LOG=error ./target/release/raiplayrss

# Filtra modulo
RUST_LOG=raiplayrss=debug,reqwest=warn ./target/release/raiplayrss
```

## Struttura Feed RSS Generato

```xml
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
  <channel>
    <!-- Metadati canale -->
    <title>Nome Programma</title>
    <link>https://www.raiplaysound.it/programmi/...</link>
    <description>Descrizione programma</description>
    <language>it</language>
    <copyright>© Rai - Nome Programma</copyright>
    <ttl>300</ttl>
    <image>...</image>
    
    <!-- iTunes extensions -->
    <itunes:author>Rai Radio</itunes:author>
    <itunes:owner><itunes:name>Rai</itunes:name><itunes:email>podcast@rai.it</itunes:email></itunes:owner>
    <itunes:image href="https://..."/>
    <itunes:explicit>no</itunes:explicit>
    <itunes:category text="Genere Principale"/>
    <itunes:category text="Sottogenere"/>
    
    <!-- Episodi (ordinati per data decrescente) -->
    <item>
      <title>Ospite - Titolo Episodio</title>
      <link>https://www.raiplaysound.it/audio/...</link>
      <description>Descrizione + metadati</description>
      <pubDate>Tue, 28 Apr 2026 00:00:00 +0000</pubDate>
      <guid isPermaLink="false">ContentItem-uuid</guid>
      <enclosure url="https://mediapolisvod.rai.it/..." type="audio/mpeg" length="..."/>
      <itunes:author>Rai Radio</itunes:author>
      <itunes:image href="https://..."/>
      <itunes:duration>47:54</itunes:duration>
      <itunes:season>10</itunes:season>
      <itunes:episode>1</itunes:episode>
      <itunes:explicit>no</itunes:explicit>
    </item>
    ...
  </channel>
</rss>
```

## Prossimi Passi

- Leggi [API.md](docs/API.md) per riferimento completo
