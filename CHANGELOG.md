# Changelog

All notable changes to this project will be documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.0.1] - 2026-03-25

### Added

- Workspace setup: `webgate` (lib), `webgate-mcp` (bin), `robot` (dev tool)
- `robot` dev tool with `bump`, `test`, `promote`, `unpromote`, `publish` commands
- Shared workspace versioning via `[workspace.package]`
- `scraper/cleaner.rs` — HTML cleaning with `scraper`/html5ever + text sterilization pipeline
- `scraper/fetcher.rs` — reqwest concurrent fetcher with streaming cap, UA rotation, retry
- `utils/url.rs` — URL sanitize, dedup, binary extension filter
- `lib.rs` — `webgate::clean()` and `webgate::fetch()` public API (initial scaffold)
- Release profile with LTO, strip, size optimization
- CLAUDE.md, CONTRIBUTING.md, PLAN.md project documentation
