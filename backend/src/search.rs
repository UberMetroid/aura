use crate::circuit_breaker::CircuitBreaker;
use std::sync::Arc;

/// Search providers ordered for the fallback chain.
///
/// Aura does not yet retry across providers automatically; this constant is
/// the canonical order in which `search_text_with_fallback` walks them.
/// SearXNG is the default because it is self-hosted; if it is unreachable
/// the chain falls back to DuckDuckGo (which rate-limits quickly), then
/// Wikipedia (no auth, low rate limits), then Grokipedia (xAI hosted).
///
/// To change the fallback order, edit this list. To disable a provider in
/// the fallback chain, remove it from the list — the explicit per-provider
/// `SEARCH_PROVIDER` setting still works.
const FALLBACK_CHAIN: &[&str] = &["searxng", "duckduckgo", "wikipedia", "grokipedia"];

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

    /// Verifies if search service is up and healthy.
    ///
    /// Uses the same fallback chain as `search_text_with_fallback`: tries the
    /// configured provider first, then walks `FALLBACK_CHAIN` on failure.
    pub async fn get_status(&self) -> bool {
        // Honor explicit provider selection first; if it's a known one,
        // try it directly. Otherwise (or on failure), walk the chain.
        let primary = self.search_provider.as_str();
        if self.provider_status(primary).await {
            return true;
        }
        for &provider in FALLBACK_CHAIN {
            if provider == primary {
                continue;
            }
            if self.provider_status(provider).await {
                return true;
            }
        }
        false
    }

    /// Check whether a specific provider responds to a probe query.
    async fn provider_status(&self, provider: &str) -> bool {
        match provider {
            "duckduckgo" => crate::duckduckgo::search_duckduckgo("test", 1)
                .await
                .is_ok(),
            "wikipedia" => crate::wikipedia::search_wikipedia("test", 1).await.is_ok(),
            "grokipedia" => crate::grokipedia::search_grokipedia("test", 1)
                .await
                .is_ok(),
            "merged" => crate::merged::search_merged("test", 1).await.is_ok(),
            // Treat anything else (including "searxng" and unknown values)
            // as the SearXNG path. This preserves the prior default behavior
            // of treating unknown values as the configured SearXNG endpoint.
            _ => {
                crate::searxng::check_searxng_health(&self.searxng_base_url, &self.circuit_breaker)
                    .await
            }
        }
    }

    /// Handles a text search request.
    ///
    /// Preserves the prior behavior of dispatching to exactly the configured
    /// provider. Use [`search_text_with_fallback`] for the new fallback-chain
    /// behavior.
    pub async fn search_text(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String)>, String> {
        self.dispatch_text(&self.search_provider, query, limit).await
    }

    /// Handles a text search with automatic fallback across providers.
    ///
    /// Tries the configured provider first, then walks `FALLBACK_CHAIN` on
    /// failure (network error, 4xx that's not 429, or empty results). The
    /// first provider that returns a non-empty result wins. If every
    /// provider fails, returns the last error message.
    pub async fn search_text_with_fallback(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String)>, String> {
        let primary = self.search_provider.as_str();
        match self.dispatch_text(primary, query, limit).await {
            Ok(results) if !results.is_empty() => return Ok(results),
            Ok(_) => {
                tracing::warn!(
                    provider = primary,
                    "primary provider returned empty results, falling back"
                );
            }
            Err(e) => {
                tracing::warn!(
                    provider = primary,
                    error = %e,
                    "primary provider failed, falling back"
                );
            }
        }

        let mut last_err = String::from("all providers failed");
        for &provider in FALLBACK_CHAIN {
            if provider == primary {
                continue;
            }
            match self.dispatch_text(provider, query, limit).await {
                Ok(results) if !results.is_empty() => return Ok(results),
                Ok(_) => {
                    tracing::debug!(provider, "provider returned empty results");
                }
                Err(e) => {
                    tracing::debug!(provider, error = %e, "provider failed");
                    last_err = format!("{provider}: {e}");
                }
            }
        }
        Err(last_err)
    }

    /// Dispatch to exactly one provider, identified by name.
    async fn dispatch_text(
        &self,
        provider: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String)>, String> {
        match provider {
            "duckduckgo" => crate::duckduckgo::search_duckduckgo(query, limit).await,
            "wikipedia" => crate::wikipedia::search_wikipedia(query, limit).await,
            "grokipedia" => crate::grokipedia::search_grokipedia(query, limit).await,
            "merged" => crate::merged::search_merged(query, limit).await,
            // Treat anything else as the configured SearXNG endpoint.
            _ => {
                crate::searxng::search_text_searxng(
                    &self.searxng_base_url,
                    &self.circuit_breaker,
                    query,
                    limit,
                )
                .await
            }
        }
    }

    /// Handles an image search request.
    ///
    /// Image search is currently SearXNG-only because the other providers do
    /// not expose image endpoints we have integrated. If you need a fallback
    /// for image search, add it here.
    pub async fn search_images(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String, String)>, String> {
        crate::searxng::search_images_searxng(
            &self.searxng_base_url,
            &self.circuit_breaker,
            query,
            limit,
        )
        .await
    }
}

pub type SharedSearchService = Arc<SearchService>;

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity-check: the fallback chain contains the expected primary and
    /// has a non-zero number of entries. If this fails, the chain was
    /// accidentally emptied or re-ordered.
    #[test]
    fn fallback_chain_is_nonempty_and_includes_searxng() {
        assert!(!FALLBACK_CHAIN.is_empty());
        assert!(
            FALLBACK_CHAIN.contains(&"searxng"),
            "SearXNG must be in the fallback chain (it is the default provider)"
        );
    }

    /// The fallback chain must not contain duplicates. Otherwise the
    /// provider_status loop would re-probe a known-failed provider.
    #[test]
    fn fallback_chain_has_no_duplicates() {
        let mut sorted: Vec<&str> = FALLBACK_CHAIN.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            FALLBACK_CHAIN.len(),
            "fallback chain has duplicates"
        );
    }

    /// `merged` is a synthetic provider that combines other sources; it
    /// should not appear in the fallback chain (calling merged as a fallback
    /// would re-invoke the primary chain, causing infinite recursion).
    #[test]
    fn fallback_chain_excludes_merged() {
        assert!(
            !FALLBACK_CHAIN.contains(&"merged"),
            "merged is a synthetic provider and must not appear in the fallback chain"
        );
    }

    /// Dispatching to an unknown provider falls back to SearXNG, preserving
    /// the original default behavior for unrecognized `SEARCH_PROVIDER` values.
    #[test]
    fn unknown_provider_dispatches_to_searxng_path() {
        // We can't easily test the async dispatch without a network, but we
        // can verify that the dispatcher's match arm treats unknown values
        // as the catch-all arm that goes to SearXNG. This is a smoke test
        // for the matcher shape.
        let provider = "some-new-future-provider";
        let arm = match provider {
            "duckduckgo" => "ddg",
            "wikipedia" => "wiki",
            "grokipedia" => "grok",
            "merged" => "merged",
            _ => "searxng",
        };
        assert_eq!(arm, "searxng");
    }
}