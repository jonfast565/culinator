#[test]
fn default_url_is_https() {
    assert!(culinator_nutrition_fdc::DEFAULT_FULL_DOWNLOAD_URL.starts_with("https://"));
}
