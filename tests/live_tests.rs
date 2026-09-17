use tt::engine::google::{lane, Google};
use tt::engine::{Engine, Query};

const QUERY: Query = Query {
    sl: "pt",
    tl: "en",
    text: "bom dia",
};

fn probe(name: &str) {
    let translation = lane(name).unwrap().call(&ureq::Agent::new(), &QUERY).unwrap();
    assert!(!translation.primary.is_empty(), "{name} returned an empty translation");
}

#[test]
#[ignore]
fn lane_googleapis_dict_chrome_ex() {
    probe("googleapis/dict-chrome-ex");
}

#[test]
#[ignore]
fn lane_googleapis_at() {
    probe("googleapis/at");
}

#[test]
#[ignore]
fn lane_clients5_dict_chrome_ex() {
    probe("clients5/dict-chrome-ex");
}

#[test]
#[ignore]
fn lane_clients5_at() {
    probe("clients5/at");
}

#[test]
#[ignore]
fn lane_google_com_dict_chrome_ex() {
    probe("google.com/dict-chrome-ex");
}

#[test]
#[ignore]
fn lane_google_com_at() {
    probe("google.com/at");
}

#[test]
#[ignore]
fn lane_googleapis_t_dict_chrome_ex() {
    probe("googleapis/t/dict-chrome-ex");
}

#[test]
#[ignore]
fn lane_googleapis_t_at() {
    probe("googleapis/t/at");
}

#[test]
#[ignore]
fn engine_end_to_end() {
    let mut log = Vec::new();
    let translation = Google::new()
        .translate(QUERY, &mut |line| log.push(line.to_string()))
        .unwrap();
    assert_eq!(translation.primary.to_lowercase(), "good morning");
    assert!(log.is_empty(), "first lane should succeed silently, got {log:?}");
}
