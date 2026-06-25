use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub pin: Option<String>,
    pub max_attempts: usize,
    pub enable_translation: bool,
    pub enable_themes: bool,
    pub enable_print: bool,
    pub searxng_base_url: String,
    pub search_provider: String,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub static_dir: String,
}

impl Config {
    pub fn load() -> Self {
        let _ = dotenvy::from_path("/app/data/.env");
        let _ = dotenvy::dotenv();

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(4408);

        let pin = env::var("RUSTSEARCH_PIN")
            .or_else(|_| env::var("PIN"))
            .ok()
            .filter(|p| !p.trim().is_empty());

        let max_attempts = env::var("MAX_ATTEMPTS")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(5);

        let enable_translation = env::var("ENABLE_TRANSLATION")
            .map(|v| v == "true" || v == "on")
            .unwrap_or(true);

        let enable_themes = env::var("ENABLE_THEMES")
            .map(|v| v != "false" && v != "off")
            .unwrap_or(true);

        let enable_print = env::var("ENABLE_PRINT")
            .map(|v| v != "false" && v != "off")
            .unwrap_or(true);

        let searxng_base_url =
            env::var("SEARXNG_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8888".to_string());

        let search_provider =
            env::var("SEARCH_PROVIDER").unwrap_or_else(|_| "duckduckgo".to_string());

        // Ollama config - default to standard host port
        let ollama_base_url =
            env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());

        let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3".to_string());

        let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| "./frontend/dist".to_string());

        Self {
            host,
            port,
            pin,
            max_attempts,
            enable_translation,
            enable_themes,
            enable_print,
            searxng_base_url,
            search_provider,
            ollama_base_url,
            ollama_model,
            static_dir,
        }
    }
}
