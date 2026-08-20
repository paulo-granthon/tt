use tt::engine::google::parse_response;

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
fn tolerates_null_back_translations() {
    let body = "[[[\"casa\",\"house\",null,null,1]],[[\"noun\",[\"casa\"],[[\"casa\",null,null,0.9]],\"house\",1]],\"en\"]";
    let t = parse_response(body).unwrap();
    assert_eq!(t.synonyms.len(), 1);
    assert_eq!(t.synonyms[0].word, "casa");
    assert!(t.synonyms[0].back.is_empty());
}
