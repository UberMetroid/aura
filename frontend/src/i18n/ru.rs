use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    let val = match key {
        TransKey::EnterPin => "Требуется авторизация",
        TransKey::PinDescription => {
            "Этот инстанс защищен паролем. Введите ключ доступа для продолжения."
        }
        TransKey::PinInputPlaceholder => "Введите ключ доступа...",
        TransKey::SignOut => "Выйти",
        TransKey::Ready => "Готово",
        TransKey::SearchPlaceholder => "Спросите о чем угодно или введите запрос...",
        TransKey::SearchBtn => "Поиск",
        TransKey::SearchingBtn => "Поиск...",
        TransKey::WebSearchOption => "Веб-поиск и ИИ",
        TransKey::ImageSearchOption => "Поиск изображений",
        TransKey::AIAssistantTitle => "ИИ-ассистент",
        TransKey::WebResultsTitle => "Результаты поиска",
        TransKey::ImageResultsTitle => "Результаты картинок",
        TransKey::SourceLink => "Источник",
        TransKey::ThinkingMsg => "Думает...",
        TransKey::ToggleThemeTooltip => "Переключить тему",
        TransKey::PrintTooltip => "Печать",
        TransKey::InvalidPassword => "Неверный пароль.",
        TransKey::ConnectionError => "Ошибка подключения.",
        TransKey::AuthenticatedSuccess => "Успешная авторизация",
        TransKey::LoggedOutSuccess => "Успешный выход",
    };
    val.to_string()
}
