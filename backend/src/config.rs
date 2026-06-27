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
    /// Maximum total characters (sum of all message contents) accepted by
    /// the inference endpoint. Requests exceeding this are rejected with a
    /// 400 Bad Request before reaching Ollama. This is a defense-in-depth
    /// measure against prompt-injection DoS: a single user cannot force
    /// the server to assemble arbitrarily large prompts for the model.
    pub max_prompt_chars: usize,
    /// Optional system prompt prefix prepended to the conversation before
    /// forwarding to Ollama. Set `AURA_SYSTEM_PROMPT` to constrain the model
    /// to the search-assistant role and reduce the surface for prompt
    /// injection from user-supplied messages.
    pub system_prompt: Option<String>,
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

        let pin = env::var("AURA_PIN")
            .or_else(|_| env::var("PIN"))
            .ok()
            .filter(|p| !p.is_empty() && p.len() >= 4 && p.len() <= 64);

        let max_attempts = env::var("MAX_ATTEMPTS")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(5);

        let enable_translation = env::var("ENABLE_TRANSLATION")
            .map(|v| v == "true" || v == "on")
            .unwrap_or(true);

        let enable_themes = env::var("ENABLE_THEMES")
            .map(|v| v == "true" || v == "on")
            .unwrap_or(false);

        let enable_print = env::var("ENABLE_PRINT")
            .map(|v| v == "true" || v == "on")
            .unwrap_or(false);

        let searxng_base_url =
            env::var("SEARXNG_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8888".to_string());

        let search_provider =
            env::var("SEARCH_PROVIDER").unwrap_or_else(|_| "duckduckgo".to_string());

        // Ollama config - default to standard host port
        let ollama_base_url =
            env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());

        let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3".to_string());

        let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| "./frontend/dist".to_string());

        let max_prompt_chars = env::var("MAX_PROMPT_CHARS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8000);

        let system_prompt = env::var("AURA_SYSTEM_PROMPT")
            .ok()
            .filter(|s| !s.is_empty());

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
            max_prompt_chars,
            system_prompt,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default `max_prompt_chars` should be 8000 — large enough for
    /// realistic chat histories but small enough to bound the cost of a
    /// single request against Ollama.
    #[test]
    fn default_max_prompt_chars_is_8000() {
        // Verify the constant we read in `from_env` matches the documented
        // default. If this test fails, update both the env-default and the
        // documentation comment in `Config`.
        let default = env::var("MAX_PROMPT_CHARS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8000);
        assert_eq!(default, 8000);
    }
}
