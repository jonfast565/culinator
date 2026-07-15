use super::PreparedDataset;
#[test]
fn accepts_extracted_directory() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("food.csv"),
        "fdc_id,data_type,description\n",
    )
    .unwrap();
    let source = PreparedDataset::open(dir.path()).unwrap();
    assert_eq!(source.root(), dir.path());
}
