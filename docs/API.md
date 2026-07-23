# Documentazione Tecnica - RaiPlayFeed RSS Generator

## Modalità Batch

RaiPlayFeed è ora un generatore batch che crea feed RSS locali invece di esporre un server web.

## Configurazione

La configurazione avviene tramite:

1. **File `config.yaml`** (nella directory corrente)
2. **Variabili d'ambiente** (prefisso `RAIPLAYRSS_`)
3. **File `urls.txt`** (un URL per riga)

### Configurazione Principale (`config.yaml`)

```yaml
# Configurazione batch
batch:
  output_dir: "./output"          # Directory di output per i feed RSS
  urls_file: "urls.txt"           # File con lista URL da processare
  urls: []                          # Lista URL alternativa al file
  template_file: "templates/index.html.tera"  # File template HTML
  index_file: "index.html"        # Nome file index HTML

# Configurazione upstream
upstream:
  base_url: "https://www.raiplaysound.it"  # URL base RaiPlaySound
  timeout_secs: 10                # Timeout richieste in secondi
  user_agent: "RaiPlayFeed/1.0"  # User-Agent per le richieste

# Configurazione cache (usata solo in modalità server legacy)
cache:
  ttl_secs: 300                   # TTL cache in secondi
```

### Variabili d'ambiente

| Variabile | Descrizione | Esempio |
|------------|-------------|---------|
| `RAIPLAYRSS_BATCH_OUTPUT_DIR` | Directory output | `./output` |
| `RAIPLAYRSS_BATCH_URLS_FILE` | File con lista URL | `urls.txt` |
| `RAIPLAYRSS_BATCH_TEMPLATE_FILE` | File template HTML | `templates/index.html.tera` |
| `RAIPLAYRSS_BATCH_INDEX_FILE` | Nome file index HTML | `index.html` |
| `RAIPLAYRSS_UPSTREAM_BASE_URL` | URL base RaiPlaySound | `https://www.raiplaysound.it` |
| `RAIPLAYRSS_UPSTREAM_TIMEOUT_SECS` | Timeout richieste | `15` |
| `RAIPLAYRSS_UPSTREAM_USER_AGENT` | User-Agent | `MioBot/1.0` |

---

## Formati URL Accettati

Il file `urls.txt` o la configurazione `urls` accettano diversi formati:

| Formato | Esempio | Descrizione |
|---------|---------|-------------|
| Solo nome | `belve` | Default: `programmi/belve` |
| Sezione/nome | `programmi/belve` | Percorso completo |
| Sezione/nome | `playlist/erastava` | Playlist |
| URL completo | `https://www.raiplaysound.it/programmi/belve` | URL assoluto |

## Struttura Output

Dopo l'esecuzione, nella directory di output troverai:

```
./output/
├── belve.xml            # Feed RSS per il programma "belve"
├── erastava.xml         # Feed RSS per la playlist "erastava"
├── index.html           # Index HTML con tutti i feed disponibili
└── ...                  # Altri feed
```

## Struttura Feed RSS

### Channel (Metadati Podcast)

| Elemento | Descrizione | Fonte RaiPlaySound |
|----------|-------------|-------------------|
| `<title>` | Titolo programma | `podcast_info.title` |
| `<link>` | URL programma web | `podcast_info.weblink` |
| `<description>` | Descrizione breve | `podcast_info.description` |
| `<language>` | Lingua | `it` (hardcoded) |
| `<copyright>` | Copyright | `© Rai - {title}` |
| `<ttl>` | Time to live (minuti) | `300` |
| `<image>` | Immagine copertina | `podcast_info.image` |
| `<itunes:author>` | Autore iTunes | `Rai Radio` |
| `<itunes:owner>` | Proprietario iTunes | Rai / podcast@rai.it |
| `<itunes:image>` | Immagine iTunes | `podcast_info.image` |
| `<itunes:explicit>` | Contenuto esplicito | `no` |
| `<itunes:category>` | Categorie iTunes | `podcast_info.genres[]` + `subgenres[]` |

### Item (Episodio)

| Elemento | Descrizione | Fonte |
|----------|-------------|-------|
| `<title>` | Titolo episodio | `{toptitle} - {episode_title}` |
| `<link>` | URL pagina episodio | `weblink` |
| `<description>` | Descrizione + metadati | `description` + `track_info` |
| `<pubDate>` | Data pubblicazione RFC2822 | `literal_publication_date` / `track_info.date` |
| `<guid>` | Identificativo univoco | `uniquename` (isPermaLink=false) |
| `<enclosure>` | File audio | `downloadable_audio.url`, type=audio/mpeg |
| `<itunes:duration>` | Durata | `duration_small_format` |
| `<itunes:season>` | Numero stagione | `track_info.season` |
| `<itunes:episode>` | Numero episodio | `track_info.episode_number` |
| `<itunes:image>` | Copertina episodio | `image` |
| `<itunes:explicit>` | Esplicito | `no` |

---

## Esempi Completi

### Feed Belve (estratto)
```xml
<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0"
     xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd"
     xmlns:content="http://purl.org/rss/1.0/modules/content/">
  <channel>
    <title>Belve</title>
    <link>https://www.raiplaysound.it/programmi/belve</link>
    <description>Francesca Fagnani si confronta senza sconti con grandi nomi...</description>
    <language>it</language>
    <copyright>© Rai - Belve</copyright>
    <ttl>300</ttl>
    <image>
      <url>https://www.raiplaysound.it/dl/img/2026/04/07/1775571012577_belve-2048x2048.jpg</url>
      <title>Belve</title>
      <link>https://www.raiplaysound.it/programmi/belve</link>
    </image>
    <itunes:author>Rai Radio</itunes:author>
    <itunes:owner>
      <itunes:name>Rai</itunes:name>
      <itunes:email>podcast@rai.it</itunes:email>
    </itunes:owner>
    <itunes:image href="https://www.raiplaysound.it/dl/img/2026/04/07/1775571012577_belve-2048x2048.jpg"/>
    <itunes:explicit>no</itunes:explicit>
    <itunes:category text="Attualità"/>
    <itunes:category text="Interviste"/>
    <itunes:category text="Storie vere"/>
    <itunes:category text="Personaggi"/>
    <item>
      <title>Elena Santarelli - Elena Santarelli</title>
      <link>https://www.raiplaysound.it/audio/2026/04/Elena-Santarelli---Belve-28042026-040cb161-13c7-4146-ab62-d51c272fac5b.html</link>
      <description><![CDATA[Elena Santarelli si racconta a Francesca Fagnani: l'amore per il marito Bernardo Corradi, la carriera in tv e la felicità per la guarigione del figlio dopo una grave malattia.

Stagione: 2026
Edizione: 2026
Durata: 47 min
Formato: Integrale]]></description>
      <pubDate>Tue, 28 Apr 2026 00:00:00 +0000</pubDate>
      <guid isPermaLink="false">ContentItem-040cb161-13c7-4146-ab62-d51c272fac5b</guid>
      <enclosure url="https://mediapolisvod.rai.it/relinker/relinkerServlet.htm?cont=NJypPpPlussX1Hv8ga9Eaj40QesgAeeqqEEqualeeqqEEqual" type="audio/mpeg" length="45926400"/>
      <itunes:duration>47:54</itunes:duration>
      <itunes:season>2026</itunes:season>
      <itunes:episode>0</itunes:episode>
      <itunes:image href="https://www.raiplaysound.it/dl/img/2026/04/29/1777493800388_Elena_Santarelli_2048x2048.jpg"/>
      <itunes:explicit>no</itunes:explicit>
    </item>
    <!-- ... altri episodi ... -->
  </channel>
</rss>
```

---

## Struttura Index HTML

Il file `index.html` generato contiene:

- Titolo e descrizione di ogni feed
- Immagine di copertina
- Data dell'ultimo episodio
- Numero di episodi
- Link diretto al feed RSS
- Link alla pagina RaiPlaySound
- Design responsive con tema scuro

## Template Engine

RaiPlayFeed utilizza **Tera** come template engine (sintassi Jinja2-like) per generare l'`index.html`.

### Variabili disponibili nel template:

```jinja2
{
  "generated_at": "22/07/2026 21:15",  # Data generazione
  "feeds": [
    {
      "title": "Belve",
      "description": "Francesca Fagnani si confronta senza sconti...",
      "image": "https://www.raiplaysound.it/dl/img/.../belve-2048x2048.jpg",
      "weblink": "https://www.raiplaysound.it/programmi/belve",
      "rss_url": "belve.xml",
      "section": "programmi",
      "latest_episode_date": "28/04/2026",
      "episode_count": 124
    }
  ]
}
```

### Personalizzazione template:

1. Crea un file template personalizzato (es. `templates/custom.html.tera`)
2. Configuralo in `config.yaml`:
   ```yaml
   batch:
     template_file: "templates/custom.html.tera"
   ```

## Errori e Log

| Tipo Errore | Causa | Azione |
|-------------|-------|--------|
| `Program not found` | Programma inesistente | Verificare slug |
| `Request failed` | Errore API RaiPlaySound | Riprovare più tardi |
| `Request timeout` | Timeout API | Aumentare timeout o riprovare |
| `URL parse error` | URL malformato | Verificare URL in `urls.txt` |
| `I/O error` | Problema scrittura file | Verificare permessi directory output |

## Log

```bash
# Log dettagliati
RUST_LOG=debug cargo run

# Solo errori
RUST_LOG=error cargo run

# Log specifici
RUST_LOG=raiplayfeed=debug,reqwest=warn cargo run
```

## Limitazioni Note

1. **Audio URL**: Alcuni episodi hanno `login_required: true` - l'URL potrebbe richiedere cookie di sessione RaiPlay
2. **Immagini**: URL relativi convertiti in assoluti (`https://www.raiplaysound.it/...`)
3. **Date**: Parsing supporta formati "DD MMM YYYY" e "YYYY-MM-DD"
4. **Durata**: Stima lunghezza enclosure da durata (128 kbps assumed)

## Esempi di Utilizzo

### Generazione feed per più programmi

```bash
# Crea urls.txt
cat > urls.txt <<EOF
belve
il-ruggito-del-coniglio
caterpillar
erastava
EOF

# Esegui il generatore
cargo run

# Output atteso:
# ./output/
# ├── belve.xml
# ├── il-ruggito-del-coniglio.xml
# ├── caterpillar.xml
# ├── erastava.xml
# └── index.html
```

### Utilizzo con variabili d'ambiente

```bash
# Configura via variabili d'ambiente
export RAIPLAYRSS_BATCH_OUTPUT_DIR="./my_feeds"
export RAIPLAYRSS_UPSTREAM_TIMEOUT_SECS=20

# Crea directory output
mkdir -p ./my_feeds

# Esegui
cargo run
```

### Servire i feed generati

I feed RSS generati possono essere serviti con qualsiasi web server:

```bash
# Usando Python
cd ./output
python3 -m http.server 8000

# Ora i feed sono disponibili su:
# http://localhost:8000/belve.xml
# http://localhost:8000/index.html
```

## Integrazione con Client Podcast

1. Esegui RaiPlayFeed per generare i feed
2. Apri `index.html` nel browser
3. Copia il link RSS (pulsante RSS) nel tuo client preferito:

| Client | Come aggiungere |
|--------|-----------------|
| Apple Podcasts | File → Aggiungi URL feed |
| Spotify | Cerca "Aggiungi podcast" → Incolla URL |
| Pocket Casts | + → Aggiungi feed RSS |
| AntennaPod | + → Aggiungi feed |
| Podcast Addict | + → Aggiungi feed RSS |
