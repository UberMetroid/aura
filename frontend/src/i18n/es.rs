use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    let val = match key {
        TransKey::EnterPin => "Autorización Requerida",
        TransKey::PinDescription => {
            "Esta instancia está protegida. Ingrese la clave de acceso para continuar."
        }
        TransKey::PinInputPlaceholder => "Ingrese la clave de acceso...",
        TransKey::SignOut => "Cerrar sesión",
        TransKey::Ready => "Listo",
        TransKey::SearchPlaceholder => "Pregunta cualquier cosa o busca...",
        TransKey::SearchBtn => "Buscar",
        TransKey::SearchingBtn => "Buscando...",
        TransKey::WebSearchOption => "Búsqueda Web y IA",
        TransKey::ImageSearchOption => "Búsqueda de Imágenes",
        TransKey::AIAssistantTitle => "Asistente de IA",
        TransKey::WebResultsTitle => "Resultados de la Web",
        TransKey::ImageResultsTitle => "Resultados de Imágenes",
        TransKey::SourceLink => "Fuente",
        TransKey::ThinkingMsg => "Pensando...",
        TransKey::ToggleThemeTooltip => "Cambiar tema",
        TransKey::PrintTooltip => "Imprimir",
        TransKey::InvalidPassword => "Contraseña incorrecta.",
        TransKey::ConnectionError => "Error de conexión.",
        TransKey::AuthenticatedSuccess => "Autenticado con éxito",
        TransKey::LoggedOutSuccess => "Sesión cerrada con éxito",
    };
    val.to_string()
}
