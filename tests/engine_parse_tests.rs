use tt::engine::google::{parse_response, parse_t_response};

const HELLO: &str = include_str!("fixtures/hello_en_pt.json");
const HOUSE: &str = include_str!("fixtures/house_no_dict.json");
const MULTI: &str = include_str!("fixtures/multi_sentence.json");

#[test]
fn parses_primary_translation() {
    let t = parse_response(HELLO).unwrap();
    assert_eq!(t.primary, "olá");
}

#[test]
fn parses_detected_source() {
    let t = parse_response(HELLO).unwrap();
    assert_eq!(t.detected_source.as_deref(), Some("en"));
}

#[test]
fn parses_synonyms_with_back_translations() {
    let t = parse_response(HELLO).unwrap();
    assert_eq!(t.synonyms.len(), 3);
    assert_eq!(t.synonyms[0].word, "olá");
    assert_eq!(t.synonyms[0].back, vec!["hello", "hi"]);
    assert_eq!(t.synonyms[1].word, "oi");
    assert_eq!(t.synonyms[1].back, vec!["hi", "hello"]);
    assert_eq!(t.synonyms[2].word, "alô");
    assert_eq!(t.synonyms[2].back, vec!["hello"]);
}

#[test]
fn handles_missing_dictionary() {
    let t = parse_response(HOUSE).unwrap();
    assert_eq!(t.primary, "casa");
    assert_eq!(t.detected_source.as_deref(), Some("en"));
    assert!(t.synonyms.is_empty());
}

#[test]
fn concatenates_multiple_sentences() {
    let t = parse_response(MULTI).unwrap();
    assert_eq!(t.primary, "Hello. How are you?");
    assert_eq!(t.detected_source.as_deref(), Some("pt-BR"));
    assert!(t.synonyms.is_empty());
}

#[test]
fn rejects_invalid_json() {
    assert!(parse_response("not json at all").is_err());
}

#[test]
fn rejects_empty_translation() {
    assert!(parse_response("[[],null,\"en\"]").is_err());
    assert!(parse_response("[null,null,\"en\"]").is_err());
}

#[test]
fn parses_transliteration_rows() {
    let body = "[[[\"翻訳\",\"translation\",null,null,10],[null,null,\"Hon'yaku\",\"tranz\"]],null,\"en\"]";
    let t = parse_response(body).unwrap();
    assert_eq!(t.primary, "翻訳");
    assert_eq!(t.target_translit.as_deref(), Some("Hon'yaku"));
    assert_eq!(t.source_translit.as_deref(), Some("tranz"));
}

#[test]
fn parses_spelling_correction() {
    let body = "[[[\"receber\",\"recieve\",null,null,3]],null,\"en\",null,null,null,0.9,[\"<b><i>receive</i></b>\",\"receive\",[1]]]";
    let t = parse_response(body).unwrap();
    assert_eq!(t.primary, "receber");
    assert_eq!(t.correction.as_deref(), Some("receive"));
}

#[test]
fn no_correction_when_field_empty() {
    let body = "[[[\"casa\",\"house\",null,null,1]],null,\"en\",null,null,null,1.0,[]]";
    let t = parse_response(body).unwrap();
    assert_eq!(t.correction, None);
    assert_eq!(t.source_translit, None);
    assert_eq!(t.target_translit, None);
}

#[test]
fn tolerates_null_back_translations() {
    let body = "[[[\"casa\",\"house\",null,null,1]],[[\"noun\",[\"casa\"],[[\"casa\",null,null,0.9]],\"house\",1]],\"en\"]";
    let t = parse_response(body).unwrap();
    assert_eq!(t.synonyms.len(), 1);
    assert_eq!(t.synonyms[0].word, "casa");
    assert!(t.synonyms[0].back.is_empty());
}

const T_AUTO: &str = include_str!("fixtures/t_auto.json");
const T_EXPLICIT: &str = include_str!("fixtures/t_explicit.json");

#[test]
fn t_format_with_auto_source_yields_primary_and_detected() {
    let t = parse_t_response(T_AUTO).unwrap();
    assert_eq!(t.primary, "good morning. all good?");
    assert_eq!(t.detected_source.as_deref(), Some("pt"));
    assert!(t.synonyms.is_empty());
}

#[test]
fn t_format_with_explicit_source_yields_primary_only() {
    let t = parse_t_response(T_EXPLICIT).unwrap();
    assert_eq!(t.primary, "good morning");
    assert_eq!(t.detected_source, None);
}

#[test]
fn t_format_rejects_empty_and_html() {
    assert!(parse_t_response("[]").is_err());
    assert!(parse_t_response("[[\"\",\"pt\"]]").is_err());
    assert!(parse_t_response("<html>Sorry</html>").is_err());
}
