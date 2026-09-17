use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tt::engine::fallback::{race, Policy};
use tt::engine::Translation;
use tt::error::{Error, Result};

const FAST: Policy = Policy {
    delays: &[Duration::from_millis(20), Duration::from_millis(20)],
    deadline: Duration::from_secs(5),
};

const NO_RETRY: Policy = Policy {
    delays: &[],
    deadline: Duration::from_secs(5),
};

fn ok(text: &str) -> Translation {
    Translation {
        primary: text.to_string(),
        ..Translation::default()
    }
}

type Attempt = Box<dyn Fn() -> Result<Translation> + Send + 'static>;

fn always_ok(text: &'static str) -> Attempt {
    Box::new(move || Ok(ok(text)))
}

fn always_network(reason: &'static str) -> Attempt {
    Box::new(move || Err(Error::Network(reason.to_string())))
}

fn always_parse(reason: &'static str) -> Attempt {
    Box::new(move || Err(Error::Parse(reason.to_string())))
}

fn counting(counter: &Arc<AtomicUsize>, attempt: Attempt) -> Attempt {
    let counter = counter.clone();
    Box::new(move || {
        counter.fetch_add(1, Ordering::SeqCst);
        attempt()
    })
}

fn ok_after(failures: usize, text: &'static str) -> Attempt {
    let seen = AtomicUsize::new(0);
    Box::new(move || {
        if seen.fetch_add(1, Ordering::SeqCst) < failures {
            Err(Error::Network("HTTP 429".to_string()))
        } else {
            Ok(ok(text))
        }
    })
}

fn run(attempts: Vec<(&str, Attempt)>, policy: &Policy) -> (Result<Translation>, Vec<String>) {
    let mut log = Vec::new();
    let result = race(
        attempts.into_iter().map(|(n, a)| (n.to_string(), a)).collect(),
        policy,
        &mut |line: &str| log.push(line.to_string()),
    );
    (result, log)
}

#[test]
fn first_success_wins_silently() {
    let calls = Arc::new(AtomicUsize::new(0));
    let (result, log) = run(
        vec![
            ("one", counting(&calls, always_ok("first"))),
            ("two", counting(&calls, always_ok("second"))),
        ],
        &FAST,
    );
    assert_eq!(result.unwrap().primary, "first");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(log.is_empty());
}

#[test]
fn network_failure_moves_to_the_next_lane() {
    let (result, log) = run(
        vec![("one", always_network("HTTP 429")), ("two", always_ok("second"))],
        &FAST,
    );
    assert_eq!(result.unwrap().primary, "second");
    assert_eq!(log[0], "one failed: HTTP 429, trying two");
}

#[test]
fn parse_failure_is_skipped_and_never_retried() {
    let calls = Arc::new(AtomicUsize::new(0));
    let (result, log) = run(
        vec![
            ("one", counting(&calls, always_parse("bad shape"))),
            ("two", always_network("HTTP 500")),
        ],
        &FAST,
    );
    assert!(result.is_err());
    assert_eq!(log[0], "one failed: bad shape, skipping");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn background_retry_can_win_after_every_lane_failed_once() {
    let (result, log) = run(
        vec![("one", ok_after(1, "retried")), ("two", always_network("HTTP 429"))],
        &FAST,
    );
    assert_eq!(result.unwrap().primary, "retried");
    assert!(log.contains(&"one failed: HTTP 429, trying two".to_string()));
    assert!(log.contains(&"two failed: HTTP 429, waiting for retries".to_string()));
    assert!(log.contains(&"retrying one in 20ms".to_string()));
}

#[test]
fn every_failed_retry_is_logged_on_its_own() {
    let (result, log) = run(vec![("one", always_network("HTTP 429"))], &FAST);
    assert!(result.is_err());
    assert_eq!(log.iter().filter(|l| *l == "retrying one in 20ms").count(), 2);
    assert_eq!(log.iter().filter(|l| *l == "retry of one failed: HTTP 429").count(), 2);
}

#[test]
fn exhausted_retries_report_every_lane() {
    let (result, _) = run(
        vec![("one", always_network("HTTP 429")), ("two", always_parse("bad shape"))],
        &FAST,
    );
    let message = result.unwrap_err().to_string();
    assert!(message.starts_with("network error: all lanes failed: "), "{message}");
    assert!(message.contains("one: HTTP 429"));
    assert!(message.contains("two: bad shape"));
}

#[test]
fn no_delays_means_no_retries() {
    let calls = Arc::new(AtomicUsize::new(0));
    let (result, log) = run(
        vec![("one", counting(&calls, always_network("down")))],
        &NO_RETRY,
    );
    assert!(result.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(log, vec!["one failed: down, waiting for retries".to_string()]);
}

const SHORT_DEADLINE: Policy = Policy {
    delays: &[Duration::from_secs(30)],
    deadline: Duration::from_millis(50),
};

const SLOW_RETRY: Policy = Policy {
    delays: &[Duration::from_secs(30)],
    deadline: Duration::from_secs(60),
};

#[test]
fn deadline_cuts_retries_short() {
    let started = Instant::now();
    let (result, _) = run(vec![("one", always_network("HTTP 429"))], &SHORT_DEADLINE);
    assert!(started.elapsed() < Duration::from_secs(2));
    let message = result.unwrap_err().to_string();
    assert!(message.contains("deadline of 50ms reached"), "{message}");
}

#[test]
fn winner_returns_while_another_retry_still_sleeps() {
    let started = Instant::now();
    let (result, _) = run(
        vec![("one", always_network("HTTP 429")), ("two", always_ok("second"))],
        &SLOW_RETRY,
    );
    assert_eq!(result.unwrap().primary, "second");
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn parse_failure_during_retry_stops_that_lane() {
    let seen = Arc::new(AtomicUsize::new(0));
    let flaky: Attempt = {
        let seen = seen.clone();
        Box::new(move || match seen.fetch_add(1, Ordering::SeqCst) {
            0 => Err(Error::Network("HTTP 429".to_string())),
            _ => Err(Error::Parse("bad shape".to_string())),
        })
    };
    let (result, log) = run(vec![("one", flaky)], &FAST);
    assert!(result.is_err());
    assert_eq!(seen.load(Ordering::SeqCst), 2);
    assert!(log.contains(&"retry of one failed: bad shape, giving up".to_string()));
}
