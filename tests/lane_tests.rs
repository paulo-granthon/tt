use tt::engine::google::{lane, LANES};
use tt::engine::Query;

const QUERY: Query = Query {
    sl: "pt",
    tl: "en",
    text: "bom dia",
};

fn pairs(name: &str) -> (String, String, Vec<(String, String)>, ureq::Request) {
    let request = (lane(name).unwrap().build)(&ureq::Agent::new(), &QUERY);
    let url = request.request_url().unwrap();
    (
        url.host().to_string(),
        url.path().to_string(),
        url.query_pairs()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        request,
    )
}

#[test]
fn lane_names_are_unique_and_ordered_rich_first() {
    let names: Vec<&str> = LANES.iter().map(|l| l.name).collect();
    assert_eq!(
        names,
        vec![
            "googleapis/dict-chrome-ex",
            "googleapis/at",
            "clients5/dict-chrome-ex",
            "clients5/at",
            "google.com/dict-chrome-ex",
            "google.com/at",
            "googleapis/t/dict-chrome-ex",
            "googleapis/t/at",
        ]
    );
}

#[test]
fn lookup_by_name() {
    assert!(lane("googleapis/at").is_some());
    assert!(lane("nope").is_none());
}

#[test]
fn rich_lane_requests_every_dictionary_section() {
    let (host, path, query, request) = pairs("googleapis/dict-chrome-ex");
    assert_eq!(host, "translate.googleapis.com");
    assert_eq!(path, "/translate_a/single");
    let dt: Vec<&str> = query.iter().filter(|(k, _)| k == "dt").map(|(_, v)| v.as_str()).collect();
    assert_eq!(dt, vec!["t", "bd", "at", "rm", "qc"]);
    assert!(query.contains(&("client".to_string(), "dict-chrome-ex".to_string())));
    assert!(query.contains(&("sl".to_string(), "pt".to_string())));
    assert!(query.contains(&("tl".to_string(), "en".to_string())));
    assert!(query.contains(&("q".to_string(), "bom dia".to_string())));
    assert!(request.header("User-Agent").unwrap().contains("Chrome/"));
    assert_eq!(request.header("Referer"), Some("https://translate.google.com/"));
}

#[test]
fn hosts_and_clients_vary_per_lane() {
    assert_eq!(pairs("clients5/at").0, "clients5.google.com");
    assert_eq!(pairs("google.com/dict-chrome-ex").0, "translate.google.com");
    assert!(pairs("google.com/at").2.contains(&("client".to_string(), "at".to_string())));
}

#[test]
fn degraded_lanes_hit_the_t_endpoint_without_dictionary_params() {
    let (host, path, query, _) = pairs("googleapis/t/at");
    assert_eq!(host, "translate.googleapis.com");
    assert_eq!(path, "/translate_a/t");
    assert!(query.iter().all(|(k, _)| k != "dt"));
    assert!(query.contains(&("client".to_string(), "at".to_string())));
    assert!(query.contains(&("q".to_string(), "bom dia".to_string())));
}
