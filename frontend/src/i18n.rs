mod de;
mod en;
mod es;
mod fr;
mod ja;
mod pt;
mod ru;
mod zh;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Locale {
    En,
    Zh,
    Es,
    De,
    Ja,
    Fr,
    Pt,
    Ru,
}

impl Locale {
    pub fn from_str(s: &str) -> Self {
        match s {
            "zh" => Self::Zh,
            "es" => Self::Es,
            "de" => Self::De,
            "ja" => Self::Ja,
            "fr" => Self::Fr,
            "pt" => Self::Pt,
            "ru" => Self::Ru,
            _ => Self::En,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::Zh => "zh",
            Self::Es => "es",
            Self::De => "de",
            Self::Ja => "ja",
            Self::Fr => "fr",
            Self::Pt => "pt",
            Self::Ru => "ru",
            Self::En => "en",
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::Zh => "简体中文",
            Self::Es => "Español",
            Self::De => "Deutsch",
            Self::Ja => "日本語",
            Self::Fr => "Français",
            Self::Pt => "Português",
            Self::Ru => "Русский",
            Self::En => "English",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::En,
            Self::Zh,
            Self::Es,
            Self::De,
            Self::Ja,
            Self::Fr,
            Self::Pt,
            Self::Ru,
        ]
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TransKey {
    EnterPin,
    PinDescription,
    PinInputPlaceholder,
    SignOut,
    Ready,
    SearchPlaceholder,
    SearchBtn,
    SearchingBtn,
    WebSearchOption,
    ImageSearchOption,
    AIAssistantTitle,
    WebResultsTitle,
    ImageResultsTitle,
    SourceLink,
    ThinkingMsg,
    ToggleThemeTooltip,
    PrintTooltip,
    InvalidPassword,
    ConnectionError,
    AuthenticatedSuccess,
    LoggedOutSuccess,
}

pub fn translate(key: TransKey, locale: Locale) -> String {
    match locale {
        Locale::Zh => zh::translate(key),
        Locale::Es => es::translate(key),
        Locale::De => de::translate(key),
        Locale::Ja => ja::translate(key),
        Locale::Fr => fr::translate(key),
        Locale::Pt => pt::translate(key),
        Locale::Ru => ru::translate(key),
        Locale::En => en::translate(key),
    }
}
