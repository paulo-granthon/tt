use tt::input::{resolve, Source};

#[test]
fn file_wins_over_everything() {
    let s = resolve(None, Some("a.txt".to_string()), true).unwrap();
    assert_eq!(s, Source::File("a.txt".to_string()));
}

#[test]
fn positional_text_is_used() {
    let s = resolve(Some("hello".to_string()), None, true).unwrap();
    assert_eq!(s, Source::Text("hello".to_string()));
}

#[test]
fn dash_text_means_stdin() {
    let s = resolve(Some("-".to_string()), None, true).unwrap();
    assert_eq!(s, Source::Stdin);
}

#[test]
fn no_text_with_pipe_reads_stdin() {
    let s = resolve(None, None, false).unwrap();
    assert_eq!(s, Source::Stdin);
}

#[test]
fn no_text_on_a_tty_is_an_error() {
    assert!(resolve(None, None, true).is_err());
}

#[test]
fn file_ignores_stdin_tty_state() {
    let a = resolve(None, Some("a.txt".to_string()), true).unwrap();
    let b = resolve(None, Some("a.txt".to_string()), false).unwrap();
    assert_eq!(a, b);
}
