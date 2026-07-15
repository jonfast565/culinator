use super::*;
#[test]
fn parses_service_options() {
    let options =
        Options::parse(["--port".to_owned(), "1234".to_owned()].into_iter()).expect("parse");
    assert_eq!(options.port, 1234);
}
#[test]
fn rejects_unknown_option() {
    assert!(Options::parse(["--wat".to_owned()].into_iter()).is_err());
}
