## swift-gnostr-crawler

Swift client and query-builder for the crawler HTTP surface.

### Dependency chain

`Crawler` → `GnostrTypes`

`Crawler` → `RustCrawlerBridge` → `crawler-ffi` (`cdylib`) → `gnostr-crawler`

### What it covers

- relay discovery payloads
- relay process state (`/api/relay/status`, `/start`, `/stop`)
- crawler query URL building (`/query`, `/:nip/query`)
- Nostr REQ wire-message generation for crawler queries
- Rust-backed normalization helpers for crawler metadata when the dylib is present

### Runtime loading

The crawler bridge looks for `GNOSTR_CRAWLER_FFI_LIBRARY`, then falls back to:

- `Rust/crawler-ffi/target/debug/libcrawler_ffi.dylib`
- `Rust/crawler-ffi/target/release/libcrawler_ffi.dylib`
