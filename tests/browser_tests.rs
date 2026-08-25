use tt::browser::{encode, translate_url};

#[test]
fn encode_leaves_unreserved_untouched() {
    assert_eq!(encode("Hello-world_1.0~"), "Hello-world_1.0~");
}

#[test]
fn encode_percent_encodes_spaces_and_symbols() {
    assert_eq!(encode("bom dia"), "bom%20dia");
    assert_eq!(encode("a&b=c"), "a%26b%3Dc");
}

#[test]
fn encode_handles_multibyte_utf8() {
    assert_eq!(encode("olá"), "ol%C3%A1");
}

#[test]
fn translate_url_has_all_params() {
    let url = translate_url("pt-BR", "en", "bom dia");
    assert_eq!(
        url,
        "https://translate.google.com/?sl=pt-BR&tl=en&text=bom%20dia&op=translate"
    );
}

#[test]
fn translate_url_encodes_auto_and_target() {
    let url = translate_url("auto", "es", "olá");
    assert!(url.contains("sl=auto"));
    assert!(url.contains("tl=es"));
    assert!(url.contains("text=ol%C3%A1"));
}
