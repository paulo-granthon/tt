use crate::error::{Error, Result};

pub const LANGUAGES: &[(&str, &str)] = &[
    ("auto", "detect"),
    ("en", "English"),
    ("pt-BR", "Portuguese (Brazil)"),
    ("pt-PT", "Portuguese (Portugal)"),
    ("es", "Spanish"),
    ("fr", "French"),
    ("de", "German"),
    ("it", "Italian"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("zh-CN", "Chinese (Simplified)"),
    ("zh-TW", "Chinese (Traditional)"),
    ("ru", "Russian"),
    ("ar", "Arabic"),
    ("nl", "Dutch"),
    ("sv", "Swedish"),
    ("pl", "Polish"),
    ("tr", "Turkish"),
    ("hi", "Hindi"),
    ("el", "Greek"),
    ("he", "Hebrew"),
    ("cs", "Czech"),
    ("da", "Danish"),
    ("fi", "Finnish"),
    ("no", "Norwegian"),
    ("uk", "Ukrainian"),
    ("vi", "Vietnamese"),
    ("th", "Thai"),
    ("id", "Indonesian"),
    ("ro", "Romanian"),
    ("hu", "Hungarian"),
];

pub fn is_identity(sl: &str, tl: &str) -> bool {
    sl != "auto" && sl == tl
}

pub fn name(code: &str) -> Option<&'static str> {
    LANGUAGES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, n)| *n)
}

pub fn normalize_key(input: &str) -> String {
    input
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

pub fn canonical(input: &str) -> Option<&'static str> {
    match normalize_key(input).as_str() {
        "auto" | "detect" => Some("auto"),
        "en" | "eng" | "english" => Some("en"),
        "ptbr" | "br" | "pt" | "ptbra" | "portugues" | "portuguese" | "brazilian" => Some("pt-BR"),
        "ptpt" | "portugal" => Some("pt-PT"),
        "es" | "spa" | "spanish" | "espanol" | "castellano" => Some("es"),
        "fr" | "fra" | "fre" | "french" | "francais" => Some("fr"),
        "de" | "deu" | "ger" | "german" | "deutsch" => Some("de"),
        "it" | "ita" | "italian" | "italiano" => Some("it"),
        "ja" | "jp" | "jpn" | "japanese" => Some("ja"),
        "ko" | "kor" | "korean" => Some("ko"),
        "zh" | "zhcn" | "chinese" | "mandarin" | "simplifiedchinese" => Some("zh-CN"),
        "zhtw" | "traditionalchinese" => Some("zh-TW"),
        "ru" | "rus" | "russian" => Some("ru"),
        "ar" | "ara" | "arabic" => Some("ar"),
        "nl" | "nld" | "dutch" => Some("nl"),
        "sv" | "swe" | "swedish" => Some("sv"),
        "pl" | "pol" | "polish" => Some("pl"),
        "tr" | "tur" | "turkish" => Some("tr"),
        "hi" | "hin" | "hindi" => Some("hi"),
        "el" | "gre" | "greek" => Some("el"),
        "he" | "heb" | "hebrew" | "iw" => Some("he"),
        "cs" | "cze" | "czech" => Some("cs"),
        "da" | "dan" | "danish" => Some("da"),
        "fi" | "fin" | "finnish" => Some("fi"),
        "no" | "nor" | "norwegian" => Some("no"),
        "uk" | "ukr" | "ukrainian" => Some("uk"),
        "vi" | "vie" | "vietnamese" => Some("vi"),
        "th" | "tha" | "thai" => Some("th"),
        "id" | "ind" | "indonesian" => Some("id"),
        "ro" | "ron" | "rum" | "romanian" => Some("ro"),
        "hu" | "hun" | "hungarian" => Some("hu"),
        _ => None,
    }
}

pub fn resolve_source(input: &str) -> Result<String> {
    canonical(input)
        .map(str::to_string)
        .ok_or_else(|| Error::UnknownLanguage(input.to_string()))
}

pub fn resolve_target(input: &str) -> Result<String> {
    match canonical(input) {
        Some("auto") => Err(Error::BadArgs(
            "target language cannot be 'auto'".to_string(),
        )),
        Some(c) => Ok(c.to_string()),
        None => Err(Error::UnknownLanguage(input.to_string())),
    }
}
