# Changelog

* **2026-03-25: v0.1.7** - Integration tests, diagnostic harness, docs & examples
  * feat(test): `test.toml` config for live integration tests (`#[ignore]`, per-backend/LLM `enabled` flags)
  * feat(test): `TestConfig` struct with `to_webgate_config()` conversion
  * feat(test): integration tests — 5 backend live tests, 3 LLM pipeline tests, 1 fetch test
  * feat(robot): `harness` subcommand — full pipeline runner with BM25 scores, budget stats, timing
  * docs(readme): comprehensive README with installation, configuration, backends, LLM, integrations
  * docs(contributing): integration tests and diagnostic harness documentation
  * docs(plan): check off integration test infrastructure in M4
  * chore(examples): `webgate.toml`, `webgate-ollama.toml`, `webgate-minimal.toml`, `webgate-brave.toml`
  * chore(examples): `claude-desktop.json`, `claude-desktop-ollama.json` MCP client configs
  * chore(examples): `test.toml.example` template for contributor integration testing

---

* **2026-03-25: v0.1.6** - M4 complete — LLM features
  * feat(llm): `LlmClient` — async OpenAI-compatible chat completions client (reqwest, no SDK dependency)
  * feat(llm): `expand_queries()` — single query → N complementary queries via LLM, with JSON fence stripping and fallback
  * feat(llm): `summarize_results()` — Markdown report with inline citations `[1]`, `[2]`, etc.
  * feat(reranker): `rerank_llm()` — LLM-assisted tier-2 reranking (behind `llm` feature flag), falls back to input order on error
  * feat(query): LLM expansion integrated before search (single query input, `llm.expansion_enabled`)
  * feat(query): LLM reranking integrated after BM25 (`llm.llm_rerank_enabled`)
  * feat(query): LLM summarization integrated after reranking (`llm.summarization_enabled`); errors captured in `llm_summary_error` field
  * feat(mcp): `--llm-enabled/model/base-url/api-key/timeout/expansion-enabled/summarization-enabled/rerank-enabled/max-summary-words/input-budget-factor` CLI args
  * feat(mcp): `webgate-mcp` binary now builds with `llm` feature enabled
  * test(llm): `LlmClient` — content, disabled, HTTP error, API key header (4 tests)
  * test(llm): expander — variants, n=1 skip, LLM error fallback, markdown fences (4 tests)
  * test(llm): summarizer — Markdown output, error propagation (2 tests)
  * test(reranker): `rerank_llm` — LLM judgment ordering, fallback on error (2 tests)
  * test(pipeline): LLM pipeline — expansion, summarization, error capture (3 tests)
  * test(mcp): `cli_parse_llm_args` — all LLM CLI flag deserialization (1 test)
  * docs(plan): check off all M4 tasks

---

* **2026-03-25: v0.1.5** - M3 complete — search backends + query pipeline
  * feat(backends): `SearchBackend` trait + `create_backend` factory with 5 implementations (SearXNG, Brave, Tavily, Exa, SerpAPI)
  * feat(reranker): BM25 deterministic reranking + adaptive budget redistribution
  * feat(query): full pipeline — search → dedup → fetch → clean → rerank → assemble with oversampling and gap filler
  * feat(query): `webgate::query()` and `webgate::query_with_options()` public API
  * feat(mcp): `webgate_query` tool with `StringOrList` queries param, backend override, lang support
  * test(backends): factory tests (4), SearXNG wiremock tests (4)
  * test(reranker): BM25 scoring, ranking, budget redistribution (6 tests)
  * test(pipeline): integration tests with mock search + mock pages (8 tests)
  * test(mcp): QueryParams deserialization tests (3 tests)
  * docs(plan): check off all M3 tasks

---

* **2026-03-25: v0.1.4** - M2 tests + robot auto-commit
  * test(mcp): server construction, onboarding JSON, CLI parsing, param deserialization (10 new tests)
  * feat(robot): `bump` now auto-commits all tracked changes (not just Cargo/CHANGELOG)
  * docs(plan): add M2 test task

---

* **2026-03-25: v0.1.3** - M2 complete — MCP server with fetch tool
  * feat(mcp): `webgate_fetch` tool via `rmcp` 1.x with stdio transport
  * feat(mcp): `webgate_onboarding` tool — operational guide JSON (matches Python)
  * feat(mcp): CLI argument parsing with clap (`--config`, `--default-backend`, `--debug`, `--trace`, `--log-file`)
  * feat(mcp): server instructions for AI agent guidance
  * feat(mcp): tracing-subscriber logging to stderr or file
  * docs(plan): check off all M2 tasks

---

* **2026-03-25: v0.1.2** - M1 complete — config, tests
  * feat(config): TOML loading + `WEBGATE_*` env var overrides + tests
  * test(cleaner): port full Python test suite (12 new tests)
  * test(fetcher): wiremock retry tests — 429, 503, exhausted retries, 404 no-retry (6 new tests)
  * docs(plan): check off all M1 tasks

---

* **2026-03-25: v0.1.1** - Initial workspace scaffold
  * feat(workspace): setup `webgate` (lib), `webgate-mcp` (bin), `robot` (dev tool)
  * feat(robot): `bump`, `test`, `promote`, `unpromote`, `publish` commands
  * feat(cleaner): HTML cleaning with `scraper`/html5ever + text sterilization pipeline
  * feat(fetcher): reqwest concurrent fetcher with streaming cap, UA rotation, retry
  * feat(url): sanitize, dedup, binary extension filter
  * feat(lib): `webgate::clean()` and `webgate::fetch()` public API (initial scaffold)
  * chore(build): release profile with LTO, strip, size optimization
  * docs: CLAUDE.md, CONTRIBUTING.md, PLAN.md

---
