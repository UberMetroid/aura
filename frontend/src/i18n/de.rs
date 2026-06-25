use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    let val = match key {
        TransKey::EnterPin => "Autorisierung erforderlich",
        TransKey::PinDescription => {
            "Diese Instanz ist passwortgeschützt. Geben Sie den Zugriffsschlüssel ein."
        }
        TransKey::PinInputPlaceholder => "Zugriffsschlüssel eingeben...",
        TransKey::SignOut => "Abmelden",
        TransKey::Ready => "Bereit",
        TransKey::SearchPlaceholder => "Fragen Sie etwas oder suchen Sie...",
        TransKey::SearchBtn => "Suchen",
        TransKey::SearchingBtn => "Suche läuft...",
        TransKey::WebSearchOption => "Websuche & KI",
        TransKey::ImageSearchOption => "Bildersuche",
        TransKey::AIAssistantTitle => "KI-Assistent",
        TransKey::WebResultsTitle => "Web-Ergebnisse",
        TransKey::ImageResultsTitle => "Bild-Ergebnisse",
        TransKey::SourceLink => "Quelle",
        TransKey::ThinkingMsg => "Überlegt...",
        TransKey::ToggleThemeTooltip => "Design umschalten",
        TransKey::PrintTooltip => "Drucken",
        TransKey::InvalidPassword => "Ungültiges Passwort.",
        TransKey::ConnectionError => "Verbindungsfehler.",
        TransKey::AuthenticatedSuccess => "Erfolgreich angemeldet",
        TransKey::LoggedOutSuccess => "Erfolgreich abgemeldet",
    };
    val.to_string()
}
