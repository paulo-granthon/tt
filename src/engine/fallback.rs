use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::engine::Translation;
use crate::error::{Error, Result};

pub struct Policy {
    pub delays: &'static [Duration],
    pub deadline: Duration,
}

pub const DEFAULT_POLICY: Policy = Policy {
    delays: &[
        Duration::from_secs(1),
        Duration::from_secs(2),
        Duration::from_secs(4),
    ],
    deadline: Duration::from_secs(20),
};

enum Event {
    Log(String),
    Done(String, Result<Translation>),
}

fn reason(error: &Error) -> String {
    match error {
        Error::Network(m) | Error::Parse(m) => m.clone(),
        other => other.to_string(),
    }
}

pub fn race<F>(
    attempts: Vec<(String, F)>,
    policy: &Policy,
    log: &mut dyn FnMut(&str),
) -> Result<Translation>
where
    F: Fn() -> Result<Translation> + Send + 'static,
{
    let started = Instant::now();
    let (tx, rx) = channel();
    let mut failures = Vec::new();
    let mut lanes = attempts.into_iter().peekable();
    while let Some((name, attempt)) = lanes.next() {
        match attempt() {
            Ok(translation) => return Ok(translation),
            Err(Error::Network(why)) => {
                let next = lanes
                    .peek()
                    .map_or("waiting for retries".to_string(), |(n, _)| format!("trying {n}"));
                log(&format!("{name} failed: {why}, {next}"));
                if policy.delays.is_empty() {
                    failures.push(format!("{name}: {why}"));
                } else {
                    retry_in_background(name, attempt, why, policy.delays, tx.clone());
                }
            }
            Err(error) => {
                let why = reason(&error);
                log(&format!("{name} failed: {why}, skipping"));
                failures.push(format!("{name}: {why}"));
            }
        }
        while let Ok(event) = rx.try_recv() {
            if let Some(translation) = absorb(event, log, &mut failures) {
                return Ok(translation);
            }
        }
    }
    drop(tx);
    loop {
        let remaining = policy.deadline.saturating_sub(started.elapsed());
        match rx.recv_timeout(remaining) {
            Ok(event) => {
                if let Some(translation) = absorb(event, log, &mut failures) {
                    return Ok(translation);
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {
                failures.push(format!("deadline of {:?} reached", policy.deadline));
                break;
            }
        }
    }
    Err(Error::Network(format!("all lanes failed: {}", failures.join("; "))))
}

fn absorb(event: Event, log: &mut dyn FnMut(&str), failures: &mut Vec<String>) -> Option<Translation> {
    match event {
        Event::Log(line) => log(&line),
        Event::Done(_, Ok(translation)) => return Some(translation),
        Event::Done(name, Err(error)) => failures.push(format!("{name}: {}", reason(&error))),
    }
    None
}

fn retry_in_background<F>(
    name: String,
    attempt: F,
    first: String,
    delays: &'static [Duration],
    tx: Sender<Event>,
) where
    F: Fn() -> Result<Translation> + Send + 'static,
{
    thread::spawn(move || {
        let mut last = first;
        for delay in delays {
            let _ = tx.send(Event::Log(format!("retrying {name} in {delay:?}")));
            thread::sleep(*delay);
            match attempt() {
                Ok(translation) => {
                    let _ = tx.send(Event::Done(name, Ok(translation)));
                    return;
                }
                Err(Error::Network(why)) => {
                    let _ = tx.send(Event::Log(format!("retry of {name} failed: {why}")));
                    last = why;
                }
                Err(error) => {
                    let why = reason(&error);
                    let _ = tx.send(Event::Log(format!("retry of {name} failed: {why}, giving up")));
                    let _ = tx.send(Event::Done(name, Err(Error::Parse(why))));
                    return;
                }
            }
        }
        let _ = tx.send(Event::Done(name, Err(Error::Network(last))));
    });
}
