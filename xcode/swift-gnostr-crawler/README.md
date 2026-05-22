## swift-gnostr-crawler

Swift client and query-builder for the crawler HTTP surface.

### Dependency chain

`Crawler` → `GnostrTypes`

### What it covers

- relay discovery payloads
- relay process state (`/api/relay/status`, `/start`, `/stop`)
- crawler query URL building (`/query`, `/:nip/query`)
- Nostr REQ wire-message generation for crawler queries
