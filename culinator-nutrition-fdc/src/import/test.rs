#[test]
fn release_url_targets_full_csv_archive() {
    assert!(crate::DEFAULT_FULL_DOWNLOAD_URL.contains("FoodData_Central_csv"));
}

#[test]
fn loose_i64_parses_usda_float_ranks() {
    use super::parse_loose_i64;

    assert_eq!(parse_loose_i64("280.0").unwrap(), 280);
    assert_eq!(parse_loose_i64("1110").unwrap(), 1110);
    assert!(parse_loose_i64("Oils Edible").is_err());
}
