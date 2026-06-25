use std::sync::Arc;
use crate::circuit_breaker::CircuitBreaker;

pub struct SearchService {
    search_provider: String,
    searxng_base_url: String,
    circuit_breaker: CircuitBreaker,
}

impl SearchService {
    pub fn new(search_provider: String, searxng_base_url: String) -> Arc<Self> {
        Arc::new(Self {
            search_provider,
            searxng_base_url,
            circuit_breaker: CircuitBreaker::new(),
        })
    }

    /// Verifies if search service is up and healthy
    pub async fn get_status(&self) -> bool {
        match self.search_provider.as_str() {
            "duckduckgo" => crate::duckduckgo::search_duckduckgo("test", 1).await.is_ok(),
            "wikipedia" => crate::wikipedia::search_wikipedia("test", 1).await.is_ok(),
            "grokipedia" => crate::grokipedia::search_grokipedia("test", 1).await.is_ok(),
            "merged" => crate::merged::search_merged("test", 1).await.is_ok(),
            _ => crate::searxng::check_searxng_health(&self.searxng_base_url, &self.circuit_breaker).await,
        }
    }

    /// Handles a text search request
    pub async fn search_text(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String)>, String> {
        match self.search_provider.as_str() {
            "duckduckgo" => crate::duckduckgo::search_duckduckgo(query, limit).await,
            "wikipedia" => crate::wikipedia::search_wikipedia(query, limit).await,
            "grokipedia" => crate::grokipedia::search_grokipedia(query, limit).await,
            "merged" => crate::merged::search_merged(query, limit).await,
            _ => crate::searxng::search_text_searxng(&self.searxng_base_url, &self.circuit_breaker, query, limit).await,
        }
    }

    /// Handles an image search request
    pub async fn search_images(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String, String)>, String> {
        if self.search_provider != "searxng" {
            return Ok(Vec::new());
        }
        crate::searxng::search_images_searxng(&self.searxng_base_url, &self.circuit_breaker, query, limit).await
    }
}

pub type SharedSearchService = Arc<SearchService>;

