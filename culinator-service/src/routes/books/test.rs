use super::*;
#[tokio::test]
async fn list_is_empty_for_fresh_state() {
    let Json(books) = list(State(crate::state::test_state()))
        .await
        .expect("list books");
    assert!(books.is_empty());
}
