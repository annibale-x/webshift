//! Live integration tests for search backends.
//!
//! These tests hit real services configured in `test.toml`.
//! Run with: `cargo test -p webgate --features backends -- --ignored`

mod common;

use common::TestConfig;

fn load_or_skip() -> TestConfig {
    TestConfig::load().expect(
        "test.toml not found at workspace root — copy test.toml.example and configure",
    )
}

#[tokio::test]
#[ignore]
async fn searxng_live() {
    let tc = load_or_skip();
    if !tc.backends.searxng.enabled {
        println!("SKIP: searxng not enabled in test.toml");
        return;
    }
    let config = tc.to_webgate_config("searxng");
    let result = webgate::query(&["rust programming language"], &config)
        .await
        .expect("query failed");
    assert!(!result.sources.is_empty(), "expected at least one source");
    assert!(result.stats.fetched > 0, "expected at least one fetch");
    println!(
        "searxng_live: {} sources, {} fetched, {} failed, {} total_chars",
        result.sources.len(),
        result.stats.fetched,
        result.stats.failed,
        result.stats.total_chars
    );
}

#[tokio::test]
#[ignore]
async fn brave_live() {
    let tc = load_or_skip();
    if !tc.backends.brave.enabled {
        println!("SKIP: brave not enabled in test.toml");
        return;
    }
    let config = tc.to_webgate_config("brave");
    let result = webgate::query(&["rust programming language"], &config)
        .await
        .expect("query failed");
    assert!(!result.sources.is_empty(), "expected at least one source");
    assert!(result.stats.fetched > 0, "expected at least one fetch");
    println!(
        "brave_live: {} sources, {} fetched, {} failed, {} total_chars",
        result.sources.len(),
        result.stats.fetched,
        result.stats.failed,
        result.stats.total_chars
    );
}

#[tokio::test]
#[ignore]
async fn tavily_live() {
    let tc = load_or_skip();
    if !tc.backends.tavily.enabled {
        println!("SKIP: tavily not enabled in test.toml");
        return;
    }
    let config = tc.to_webgate_config("tavily");
    let result = webgate::query(&["rust programming language"], &config)
        .await
        .expect("query failed");
    assert!(!result.sources.is_empty(), "expected at least one source");
    assert!(result.stats.fetched > 0, "expected at least one fetch");
    println!(
        "tavily_live: {} sources, {} fetched, {} failed, {} total_chars",
        result.sources.len(),
        result.stats.fetched,
        result.stats.failed,
        result.stats.total_chars
    );
}

#[tokio::test]
#[ignore]
async fn exa_live() {
    let tc = load_or_skip();
    if !tc.backends.exa.enabled {
        println!("SKIP: exa not enabled in test.toml");
        return;
    }
    let config = tc.to_webgate_config("exa");
    let result = webgate::query(&["rust programming language"], &config)
        .await
        .expect("query failed");
    assert!(!result.sources.is_empty(), "expected at least one source");
    assert!(result.stats.fetched > 0, "expected at least one fetch");
    println!(
        "exa_live: {} sources, {} fetched, {} failed, {} total_chars",
        result.sources.len(),
        result.stats.fetched,
        result.stats.failed,
        result.stats.total_chars
    );
}

#[tokio::test]
#[ignore]
async fn serpapi_live() {
    let tc = load_or_skip();
    if !tc.backends.serpapi.enabled {
        println!("SKIP: serpapi not enabled in test.toml");
        return;
    }
    let config = tc.to_webgate_config("serpapi");
    let result = webgate::query(&["rust programming language"], &config)
        .await
        .expect("query failed");
    assert!(!result.sources.is_empty(), "expected at least one source");
    assert!(result.stats.fetched > 0, "expected at least one fetch");
    println!(
        "serpapi_live: {} sources, {} fetched, {} failed, {} total_chars",
        result.sources.len(),
        result.stats.fetched,
        result.stats.failed,
        result.stats.total_chars
    );
}
