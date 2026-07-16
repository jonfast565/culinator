#[test]
fn release_url_targets_full_csv_archive() {
    assert!(crate::DEFAULT_FULL_DOWNLOAD_URL.contains("FoodData_Central_csv"));
}
