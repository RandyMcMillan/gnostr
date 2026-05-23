# swift-gnostr-ff-relay

Swift wrapper for the Rust relay crate using FFI only.

### Dependency chain

`FFRelay` → `RustRelayBridge` → `relay-ffi` (`cdylib`) → `gnostr-relay` Rust crate

`RelayGUI` → `FFRelay` → `relay-ffi`

### What it covers

- relay configuration defaults from the Rust relay CLI
- relay listen-endpoint normalization
- relay process and discovery models for FFI payloads
- cross-platform SwiftUI dashboard for iOS and macOS

### Runtime loading

The bridge looks for `GNOSTR_RELAY_FFI_LIBRARY`, then falls back to:

- `Rust/relay-ffi/target/debug/librelay_ffi.dylib`
- `Rust/relay-ffi/target/release/librelay_ffi.dylib`
