mod config;
mod models;

use crate::models::*;
use chrono::{FixedOffset, NaiveDate, NaiveDateTime, TimeZone};
use config::AppConfig;
use reqwest::Client;
use rss::{
    Category, ChannelBuilder, EnclosureBuilder, GuidBuilder, ImageBuilder, ItemBuilder,
    extension::itunes::{
        ITunesCategoryBuilder, ITunesChannelExtensionBuilder, ITunesItemExtensionBuilder,
        ITunesOwnerBuilder,
    },
};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};
use thiserror::Error;
use tokio::time::timeout;
use tracing::{error, info, warn};
use tracing_subscriber::prelude::*;
use url::Url;

const TEMPLATE_STR: &str = include_str!("../templates/index.html.tera");

#[derive(Error, Debug)]
enum AppError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Date parse error: {0}")]
    DateParse(#[from] chrono::ParseError),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("RSS generation error: {0}")]
    Rss(#[from] rss::Error),
    #[error("Request timeout: {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Program not found")]
    NotFound,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Template error: {0}")]
    Template(#[from] tera::Error),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Clone)]
struct FeedEntry {
    section: String,
    program_name: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct FeedSummary {
    title: String,
    description: String,
    image: String,
    weblink: String,
    rss_url: String,
    section: String,
    latest_episode_date: Option<String>,
    episode_count: usize,
}

fn parse_raiplay_url(url: &str) -> Option<(String, String)> {
    let parsed = Url::parse(url).ok()?;
    let path = parsed.path();
    let path = path.strip_suffix('/').unwrap_or(path);
    let path = path.strip_suffix(".json").unwrap_or(path);
    let path = path.strip_suffix(".xml").unwrap_or(path);
    let path = path.strip_suffix(".rss").unwrap_or(path);

    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() >= 2 {
        // La sezione è libera (es. "programmi", "playlist", "audiolibri", ...)
        // e viene usata solo per comporre l'URL upstream.
        let section = segments[segments.len() - 2].to_string();
        let name = segments[segments.len() - 1].to_string();
        return Some((section, name));
    }
    if segments.len() == 1 {
        warn!(
            "URL {} ha un solo segmento, impossibile determinare la sezione. Provo 'programmi' come default.",
            url
        );
        return Some(("programmi".to_string(), segments[0].to_string()));
    }
    None
}

fn parse_url_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    if line.starts_with("http://") || line.starts_with("https://") {
        return parse_raiplay_url(line);
    }

    // Formato: sezione/nome (es. "programmi/belve", "playlist/erastava", "audiolibri/preghierapercernobyl").
    // La sezione è libera e viene usata solo per comporre l'URL upstream.
    if let Some((section, name)) = line.split_once('/') {
        let section = section.trim().to_lowercase();
        let name = name.trim().to_string();
        if !section.is_empty() && !name.is_empty() {
            return Some((section, name));
        }
    }

    // Solo nome, default "programmi"
    Some(("programmi".to_string(), line.to_string()))
}

fn load_urls(config: &AppConfig) -> Vec<FeedEntry> {
    if let Some(ref urls) = config.batch.urls {
        if !urls.is_empty() {
            info!("Caricamento URL dalla configurazione");
            return urls
                .iter()
                .filter_map(|url| {
                    parse_url_line(url).map(|(s, n)| FeedEntry {
                        section: s,
                        program_name: n,
                    })
                })
                .collect();
        }
    }

    if let Some(ref urls_file) = config.batch.urls_file {
        if Path::new(urls_file).exists() {
            info!("Caricamento URL da file: {}", urls_file);
            match fs::read_to_string(urls_file) {
                Ok(content) => {
                    return content
                        .lines()
                        .filter_map(|line| {
                            parse_url_line(line).map(|(s, n)| FeedEntry {
                                section: s,
                                program_name: n,
                            })
                        })
                        .collect();
                }
                Err(e) => {
                    error!("Errore nella lettura del file {}: {}", urls_file, e);
                }
            }
        } else {
            warn!("File URL non trovato: {}", urls_file);
        }
    }

    error!("Nessun URL configurato. Usa RAIPLAYRSS_BATCH_URLS o RAIPLAYRSS_BATCH_URLS_FILE");
    vec![]
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,raiplayfeed=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("RaiPlayFeed - Generatore batch di feed RSS");
    info!("  Upstream: {}", config.upstream.base_url);
    info!("  Timeout: {}s", config.upstream.timeout_secs);
    info!("  Output dir: {}", config.batch.output_dir);
    info!("  Index file: {}", config.batch.index_file);

    let feeds = load_urls(&config);
    if feeds.is_empty() {
        error!(
            "Nessun feed da generare. Aggiungi URL nel file {} o in config.yaml.",
            config.batch.urls_file.as_deref().unwrap_or("urls.txt")
        );
        anyhow::bail!("Nessun URL da processare");
    }
    info!("Trovati {} feed da processare", feeds.len());

    fs::create_dir_all(&config.batch.output_dir)?;

    let client = Client::builder()
        .timeout(config.request_timeout())
        .user_agent(&config.upstream.user_agent)
        .build()?;

    let mut summaries = Vec::new();

    for feed in &feeds {
        info!("Processando: {}/{}", feed.section, feed.program_name);
        match process_feed(&client, &config, feed).await {
            Ok(summary) => {
                info!(
                    "  OK: {} episodi, ultimo: {}",
                    summary.episode_count,
                    summary.latest_episode_date.as_deref().unwrap_or("N/D")
                );
                summaries.push(summary);
            }
            Err(e) => {
                error!("  ERRORE per {}/{}: {}", feed.section, feed.program_name, e);
            }
        }
    }

    if summaries.is_empty() {
        warn!("Nessun feed generato con successo.");
    }
    summaries.sort_by(|a, b| {
        // Ordinamento primario: data ultimo episodio (dal più recente al più vecchio)
        match (
            b.latest_episode_date.as_deref(),
            a.latest_episode_date.as_deref(),
        ) {
            // Se entrambi hanno date, confrontale correttamente
            (Some(b_date), Some(a_date)) => {
                // Parsing della data da "GG/MM/AAAA" a (AAAA, MM, GG) per confronto corretto
                let parse_date = |date_str: &str| {
                    let parts: Vec<&str> = date_str.split('/').collect();
                    if parts.len() == 3 {
                        (
                            parts[2].parse::<i32>().unwrap_or(0), // Anno
                            parts[1].parse::<i32>().unwrap_or(0), // Mese
                            parts[0].parse::<i32>().unwrap_or(0), // Giorno
                        )
                    } else {
                        (0, 0, 0) // Data non valida
                    }
                };
                parse_date(b_date).cmp(&parse_date(a_date))
            }
            // Se solo b ha una data, viene prima
            (Some(_), None) => std::cmp::Ordering::Less,
            // Se solo a ha una data, viene dopo
            (None, Some(_)) => std::cmp::Ordering::Greater,
            // Se entrambi non hanno date, ordina per nome
            (None, None) => a.title.cmp(&b.title),
        }
        // Ordinamento secondario: nome del programma in ordine alfabetico
        .then_with(|| a.title.cmp(&b.title))
    });

    generate_index(&config, &summaries)?;

    info!(
        "Generazione completata. {} feed RSS generati. Index: {}/{}",
        summaries.len(),
        config.batch.output_dir,
        config.batch.index_file
    );

    Ok(())
}

async fn process_feed(
    client: &Client,
    config: &AppConfig,
    feed: &FeedEntry,
) -> Result<FeedSummary, AppError> {
    let rss_xml = generate_rss_feed(client, config, &feed.section, &feed.program_name).await?;

    let filename = format!("{}.xml", feed.program_name);
    let output_path = PathBuf::from(&config.batch.output_dir).join(&filename);
    fs::write(&output_path, &rss_xml)?;
    info!("  Scritto: {}", output_path.display());

    // Build summary for index
    let program_url = format!(
        "{}/{}/{}.json",
        config.upstream.base_url, feed.section, feed.program_name
    );
    let program_page: ProgramPage = fetch_json(client, &program_url, config).await?;

    let mut all_episodes = Vec::new();
    let mut seen_guids = HashSet::new();

    if let Some(block) = &program_page.block {
        for card in &block.cards {
            if let Some(item) = card_to_rss_item(card, config)? {
                if seen_guids.insert(item.guid.clone()) {
                    all_episodes.push(item);
                }
            }
        }
    }

    if let Some(filters) = &program_page.filters {
        for filter in filters {
            if !filter.active {
                let season_url = format!("{}{}", config.upstream.base_url, filter.path_id);
                if let Ok(season_content) =
                    fetch_json::<SeasonContent>(client, &season_url, config).await
                {
                    for card in season_content.cards {
                        if let Some(item) = card_to_rss_item(&card, config)? {
                            if seen_guids.insert(item.guid.clone()) {
                                all_episodes.push(item);
                            }
                        }
                    }
                }
            }
        }
    }

    all_episodes.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));

    let latest_episode_date = all_episodes.first().map(|ep| {
        ep.pub_date
            .with_timezone(&chrono::Local)
            .format("%d/%m/%Y")
            .to_string()
    });

    let image_url = resolve_image_url(&program_page.podcast_info, config)?;

    let base_url = Url::parse(&config.upstream.base_url)?;
    let weblink = base_url
        .join(&program_page.podcast_info.weblink)
        .map(|u| u.to_string())
        .unwrap_or_else(|_| {
            format!(
                "{}{}",
                config.upstream.base_url, program_page.podcast_info.weblink
            )
        });

    Ok(FeedSummary {
        title: program_page.podcast_info.title,
        description: if program_page.podcast_info.description.is_empty() {
            "Nessuna descrizione disponibile".to_string()
        } else {
            program_page.podcast_info.description
        },
        image: image_url,
        weblink,
        rss_url: filename,
        section: feed.section.clone(),
        latest_episode_date,
        episode_count: all_episodes.len(),
    })
}

fn resolve_image_url(podcast_info: &PodcastInfo, config: &AppConfig) -> Result<String, AppError> {
    let base_url = Url::parse(&config.upstream.base_url)?;

    let image_field =
        if podcast_info.image.starts_with("http") || podcast_info.image.starts_with('/') {
            podcast_info.image.clone()
        } else if !podcast_info.image.is_empty() {
            format!("/{}", podcast_info.image)
        } else if let Some(ref landscape) = podcast_info.images.landscape {
            landscape.clone()
        } else if let Some(ref square) = podcast_info.images.square {
            square.clone()
        } else {
            String::new()
        };

    if image_field.is_empty() {
        return Ok(String::new());
    }

    if image_field.starts_with("http") {
        return Ok(image_field);
    }

    Ok(base_url.join(&image_field)?.to_string())
}

fn generate_index(config: &AppConfig, summaries: &[FeedSummary]) -> Result<(), AppError> {
    let mut tera = Tera::default();
    tera.add_raw_template("index.html", TEMPLATE_STR)?;

    let mut ctx = Context::new();
    ctx.insert(
        "generated_at",
        &chrono::Local::now().format("%d/%m/%Y %H:%M").to_string(),
    );
    ctx.insert("feeds", summaries);

    let html = tera.render("index.html", &ctx)?;

    let index_path = PathBuf::from(&config.batch.output_dir).join(&config.batch.index_file);
    fs::write(&index_path, &html)?;
    info!("Index generato: {}", index_path.display());

    Ok(())
}

async fn generate_rss_feed(
    client: &Client,
    config: &AppConfig,
    percorso: &str,
    program_name: &str,
) -> Result<String, AppError> {
    let program_url = format!(
        "{}/{}/{}.json",
        config.upstream.base_url, percorso, program_name
    );
    let program_page: ProgramPage = fetch_json(client, &program_url, config).await?;

    let mut all_episodes = Vec::new();
    let mut seen_guids = HashSet::new();

    if let Some(block) = &program_page.block {
        for card in &block.cards {
            if let Some(item) = card_to_rss_item(card, config)? {
                if seen_guids.insert(item.guid.clone()) {
                    all_episodes.push(item);
                }
            }
        }
    }

    if let Some(filters) = &program_page.filters {
        for filter in filters {
            if !filter.active {
                let season_url = format!("{}{}", config.upstream.base_url, filter.path_id);
                if let Ok(season_content) =
                    fetch_json::<SeasonContent>(client, &season_url, config).await
                {
                    for card in season_content.cards {
                        if let Some(item) = card_to_rss_item(&card, config)? {
                            if seen_guids.insert(item.guid.clone()) {
                                all_episodes.push(item);
                            }
                        }
                    }
                } else {
                    warn!("Failed to fetch season: {}", season_url);
                }
            }
        }
    }

    all_episodes.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));

    let rss = build_rss_feed(&program_page.podcast_info, all_episodes, config)?;
    Ok(rss)
}

fn card_to_rss_item(card: &Card, config: &AppConfig) -> Result<Option<RssItem>, AppError> {
    let base_url_str = &config.upstream.base_url;

    let pub_date = parse_publication_date(&card.literal_publication_date, &card.track_info.date)?;

    let duration_seconds = parse_duration_to_seconds(&card.duration_small_format);
    let enclosure_length = duration_seconds.map(|s| (s as u64) * 128000 / 8);

    // `downloadable_audio` non è sempre presente nelle card (es. "Prima pagina"):
    // in quel caso ripieghiamo su `audio`. Se nessuno dei due è disponibile,
    // saltiamo la card perché senza URL non c'è enclosure.
    let audio_source = card
        .downloadable_audio
        .as_ref()
        .or(Some(&card.audio))
        .filter(|a| !a.url.is_empty());

    let audio_url = match audio_source {
        Some(a) if a.url.starts_with("http") => a.url.clone(),
        Some(a) => format!("{}{}", base_url_str, a.url),
        None => {
            warn!("Card senza audio scaricabile, saltata: {}", card.uniquename);
            return Ok(None);
        }
    };

    let image_url = if card.image.is_empty() {
        String::new()
    } else if card.image.starts_with("http") {
        card.image.clone()
    } else if card.image.starts_with('/') {
        format!("{}{}", base_url_str, card.image)
    } else {
        format!("{}/{}", base_url_str, card.image)
    };

    let episode_link = if card.weblink.starts_with("http") {
        card.weblink.clone()
    } else if card.weblink.starts_with('/') {
        format!("{}{}", base_url_str, card.weblink)
    } else {
        format!("{}/{}", base_url_str, card.weblink)
    };

    let guid = card.uniquename.clone();

    let description = build_description(card, &card.track_info);

    Ok(Some(RssItem {
        guid,
        title: format!("{} - {}", card.toptitle, card.episode_title),
        description,
        link: episode_link,
        pub_date,
        enclosure_url: audio_url,
        enclosure_type: "audio/mpeg".to_string(),
        enclosure_length,
        image_url: Some(image_url),
        duration: Some(card.duration_small_format.clone()),
        episode_number: if card.track_info.episode_number.is_empty() {
            None
        } else {
            Some(card.track_info.episode_number.clone())
        },
        season: if card.track_info.season.is_empty() {
            None
        } else {
            Some(card.track_info.season.clone())
        },
    }))
}

fn parse_publication_date(
    literal_date: &str,
    track_date: &str,
) -> Result<chrono::DateTime<FixedOffset>, AppError> {
    if let Ok(naive) = NaiveDate::parse_from_str(literal_date, "%d %b %Y") {
        let datetime = naive.and_hms_opt(0, 0, 0).unwrap();
        return Ok(FixedOffset::east_opt(0)
            .unwrap()
            .from_local_datetime(&datetime)
            .unwrap());
    }

    if let Ok(naive) = NaiveDate::parse_from_str(track_date, "%Y-%m-%d") {
        let datetime = naive.and_hms_opt(0, 0, 0).unwrap();
        return Ok(FixedOffset::east_opt(0)
            .unwrap()
            .from_local_datetime(&datetime)
            .unwrap());
    }

    if let Ok(naive) =
        NaiveDateTime::parse_from_str(&format!("{} 00:00:00", track_date), "%Y-%m-%d %H:%M:%S")
    {
        return Ok(FixedOffset::east_opt(0)
            .unwrap()
            .from_local_datetime(&naive)
            .unwrap());
    }

    Err(AppError::Internal(anyhow::anyhow!(
        "Could not parse date: {} or {}",
        literal_date,
        track_date
    )))
}

fn parse_duration_to_seconds(duration: &str) -> Option<u32> {
    let parts: Vec<&str> = duration.split(':').collect();
    match parts.len() {
        2 => {
            let minutes = parts[0].parse::<u32>().ok()?;
            let seconds = parts[1].parse::<u32>().ok()?;
            Some(minutes * 60 + seconds)
        }
        3 => {
            let hours = parts[0].parse::<u32>().ok()?;
            let minutes = parts[1].parse::<u32>().ok()?;
            let seconds = parts[2].parse::<u32>().ok()?;
            Some(hours * 3600 + minutes * 60 + seconds)
        }
        _ => None,
    }
}

fn build_description(card: &Card, track_info: &TrackInfo) -> String {
    let mut desc = String::new();

    if !card.description.trim().is_empty() {
        desc.push_str(&card.description);
        desc.push_str("\n\n");
    } else {
        desc.push_str(&card.toptitle);
        desc.push_str("\n\n")
    }

    if !track_info.season.is_empty() {
        desc.push_str(&format!("Stagione: {}\n", track_info.season));
    }
    if !track_info.edition.is_empty() {
        desc.push_str(&format!("Edizione: {}\n", track_info.edition));
    }
    if !track_info.episode_number.is_empty() {
        desc.push_str(&format!("Episodio: {}\n", track_info.episode_number));
    }
    desc.push_str(&format!("Durata: {}\n", card.literal_duration));
    desc.push_str(&format!("Formato: {}\n", card.form));

    desc
}

fn build_rss_feed(
    podcast_info: &PodcastInfo,
    episodes: Vec<RssItem>,
    config: &AppConfig,
) -> Result<String, AppError> {
    let base_url = Url::parse(&config.upstream.base_url)?;
    let program_url = base_url.join(&podcast_info.weblink)?;

    let image_url = resolve_image_url(podcast_info, config)?;

    let owner = ITunesOwnerBuilder::default()
        .name("Rai".to_string())
        .email("podcast@rai.it".to_string())
        .build();

    let mut itunes_channel_ext = ITunesChannelExtensionBuilder::default()
        .author(Some("Rai Radio".to_string()))
        .owner(Some(owner))
        .image(Some(image_url.clone()))
        .explicit(Some("no".to_string()))
        .build();

    for genre in &podcast_info.genres {
        let cat = ITunesCategoryBuilder::default()
            .text(genre.name.clone())
            .build();
        itunes_channel_ext.categories.push(cat);
    }
    for subgenre in &podcast_info.subgenres {
        let cat = ITunesCategoryBuilder::default()
            .text(subgenre.name.clone())
            .build();
        itunes_channel_ext.categories.push(cat);
    }

    let mut channel = ChannelBuilder::default()
        .title(podcast_info.title.clone())
        .language(Some("it".to_string()))
        .copyright(Some(format!("© Rai - {}", podcast_info.title)))
        .ttl(Some("300".to_string()))
        .itunes_ext(Some(itunes_channel_ext))
        .build();

    channel.set_link(program_url.to_string());
    channel.set_description(podcast_info.description.clone());

    if !image_url.is_empty() {
        let img = ImageBuilder::default()
            .url(image_url.clone())
            .title(podcast_info.title.clone())
            .link(program_url.to_string())
            .build();
        channel.set_image(img);
    }

    let categories: Vec<Category> = podcast_info
        .genres
        .iter()
        .map(|g| Category::from(g.name.as_str()))
        .collect();
    channel.set_categories(categories);

    let mut items = Vec::new();
    for episode in episodes {
        let mut itunes_item_ext = ITunesItemExtensionBuilder::default()
            .author(Some("Rai Radio".to_string()))
            .image(episode.image_url.clone())
            .duration(episode.duration.clone())
            .explicit(Some("no".to_string()))
            .build();

        if let Some(season) = &episode.season {
            itunes_item_ext.season = Some(season.clone());
        }
        if let Some(ep_num) = &episode.episode_number {
            itunes_item_ext.episode = Some(ep_num.clone());
        }

        let enclosure = EnclosureBuilder::default()
            .url(episode.enclosure_url.clone())
            .mime_type(episode.enclosure_type.clone())
            .length(episode.enclosure_length.unwrap_or(0).to_string())
            .build();

        let guid = GuidBuilder::default()
            .value(episode.guid.clone())
            .permalink(false)
            .build();

        let item = ItemBuilder::default()
            .title(Some(episode.title))
            .link(Some(episode.link))
            .description(Some(episode.description))
            .pub_date(Some(episode.pub_date.to_rfc2822()))
            .guid(Some(guid))
            .enclosure(Some(enclosure))
            .itunes_ext(Some(itunes_item_ext))
            .build();

        items.push(item);
    }

    channel.set_items(items);

    let mut buf = Vec::new();
    channel.write_to(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}

async fn fetch_json<T: for<'de> serde::Deserialize<'de>>(
    client: &Client,
    url: &str,
    config: &AppConfig,
) -> Result<T, AppError> {
    let response = timeout(config.request_timeout(), client.get(url).send()).await??;

    if !response.status().is_success() {
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(AppError::NotFound);
        }
        return Err(AppError::Request(response.error_for_status().unwrap_err()));
    }

    let text = timeout(config.request_timeout(), response.text()).await??;
    let json = serde_json::from_str::<T>(&text)?;
    Ok(json)
}
