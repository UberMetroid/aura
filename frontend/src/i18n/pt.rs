use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    let val = match key {
        TransKey::EnterPin => "Autenticação Necessária",
        TransKey::PinDescription => {
            "Esta instância está protegida por senha. Insira a chave de acesso para continuar."
        }
        TransKey::PinInputPlaceholder => "Insira a chave de acesso...",
        TransKey::SignOut => "Sair",
        TransKey::Ready => "Pronto",
        TransKey::SearchPlaceholder => "Pergunte qualquer coisa ou pesquise...",
        TransKey::SearchBtn => "Pesquisar",
        TransKey::SearchingBtn => "Pesquisando...",
        TransKey::WebSearchOption => "Pesquisa Web e IA",
        TransKey::ImageSearchOption => "Pesquisa de Imagens",
        TransKey::AIAssistantTitle => "Assistente de IA",
        TransKey::WebResultsTitle => "Resultados da Web",
        TransKey::ImageResultsTitle => "Resultados de Imagens",
        TransKey::SourceLink => "Fonte",
        TransKey::ThinkingMsg => "Pensando...",
        TransKey::ToggleThemeTooltip => "Alternar tema",
        TransKey::PrintTooltip => "Imprimir",
        TransKey::InvalidPassword => "Senha inválida.",
        TransKey::ConnectionError => "Erro de conexão.",
        TransKey::AuthenticatedSuccess => "Autenticado com sucesso",
        TransKey::LoggedOutSuccess => "Sessão encerrada com sucesso",
    };
    val.to_string()
}
