use serde::Deserialize;

/// Deserializza un campo stringa accettando `null` o campo assente come stringa vuota.
/// RaiPlaySound restituisce a volte `null` (es. `image`, `images.square`) o omette campi
/// (es. `description` nelle playlist), e serde altrimenti fallirebbe la deserializzazione.
fn string_or_empty<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Debug, Deserialize)]
pub struct ProgramPage {
    #[serde(rename = "podcast_info")]
    pub podcast_info: PodcastInfo,
    pub block: Option<Block>,
    pub filters: Option<Vec<Filter>>,
}

#[derive(Debug, Deserialize)]
pub struct PodcastInfo {
    pub uniquename: String,
    pub title: String,
    #[serde(default)]
    pub vanity: String,
    pub description: String,
    pub images: Images,
    pub image: String,
    pub weblink: String,
    pub path_id: String,
    #[serde(default)]
    pub genres: Vec<Genre>,
    #[serde(default)]
    pub subgenres: Vec<Genre>,
}

#[derive(Debug, Deserialize)]
pub struct Images {
    pub landscape: Option<String>,
    pub square: Option<String>,
    #[serde(rename = "landscape_logo")]
    pub landscape_logo: Option<String>,
    #[serde(rename = "square_external")]
    pub square_external: Option<String>,
    #[serde(rename = "landscape_43_logo")]
    pub landscape_43_logo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Genre {
    pub id: String,
    pub name: String,
    pub pipe: String,
    #[serde(default)]
    pub principal: bool,
}

#[derive(Debug, Deserialize)]
pub struct Block {
    pub cards: Vec<Card>,
}

#[derive(Debug, Deserialize)]
pub struct Filter {
    pub active: bool,
    pub path: String,
    pub label: String,
    pub weblink: String,
    pub path_id: String,
    #[serde(rename = "content_size")]
    pub content_size: ContentSize,
}

#[derive(Debug, Deserialize)]
pub struct ContentSize {
    pub number: usize,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct Card {
    pub uniquename: String,
    pub title: String,
    pub toptitle: String,
    pub subtitle: String,
    #[serde(default, deserialize_with = "string_or_empty")]
    pub description: String,
    #[serde(rename = "episode_title")]
    pub episode_title: String,
    pub form: String,
    pub audio: Audio,
    #[serde(rename = "downloadable_audio")]
    #[serde(default)]
    pub downloadable_audio: Option<Audio>,
    pub images: CardImages,
    #[serde(default, deserialize_with = "string_or_empty")]
    pub image: String,
    pub weblink: String,
    pub path_id: String,
    #[serde(rename = "literal_publication_date")]
    pub literal_publication_date: String,
    #[serde(rename = "literal_duration")]
    pub literal_duration: String,
    #[serde(rename = "duration_small_format")]
    pub duration_small_format: String,
    #[serde(rename = "track_info")]
    pub track_info: TrackInfo,
    #[serde(rename = "login_required")]
    pub login_required: bool,
}

#[derive(Debug, Deserialize)]
pub struct Audio {
    pub title: String,
    #[serde(default)]
    pub poster: String,
    pub url: String,
    #[serde(rename = "type")]
    pub audio_type: String,
    #[serde(default)]
    pub duration: String,
}

#[derive(Debug, Deserialize)]
pub struct CardImages {
    #[serde(default, deserialize_with = "string_or_empty")]
    pub square: String,
    pub cover: Option<String>,
    pub landscape: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TrackInfo {
    pub date: String,
    #[serde(rename = "episode_title")]
    pub episode_title: String,
    #[serde(rename = "episode_number")]
    pub episode_number: String,
    pub season: String,
    pub edition: String,
}

#[derive(Debug, Deserialize)]
pub struct SeasonContent {
    pub cards: Vec<Card>,
}

#[derive(Debug, Clone)]
pub struct RssItem {
    pub guid: String,
    pub title: String,
    pub description: String,
    pub link: String,
    pub pub_date: chrono::DateTime<chrono::FixedOffset>,
    pub enclosure_url: String,
    pub enclosure_type: String,
    pub enclosure_length: Option<u64>,
    pub image_url: Option<String>,
    pub duration: Option<String>,
    pub episode_number: Option<String>,
    pub season: Option<String>,
}