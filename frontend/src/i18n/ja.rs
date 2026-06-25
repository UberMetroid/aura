use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    let val = match key {
        TransKey::EnterPin => "認証が必要です",
        TransKey::PinDescription => {
            "このインスタンスはパスワードで保護されています。アクセスキーを入力してください。"
        }
        TransKey::PinInputPlaceholder => "アクセスキーを入力...",
        TransKey::SignOut => "ログアウト",
        TransKey::Ready => "準備完了",
        TransKey::SearchPlaceholder => "何でも質問するか、検索してください...",
        TransKey::SearchBtn => "検索",
        TransKey::SearchingBtn => "検索中...",
        TransKey::WebSearchOption => "ウェブ検索とAI",
        TransKey::ImageSearchOption => "画像検索",
        TransKey::AIAssistantTitle => "AI アシスタント",
        TransKey::WebResultsTitle => "ウェブ検索結果",
        TransKey::ImageResultsTitle => "画像検索結果",
        TransKey::SourceLink => "ソース",
        TransKey::ThinkingMsg => "思考中...",
        TransKey::ToggleThemeTooltip => "テーマ切り替え",
        TransKey::PrintTooltip => "印刷",
        TransKey::InvalidPassword => "パスワードが無効です。",
        TransKey::ConnectionError => "接続エラー。",
        TransKey::AuthenticatedSuccess => "認証に成功しました",
        TransKey::LoggedOutSuccess => "ログアウトしました",
    };
    val.to_string()
}
