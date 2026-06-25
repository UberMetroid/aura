use crate::i18n::TransKey;

pub fn translate(key: TransKey) -> String {
    let val = match key {
        TransKey::EnterPin => "验证密码",
        TransKey::PinDescription => "此实例已受保护。请输入访问密钥以继续。",
        TransKey::PinInputPlaceholder => "输入访问密钥...",
        TransKey::SignOut => "退出登录",
        TransKey::Ready => "就绪",
        TransKey::SearchPlaceholder => "问任何问题或进行搜索...",
        TransKey::SearchBtn => "搜索",
        TransKey::SearchingBtn => "正在搜索...",
        TransKey::WebSearchOption => "网页搜索与AI",
        TransKey::ImageSearchOption => "图片搜索",
        TransKey::AIAssistantTitle => "AI 助手",
        TransKey::WebResultsTitle => "网页结果",
        TransKey::ImageResultsTitle => "图片结果",
        TransKey::SourceLink => "来源",
        TransKey::ThinkingMsg => "思考中...",
        TransKey::ToggleThemeTooltip => "切换主题",
        TransKey::PrintTooltip => "打印页面",
        TransKey::InvalidPassword => "无效密码。",
        TransKey::ConnectionError => "连接错误。",
        TransKey::AuthenticatedSuccess => "成功验证身份",
        TransKey::LoggedOutSuccess => "成功退出登录",
    };
    val.to_string()
}
