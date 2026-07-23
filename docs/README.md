# RaiPlaySound RSS Generator

Un servizio web in Rust che genera feed RSS per i podcast di RaiPlaySound.it, che non li fornisce nativamente.

## Panoramica

RaiPlaySound ospita numerosi programmi radiofonici e podcast, ma non espone feed RSS standard. Questo servizio colma la lacuna interrogando le API JSON interne di RaiPlaySound e generando feed RSS 2.0 compatibili con i principali client podcast (Apple Podcasts, Spotify, Pocket Casts, ecc.).

## Architettura

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Client    │────▶│  RaiPlayRSS      │────▶│  RaiPlaySound   │
│  (Podcast   │     │  (axum server)   │     │  API (JSON)     │
│   App)      │◀────│  Port 3000       │◀────│                 │
└─────────────┘     └──────────────────┘     └─────────────────┘
                           │
                    ┌──────┴──────┐
                    │  Cache      │
                    │  (5 min)    │
                    └─────────────┘
```

### Componenti principali

- **axum** - Framework web asincrono
- **reqwest** - Client HTTP per chiamare le API RaiPlaySound
- **rss** - Generazione feed RSS 2.0 con estensioni iTunes
- **tokio** - Runtime asincrono
- **serde** - Serializzazione/deserializzazione JSON

## Endpoint API

### Health Check
```
GET /health
```
Ritorna `OK` se il servizio è attivo.

### Feed RSS Programma
```
GET /programmi/:program_name
GET /programmi/:program_name.xml
```

Esempi:
- `http://localhost:3000/programmi/belve`
- `http://localhost:3000/programmi/belve.xml`

### Feed RSS Playlist
Alcuni contenuti RaiPlaySound sono pubblicati come **playlist** invece che come programmi,
quindi risiedono sotto il percorso `/playlist/` anziché `/programmi/`.
```
GET /playlist/:program_name
GET /playlist/:program_name.xml
```

Esempi:
- `http://localhost:3000/playlist/erastava`
- `http://localhost:3000/playlist/erastava.xml`

> **Nota:** lo `slug` da usare nell'URL del feed deve corrispondere allo slug reale
> della pagina RaiPlaySound (quello presente nell'URL della pagina web), non una sua
> versione abbreviata. Ad esempio la playlist "Era Stava" si trova all'URL
> `https://www.raiplaysound.it/playlist/erastava`, quindi il feed è
> `/playlist/erastava.xml` (che recupera `https://www.raiplaysound.it/playlist/erastava.json`).

## Funzionamento interno

### 1. Recupero dati programma
Il servizio chiama `https://www.raiplaysound.it/programmi/{name}.json` (per i programmi)
oppure `https://www.raiplaysound.it/playlist/{name}.json` (per le playlist) per ottenere:
- Metadati del podcast (titolo, descrizione, immagine, generi)
- Lista episodi della stagione corrente
- Riferimenti alle stagioni precedenti (tramite `filters`)

### 2. Recupero stagioni precedenti
Per ogni stagione non attiva nei `filters`, viene fatta una richiesta aggiuntiva a:
`https://www.raiplaysound.it/programmi/{name}/ContentSet-{id}.json`

### 3. Costruzione episodi
Per ogni episodio vengono estratti:
- **GUID**: `uniquename` univoco dell'episodio
- **Titolo**: `{toptitle} - {episode_title}`
- **Descrizione**: Descrizione episodio + metadati (stagione, edizione, durata, formato)
- **Link**: URL pagina web dell'episodio
- **Data pubblicazione**: Da `literal_publication_date` o `track_info.date`
- **Enclosure**: URL audio scaricabile (`downloadable_audio.url`)
- **Durata**: Formato `MM:SS` o `HH:MM:SS`
- **Immagine**: Copertina episodio
- **Estensioni iTunes**: Stagione, numero episodio, durata, immagine, explicit

### 4. Ordinamento e deduplicazione
Gli episodi vengono ordinati per data di pubblicazione (più recenti prima) e deduplicati tramite GUID.

### 5. Cache
I feed RSS generati vengono cachati per 5 minuti (configurabile via `CACHE_TTL`) per ridurre le chiamate alle API RaiPlaySound.

## Struttura progetto

```
raiplayrss/
├── Cargo.toml
├── src/
│   └── main.rs          # Entry point + tutta la logica
├── docs/
│   ├── README.md        # Questa documentazione
│   └── FUTURE.md        # Sviluppi futuri
└── target/
    └── debug/raiplayrss # Binario compilato
```

## Modelli dati principali

### ProgramPage
Risposta principale da `/programmi/{name}.json`
- `podcast_info`: Metadati programma
- `block.cards`: Episodi stagione corrente
- `filters[]`: Stagioni disponibili con `path_id` per fetch aggiuntivi

### Card (Episodio)
- `uniquename`: GUID univoco
- `toptitle`: Nome ospite/titolo principale
- `episode_title`: Titolo episodio
- `description`: Descrizione completa
- `downloadable_audio.url`: URL file audio
- `literal_publication_date`: Data pubblicazione (es. "28 Apr 2026")
- `duration_small_format`: Durata (es. "47:54")
- `track_info`: Metadati aggiuntivi (stagione, edizione, numero episodio)

## Configurazione

Variabili d'ambiente:
| Variabile | Default | Descrizione |
|-----------|---------|-------------|
| `RUST_LOG` | `info,raiplayrss=debug` | Livello logging |
| `CACHE_TTL` | `300` | Secondi cache feed (non ancora implementato come env var) |

## Avvio

```bash
# Sviluppo
cargo run

# Produzione
cargo build --release
./target/release/raiplayrss
```

Il server ascolta su `0.0.0.0:3000`.

## Test manuale

```bash
# Verifica servizio
curl http://localhost:3000/health

# Feed RSS per "Belve"
curl http://localhost:3000/programmi/belve.xml | head -50

# Feed RSS per altro programma (es. "il-ruggito-del-coniglio")
curl http://localhost:3000/programmi/il-ruggito-del-coniglio.xml | head -50
```

## Limitazioni note

1. **Login required**: Alcuni episodi hanno `login_required: true` - l'URL audio potrebbe richiedere autenticazione
2. **Rate limiting**: Non implementato lato client verso RaiPlaySound
3. **Singola istanza**: Cache in memoria, non adatto a deploy multi-istanza senza Redis
4. **Error handling**: Errori stagioni precedenti vengono solo loggati, non bloccano il feed

## Licenza

MIT