use crate::duckduckgo::search_duckduckgo;
use crate::grokipedia::search_grokipedia;
use crate::wikipedia::search_wikipedia;

/// Queries DuckDuckGo, Grokipedia, and Wikipedia in parallel and merges their results
pub async fn search_merged(
    query: &str,
    limit: usize,
) -> Result<Vec<(String, String, String)>, String> {
    let ddg_fut = search_duckduckgo(query, limit);
    let grok_fut = search_grokipedia(query, limit);
    let wiki_fut = search_wikipedia(query, limit);

    let (ddg_res, grok_res, wiki_res) = tokio::join!(ddg_fut, grok_fut, wiki_fut);

    let ddg_list = ddg_res.unwrap_or_default();
    let grok_list = grok_res.unwrap_or_default();
    let wiki_list = wiki_res.unwrap_or_default();

    let mut merged = Vec::new();
    let mut urls = std::collections::HashSet::new();

    let mut grok_iter = grok_list.into_iter();
    let mut wiki_iter = wiki_list.into_iter();
    let mut ddg_iter = ddg_list.into_iter();

    loop {
        let mut added = false;

        if let Some(item) = grok_iter.next()
            && urls.insert(item.2.clone())
        {
            merged.push(item);
            added = true;
        }

        if let Some(item) = wiki_iter.next()
            && urls.insert(item.2.clone())
        {
            merged.push(item);
            added = true;
        }

        for _ in 0..2 {
            if let Some(item) = ddg_iter.next()
                && urls.insert(item.2.clone())
            {
                merged.push(item);
                added = true;
            }
        }

        if !added || merged.len() >= limit {
            break;
        }
    }

    merged.truncate(limit);
    Ok(merged)
}
