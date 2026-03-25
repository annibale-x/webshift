//! Live integration test for single-page fetch.
//!
//! Run with: `cargo test -p webgate -- --ignored`

#[tokio::test]
#[ignore]
async fn fetch_real_page() {
    let config = webgate::Config::default();
    let result = webgate::fetch("https://example.com", &config)
        .await
        .expect("fetch failed");

    assert!(!result.text.is_empty(), "expected non-empty text");
    assert!(result.char_count > 0, "expected char_count > 0");
    println!(
        "fetch_real_page: title={:?}, {} chars, truncated={}",
        result.title, result.char_count, result.truncated
    );
}
