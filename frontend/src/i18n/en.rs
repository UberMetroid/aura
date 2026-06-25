use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    let val = match key {
        TransKey::EnterPin => "Authorization Required",
        TransKey::PinDescription => {
            "This instance is password protected. Enter the access key to proceed."
        }
        TransKey::PinInputPlaceholder => "Enter Access Key...",
        TransKey::SignOut => "Sign Out",
        TransKey::Ready => "Ready",
        TransKey::SearchPlaceholder => "Ask anything or search...",
        TransKey::SearchBtn => "Search",
        TransKey::SearchingBtn => "Searching...",
        TransKey::WebSearchOption => "Web Search & AI",
        TransKey::ImageSearchOption => "Image Search",
        TransKey::AIAssistantTitle => "AI Assistant",
        TransKey::WebResultsTitle => "Web Results",
        TransKey::ImageResultsTitle => "Image Results",
        TransKey::SourceLink => "Source",
        TransKey::ThinkingMsg => "Thinking...",
        TransKey::ToggleThemeTooltip => "Toggle theme",
        TransKey::PrintTooltip => "Print",
        TransKey::InvalidPassword => "Invalid password.",
        TransKey::ConnectionError => "Connection error.",
        TransKey::AuthenticatedSuccess => "Authenticated successfully",
        TransKey::LoggedOutSuccess => "Logged out successfully",
    };
    val.to_string()
}
