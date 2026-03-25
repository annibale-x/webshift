# webgate

**Denoised web search library and MCP server** — native Rust port of [mcp-webgate](https://github.com/annibale-x/mcp-webgate).

Single static binary, zero runtime dependencies. Feeds clean, right-sized web
content to LLM agents without flooding the context window.

---

## How it works

```
Question
  │
  ├─ (optional) LLM query expansion → multiple search variants
  │
  ├─ Search via backend (SearXNG, Brave, Tavily, Exa, SerpAPI)
  │
  ├─ Deduplicate + filter binary URLs
  │
  ├─ Streaming fetch with per-page size cap
  │
  ├─ HTML cleaning → plain text (noise elements, scripts, nav removed)
  │
  ├─ Unicode/BiDi sterilization
  │
  ├─ BM25 deterministic reranking
  │   └─ (optional) LLM-assisted tier-2 reranking
  │
  ├─ Budget-aware truncation across all sources
  │
  ├─ (optional) LLM Markdown summary with inline citations
  │
  └─ Structured JSON output
```

---

## Installation

### From source (recommended during development)

```bash
cargo install --path crates/webgate-mcp
```

### From crates.io (when published)

```bash
cargo install webgate-mcp
```

The binary is called `mcp-webgate`.

### As a library

```toml
# Full pipeline (search + fetch + clean + rerank)
webgate = "0.1"

# Cleaner + fetcher only (no search backends)
webgate = { version = "0.1", default-features = false }

# Everything including LLM features
webgate = { version = "0.1", features = ["llm"] }
```

---

## Quick start

### 1. Set up a search backend

The easiest option is [SearXNG](https://docs.searxng.org/) — free, self-hosted,
no API key:

```bash
docker run -d -p 4000:8080 searxng/searxng
```

### 2. Configure the MCP client

Add to your MCP client config (e.g. Claude Desktop, Claude Code, Cursor):

```json
{
  "mcpServers": {
    "webgate": {
      "command": "mcp-webgate",
      "args": ["--default-backend", "searxng"]
    }
  }
}
```

That's it. The agent now has access to `webgate_query`, `webgate_fetch`, and
`webgate_onboarding` tools.

---

## MCP tools

| Tool | Description |
|------|-------------|
| `webgate_query` | Full search pipeline: search + fetch + clean + rerank + (optional) summarize |
| `webgate_fetch` | Single page fetch and clean |
| `webgate_onboarding` | Returns a JSON guide for the agent (budgets, backends, tips) |

### `webgate_query` parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `queries` | string or list | required | Search query or list of queries |
| `num_results` | integer | 5 | Results per query |
| `lang` | string | none | Language filter (e.g. `"en"`) |
| `backend` | string | config default | Override search backend |

### `webgate_fetch` parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `url` | string | required | URL to fetch |
| `max_chars` | integer | 8000 | Maximum characters in output |

---

## Client integrations

### Claude Desktop

```json
{
  "mcpServers": {
    "webgate": {
      "command": "mcp-webgate",
      "args": ["--default-backend", "searxng"]
    }
  }
}
```

### Claude Code

```bash
claude mcp add webgate -- mcp-webgate --default-backend searxng
```

### Cursor / Windsurf / VS Code

Add to MCP settings:

```json
{
  "mcpServers": {
    "webgate": {
      "command": "mcp-webgate",
      "args": ["--default-backend", "searxng"]
    }
  }
}
```

### With a config file

```bash
mcp-webgate --config /path/to/webgate.toml
```

See [`examples/webgate.toml`](examples/webgate.toml) for a full configuration example.

---

## Configuration

Resolution order (highest priority first):

1. **CLI args** — `--default-backend`, `--debug`, etc.
2. **Environment variables** — `WEBGATE_*` prefix
3. **Config file** — `webgate.toml`
4. **Built-in defaults**

### Key settings

| Setting | Default | Description |
|---------|---------|-------------|
| `max_query_budget` | 32,000 chars | Total character budget across all sources |
| `max_result_length` | 8,000 chars | Per-page character cap |
| `max_total_results` | 20 | Hard cap on results per call |
| `max_download_mb` | 1 MB | Streaming download cap per page |
| `search_timeout` | 8s | Timeout for search + fetch |
| `results_per_query` | 5 | Results requested per query |
| `oversampling_factor` | 2 | Oversample ratio for gap filling |
| `adaptive_budget` | false | Allocate budget proportionally to BM25 scores |

### Environment variables

All settings can be overridden via `WEBGATE_` prefixed env vars:

```bash
WEBGATE_MAX_QUERY_BUDGET=64000
WEBGATE_BACKENDS_DEFAULT=brave
WEBGATE_BRAVE_API_KEY=BSA-xxx
WEBGATE_LLM_ENABLED=true
WEBGATE_LLM_BASE_URL=http://localhost:11434/v1
WEBGATE_LLM_MODEL=gemma3:27b
```

---

## Search backends

| Backend | API key | Notes |
|---------|---------|-------|
| **SearXNG** | none | Self-hosted, free. Default: `http://localhost:4000` |
| **Brave** | required | Free tier available at [brave.com/search/api](https://brave.com/search/api/) |
| **Tavily** | required | [tavily.com](https://tavily.com/) |
| **Exa** | required | Neural search. [exa.ai](https://exa.ai/) |
| **SerpAPI** | required | Multi-engine proxy. [serpapi.com](https://serpapi.com/) |

Configure in `webgate.toml`:

```toml
[backends]
default = "brave"

[backends.brave]
api_key = "BSA-xxx"

[backends.searxng]
url = "http://localhost:4000"
```

---

## LLM features (optional)

All LLM features are **opt-in** — disabled by default, no data leaves your
machine unless you enable them.

| Feature | What it does |
|---------|-------------|
| **Query expansion** | Single query → N complementary search variants |
| **Summarization** | Markdown report with inline `[1]` `[2]` citations |
| **LLM reranking** | Tier-2 reranking on top of deterministic BM25 |

Works with any OpenAI-compatible API (OpenAI, Ollama, vLLM, LM Studio, etc.):

```toml
[llm]
enabled = true
base_url = "http://localhost:11434/v1"   # Ollama
model = "gemma3:27b"
# api_key = ""                           # not needed for Ollama
```

---

## Anti-flooding protections

These are the core value proposition — they exist in every code path:

| Protection | Description |
|------------|-------------|
| `max_download_mb` | Streaming cap per page — never buffers full response |
| `max_result_length` | Hard cap on characters per cleaned page |
| `max_query_budget` | Total character budget across all sources |
| `max_total_results` | Hard cap on results per call |
| Binary filter | `.pdf`, `.zip`, `.exe`, etc. filtered **before** any network request |
| Streaming fetch | Uses `bytes_stream()` with size check — never `response.text()` |
| Unicode sterilization | BiDi control chars, zero-width chars, surrogate pairs removed |

---

## Library usage

Use `webgate` as a Rust library in your own projects:

```rust
use webgate::{Config, clean, fetch, query};

// Clean raw HTML
let result = clean("<html><body><p>Hello world</p></body></html>", 8000);
println!("{}", result.text);

// Fetch and clean a single page
let config = Config::default();
let page = fetch("https://example.com", &config).await?;
println!("{}", page.text);

// Full search pipeline
let results = query(&["rust async programming"], &config).await?;
for source in &results.sources {
    println!("[{}] {} — {} chars", source.id, source.title, source.content.len());
}
```

### Feature flags

| Feature | Default | Enables |
|---------|---------|---------|
| `backends` | on | All 5 search backends + query pipeline |
| `llm` | off | LLM client, query expander, summarizer, LLM reranking |

---

## CLI flags

```
mcp-webgate [OPTIONS]

Options:
  --config <PATH>            Path to webgate.toml config file
  --default-backend <NAME>   Override default search backend
  --debug                    Enable debug logging
  --trace                    Enable trace-level logging
  --log-file <PATH>          Log to file instead of stderr
  --llm-enabled              Enable LLM features
  --llm-base-url <URL>       LLM API base URL
  --llm-api-key <KEY>        LLM API key
  --llm-model <MODEL>        LLM model name
```

---

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full development guide.

```bash
cargo build                   # build all crates
cargo test                    # run unit tests (mocked, no services needed)
cargo test -- --ignored       # run integration tests (requires test.toml)
cargo run -p robot -- harness "your query"   # diagnostic harness
```

---

## License

MIT
