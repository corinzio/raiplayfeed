use config::{Config, ConfigError, Environment, File};
use std::time::Duration;

/// Configurazione completa dell'applicazione
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub upstream: UpstreamConfig,
    pub batch: BatchConfig,
}

/// Configurazione delle chiamate upstream verso RaiPlaySound
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpstreamConfig {
    /// URL base di RaiPlaySound (es. "https://www.raiplaysound.it")
    pub base_url: String,
    /// Timeout richieste in secondi
    pub timeout_secs: u64,
    /// User-Agent per le richieste HTTP
    pub user_agent: String,
}

/// Configurazione per la modalità batch
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchConfig {
    /// Directory di output per i feed RSS generati
    pub output_dir: String,
    /// File contenente la lista di URL da processare (uno per riga)
    pub urls_file: Option<String>,
    /// Lista di URL da processare (alternativa al file)
    pub urls: Option<Vec<String>>,
    /// File template per la generazione dell'index.html
    pub template_file: Option<String>,
    /// Nome del file index.html di output
    pub index_file: String,
}

impl AppConfig {
    /// Carica la configurazione da file e variabili d'ambiente.
    ///
    /// Priorità (dalla più alta alla più bassa):
    /// 1. Variabili d'ambiente con prefisso `RAIPLAYRSS_`
    /// 2. File `config.yaml` nella directory corrente
    /// 3. Valori di default
    ///
    /// # Variabili d'ambiente supportate
    ///
    /// | Variabile | Descrizione | Esempio |
    /// |-----------|-------------|---------|
    /// | `RAIPLAYRSS_UPSTREAM_BASE_URL` | URL RaiPlaySound | `https://www.raiplaysound.it` |
    /// | `RAIPLAYRSS_UPSTREAM_TIMEOUT_SECS` | Timeout secondi | `15` |
    /// | `RAIPLAYRSS_UPSTREAM_USER_AGENT` | User-Agent | `MioBot/1.0` |
    /// | `RAIPLAYRSS_BATCH_OUTPUT_DIR` | Directory output | `./output` |
    /// | `RAIPLAYRSS_BATCH_URLS_FILE` | File con lista URL | `urls.txt` |
    /// | `RAIPLAYRSS_BATCH_TEMPLATE_FILE` | File template HTML | `templates/index.html.tera` |
    /// | `RAIPLAYRSS_BATCH_INDEX_FILE` | File index HTML | `index.html` |
    /// | `RUST_LOG` | Livello logging | `debug` |
    pub fn load() -> Result<Self, ConfigError> {
        let config = Config::builder()
            // 1. Valori di default
            .add_source(
                Config::try_from(&default_config())
                    .map_err(|e| ConfigError::Message(format!("Default config error: {e}")))?,
            )
            // 2. File config.yaml (opzionale)
            .add_source(File::with_name("config").required(false))
            // 3. Variabili d'ambiente con prefisso RAIPLAYRSS_
            .add_source(
                Environment::with_prefix("RAIPLAYRSS")
                    .separator("_")
                    .try_parsing(true),
            )
            .build()?;

        config.try_deserialize()
    }

    /// Restituisce il timeout come Duration
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.upstream.timeout_secs)
    }
}

/// Configurazione di default
fn default_config() -> AppConfig {
    AppConfig {
        upstream: UpstreamConfig {
            base_url: "https://www.raiplaysound.it".to_string(),
            timeout_secs: 10,
            user_agent: "Mozilla/5.0 (compatible; RaiPlayRSS/1.0)".to_string(),
        },
        batch: BatchConfig {
            output_dir: "./output".to_string(),
            urls_file: Some("urls.txt".to_string()),
            urls: None,
            template_file: Some("templates/index.html.tera".to_string()),
            index_file: "index.html".to_string(),
        },
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        default_config()
    }
}
