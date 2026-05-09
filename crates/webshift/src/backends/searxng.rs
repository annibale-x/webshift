//! SearXNG search backend.
//!
//! Queries a self-hosted SearXNG instance. No API key required.

use super::{BackendResponse, SearchBackend, SearchResult};
use crate::config::SearxngConfig;

#[derive(Debug)]
pub struct SearxngBackend {
    base_url: String,
    client: reqwest::Client,
}

impl SearxngBackend {
    pub fn new(config: &SearxngConfig) -> Self {
        Self {
            base_url: config.url.trim_end_matches('/').to_string(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client"),
        }
    }
}

/// Extract a string field from a JSON value, defaulting to "".
fn jstr<'a>(val: &'a serde_json::Value, key: &str) -> &'a str {
    val.get(key).and_then(serde_json::Value::as_str).unwrap_or("")
}

/// Collect string entries from a JSON array of strings, prefixed with `label:`.
///
/// Some SearXNG variants/forks expose engine errors as flat string arrays
/// under `error_msgs` / `unresponsive_msgs` (the shape mentioned in issue #1).
fn collect_string_msgs(data: &serde_json::Value, key: &str, label: &str) -> Vec<String> {
    data.get(key)
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| format!("{label}: {s}"))
                .collect()
        })
        .unwrap_or_default()
}

/// Collect entries from a JSON array of `[engine_name, reason]` tuples.
///
/// This is the actual shape used by mainline SearXNG for `unresponsive_engines`
/// and `errors`: each entry is a 2-element array (or sometimes a single string
/// when no reason is provided). Discovered against a live SearXNG instance
/// while validating issue #1 — the reporter's description of the JSON shape
/// was inaccurate.
fn collect_tuple_msgs(data: &serde_json::Value, key: &str, label: &str) -> Vec<String> {
    data.get(key)
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(|entry| match entry {
                    serde_json::Value::Array(parts) => {
                        let strs: Vec<&str> =
                            parts.iter().filter_map(|p| p.as_str()).collect();
                        match strs.as_slice() {
                            [name, reason] => format!("{label}: {name}: {reason}"),
                            [single] => format!("{label}: {single}"),
                            _ => format!("{label}: {entry}"),
                        }
                    }
                    serde_json::Value::String(s) => format!("{label}: {s}"),
                    other => format!("{label}: {other}"),
                })
                .collect()
        })
        .unwrap_or_default()
}

#[async_trait::async_trait]
impl SearchBackend for SearxngBackend {
    async fn search(
        &self,
        query: &str,
        num_results: usize,
        lang: Option<&str>,
    ) -> Result<BackendResponse, crate::WebshiftError> {
        let mut params = vec![
            ("q", query.to_string()),
            ("format", "json".to_string()),
            ("pageno", "1".to_string()),
        ];
        if let Some(lang) = lang {
            params.push(("language", lang.to_string()));
        }

        let url = format!("{}/search", self.base_url);
        let resp = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| crate::WebshiftError::Backend(format!("searxng request failed: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(crate::WebshiftError::Backend(format!(
                "searxng HTTP {status}"
            )));
        }

        let data: serde_json::Value = resp
            .json::<serde_json::Value>()
            .await
            .map_err(|e| crate::WebshiftError::Backend(format!("searxng parse error: {e}")))?;

        let empty = vec![];
        let items = data
            .get("results")
            .and_then(serde_json::Value::as_array)
            .unwrap_or(&empty);

        let mut results = Vec::new();
        for item in items {
            if results.len() >= num_results {
                break;
            }
            results.push(SearchResult {
                title: jstr(item, "title").to_string(),
                url: jstr(item, "url").to_string(),
                snippet: jstr(item, "content").to_string(),
            });
        }

        // SearXNG signals engine-side failures via several JSON keys depending
        // on version / fork. Collect from all of them so we cover both the
        // mainline shape (`unresponsive_engines`, `errors` — array of
        // `[name, reason]` tuples) and the shape mentioned in issue #1
        // (`error_msgs`, `unresponsive_msgs` — array of strings).
        let mut warnings = collect_tuple_msgs(&data, "unresponsive_engines", "searxng unresponsive");
        warnings.extend(collect_tuple_msgs(&data, "errors", "searxng engine error"));
        warnings.extend(collect_string_msgs(&data, "error_msgs", "searxng engine error"));
        warnings.extend(collect_string_msgs(&data, "unresponsive_msgs", "searxng unresponsive"));

        Ok(BackendResponse { results, warnings })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn searxng_parses_results() {
        let mock_server = MockServer::start().await;

        let body = serde_json::json!({
            "results": [
                {"title": "Rust Lang", "url": "https://rust-lang.org", "content": "Systems programming"},
                {"title": "Tokio", "url": "https://tokio.rs", "content": "Async runtime for Rust"},
                {"title": "Serde", "url": "https://serde.rs", "content": "Serialization framework"},
            ]
        });

        Mock::given(method("GET"))
            .and(path("/search"))
            .and(query_param("q", "rust"))
            .and(query_param("format", "json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&mock_server)
            .await;

        let config = crate::config::SearxngConfig {
            url: mock_server.uri(),
        };
        let backend = SearxngBackend::new(&config);
        let response = backend.search("rust", 2, None).await.unwrap();

        assert_eq!(response.results.len(), 2);
        assert_eq!(response.results[0].title, "Rust Lang");
        assert_eq!(response.results[0].url, "https://rust-lang.org");
        assert_eq!(response.results[0].snippet, "Systems programming");
        assert_eq!(response.results[1].title, "Tokio");
        assert!(response.warnings.is_empty());
    }

    #[tokio::test]
    async fn searxng_with_lang_param() {
        let mock_server = MockServer::start().await;

        let body = serde_json::json!({
            "results": [
                {"title": "Rust IT", "url": "https://rust-lang.org/it", "content": "Linguaggio di sistema"},
            ]
        });

        Mock::given(method("GET"))
            .and(path("/search"))
            .and(query_param("language", "it"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&mock_server)
            .await;

        let config = crate::config::SearxngConfig {
            url: mock_server.uri(),
        };
        let backend = SearxngBackend::new(&config);
        let response = backend.search("rust", 10, Some("it")).await.unwrap();

        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].title, "Rust IT");
    }

    #[tokio::test]
    async fn searxng_handles_http_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let config = crate::config::SearxngConfig {
            url: mock_server.uri(),
        };
        let backend = SearxngBackend::new(&config);
        let result = backend.search("test", 5, None).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("500"));
    }

    #[tokio::test]
    async fn searxng_handles_empty_results() {
        let mock_server = MockServer::start().await;

        let body = serde_json::json!({"results": []});

        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&mock_server)
            .await;

        let config = crate::config::SearxngConfig {
            url: mock_server.uri(),
        };
        let backend = SearxngBackend::new(&config);
        let response = backend.search("noresults", 5, None).await.unwrap();

        assert!(response.results.is_empty());
        assert!(response.warnings.is_empty());
    }

    #[tokio::test]
    async fn searxng_surfaces_legacy_string_msg_format() {
        // Compatibility test for the alternate JSON shape mentioned in issue
        // #1 (`error_msgs` / `unresponsive_msgs` as flat string arrays).
        // Mainline SearXNG (verified against `2026.5.9+0cba32c15`) uses
        // `unresponsive_engines` tuples instead — covered by the
        // `searxng_surfaces_unresponsive_engines_tuple_format` test below.
        // We support both so a fork that exposes either shape is handled.
        let mock_server = MockServer::start().await;

        let body = serde_json::json!({
            "results": [],
            "error_msgs": [
                "startpage: SearxEngineCaptchaException: redirected to captcha (suspended_time=3600)",
                "karmasearch: SearxEngineAccessDeniedException: HTTP 403 (suspended_time=180)"
            ],
            "unresponsive_msgs": [
                "duckduckgo: SearxEngineNetworkError: connection reset"
            ]
        });

        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&mock_server)
            .await;

        let config = crate::config::SearxngConfig {
            url: mock_server.uri(),
        };
        let backend = SearxngBackend::new(&config);
        let response = backend.search("blocked", 5, None).await.unwrap();

        assert!(response.results.is_empty());
        assert_eq!(response.warnings.len(), 3);
        assert!(response.warnings[0].contains("startpage"));
        assert!(response.warnings[0].contains("CaptchaException"));
        assert!(response.warnings[1].contains("karmasearch"));
        assert!(response.warnings[2].contains("duckduckgo"));
        assert!(response.warnings[2].contains("unresponsive"));
    }

    #[tokio::test]
    async fn searxng_surfaces_unresponsive_engines_tuple_format() {
        // Real SearXNG format observed against a live instance: failures are
        // reported as `unresponsive_engines: [[engine_name, reason], ...]`,
        // not as flat string arrays. This is the shape that the issue #1 fix
        // must cover in production — discovered while validating the fix.
        let mock_server = MockServer::start().await;

        let body = serde_json::json!({
            "results": [],
            "unresponsive_engines": [
                ["brave", "Suspended: too many requests"],
                ["startpage", "SearxEngineCaptchaException: redirected to captcha"]
            ],
            "errors": [
                ["karmasearch", "HTTP error 403"]
            ]
        });

        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&mock_server)
            .await;

        let config = crate::config::SearxngConfig {
            url: mock_server.uri(),
        };
        let backend = SearxngBackend::new(&config);
        let response = backend.search("blocked", 5, None).await.unwrap();

        assert!(response.results.is_empty());
        assert_eq!(response.warnings.len(), 3);
        assert!(response.warnings.iter().any(|w| w.contains("brave") && w.contains("Suspended")));
        assert!(response.warnings.iter().any(|w| w.contains("startpage") && w.contains("CaptchaException")));
        assert!(response.warnings.iter().any(|w| w.contains("karmasearch") && w.contains("403")));
    }

    #[tokio::test]
    async fn searxng_results_with_partial_engine_failures() {
        // Some engines respond with results while others fail. Caller still
        // gets results plus warnings — important: this should NOT be treated
        // as an error by the upper pipeline.
        let mock_server = MockServer::start().await;

        let body = serde_json::json!({
            "results": [
                {"title": "Rust", "url": "https://rust-lang.org", "content": "Systems lang"}
            ],
            "unresponsive_engines": [
                ["brave", "Suspended: too many requests"]
            ]
        });

        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&mock_server)
            .await;

        let config = crate::config::SearxngConfig {
            url: mock_server.uri(),
        };
        let backend = SearxngBackend::new(&config);
        let response = backend.search("rust", 5, None).await.unwrap();

        assert_eq!(response.results.len(), 1);
        assert_eq!(response.warnings.len(), 1);
        assert!(response.warnings[0].contains("brave"));
    }
}
