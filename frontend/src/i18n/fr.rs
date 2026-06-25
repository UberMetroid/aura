use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    let val = match key {
        TransKey::EnterPin => "Authentification requise",
        TransKey::PinDescription => {
            "Cette instance est protégée par mot de passe. Entrez la clé d'accès pour continuer."
        }
        TransKey::PinInputPlaceholder => "Entrez la clé d'accès...",
        TransKey::SignOut => "Se déconnecter",
        TransKey::Ready => "Prêt",
        TransKey::SearchPlaceholder => "Posez une question ou recherchez...",
        TransKey::SearchBtn => "Rechercher",
        TransKey::SearchingBtn => "Recherche...",
        TransKey::WebSearchOption => "Recherche Web & IA",
        TransKey::ImageSearchOption => "Recherche d'Images",
        TransKey::AIAssistantTitle => "Assistant IA",
        TransKey::WebResultsTitle => "Résultats Web",
        TransKey::ImageResultsTitle => "Résultats d'Images",
        TransKey::SourceLink => "Source",
        TransKey::ThinkingMsg => "Réflexion...",
        TransKey::ToggleThemeTooltip => "Changer de thème",
        TransKey::PrintTooltip => "Imprimer",
        TransKey::InvalidPassword => "Mot de passe invalide.",
        TransKey::ConnectionError => "Erreur de connexion.",
        TransKey::AuthenticatedSuccess => "Authentifié avec succès",
        TransKey::LoggedOutSuccess => "Déconnecté avec succès",
    };
    val.to_string()
}
