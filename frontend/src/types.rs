use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct TextSearchResult {
    pub title: String,
    pub snippet: String,
    pub url: String,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ChatCompletionRequest {
    pub messages: Vec<ChatMessage>,
}

#[derive(Deserialize, Debug)]
pub struct Delta {
    pub content: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub delta: Delta,
}

#[derive(Deserialize, Debug)]
pub struct SseChunk {
    pub choices: Vec<Choice>,
}
