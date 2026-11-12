# TODO — p2p/fractal

Findings from the review of `714634c8` (`feat(p2p/example): add --clone to
fractal_swarm_perfect_ip`), in severity order. Each finding carries its
gap-fill plan: the change, the file, and how it is verified.

## 1. Empty-placeholder slice is a correctness bug (ship-blocker)

`run_fractal_engine` answers a missing slice with
`ProtocolSlice { data: Vec::new(), is_parity: false }`. `Header` carries no
length field, so a receiver cannot distinguish "peer doesn't have it" from
"a zero-length leaf" — a placeholder counted as received silently corrupts
`reconstruct_payload`. Fix: respond with an explicit error/None, or add the
payload length to `Header` and treat length-0 data as absent.

### Plan 1 — explicit absence (no wire ambiguity)

- Change the repair response type from `ProtocolSlice` to
  `Result<ProtocolSlice, String>` in `FractalBehaviour`
  (p2p/src/fractal/fractal_ip_core.rs): a missing id answers
  `Err("not held: <id>")`. `run_fractal_engine`'s placeholder
  disappears entirely.
- Receiver side (`request_all_slices` in the tests /
  `fractal_ip.rs`): an `Err` response counts as still-missing —
  never inserted into `IntegrityManager`.
- serde supports `Result` in the JSON codec, so no codec change is
  needed for this step (binary codec is plan 3).
- Verify: new test — request a missing id, assert `Err`; reconstruct
  with one peer lacking a leaf, assert parity recovery still completes
  the payload byte-exactly.

## 2. Wire encoding: JSON → binary

`request_response::json::Behaviour` costs ~2×+ wire overhead plus parse
cost on every ~730-byte slice. Keep JSON as a debug/inspection mode; make
the production codec binary (or raw-leaf CID frames — see #3).

### Plan 2 — postcard codec beside JSON

- Add `postcard` (serde, tiny, no_std-friendly) to `gnostr-p2p`.
  Derive `Serialize`/`Deserialize` on `ProtocolSlice`/`Header` (they
  are plain data structs already).
- `FractalBehaviour` gains a second behaviour:
  `repair_bin: request_response::cbor::Behaviour<...>` — or the
  postcard equivalent — on protocol id `/fractal/repair/1.1.0/postcard`;
  the JSON behaviour stays on `/fractal/repair/1.0.0/json` for
  debugging. Default: binary; `--json` flag on the example selects the
  text protocol.
- Verify: round-trip encode/decode test on a real packet batch;
  measure and record the size delta in the commit message (expect
  ~50%+ smaller on the wire).

## 3. Per-slice authentication (makes repair trustless)

Slices are trusted by manifest membership; `verify_integrity` only catches
bad parities when siblings are present. `sha256_hex` already exists in the
module — put per-slice hashes in the manifest so every slice is
self-verifying. Better: make each slice a raw-leaf CID, so a serving peer
proves content by address and any gateway can backfill a missing slice.

### Plan 3 — manifest v2: {id, offset, len, sha256}

- `generate_manifest` (types/src/utils/protocol_slice.rs) v2: each
  entry becomes `{ id, offset, len, sha256 }` — offset/len come from
  the same tree walk that emits the ids (this also closes #5: ordering
  no longer relies on `seq_num`).
- `IntegrityManager::record_slice` verifies `sha256_hex(&slice.data)`
  against the manifest entry and REJECTS mismatches (count them in a
  `rejected` list — do not store).
- `verify_integrity` keeps its parity check; add the per-slice hash
  check so a manifest-less store still gets verified on load.
- CID upgrade path (later, noted not built): slices as raw-leaf CIDs —
  the sha256 in the manifest is already the multihash digest, so the
  manifest entry can gain a `cid` field without a format break.
- Verify: corrupted-slice rejection test; a manifest with a wrong hash
  fails closed; `load_from_disk` rejects tampered persisted state.

## 4. One-loss-per-pair ceiling

XOR-tree recovers exactly one missing sibling per pair; a pair losing
children *and* parity is unrecoverable. Document the ceiling in the
protocol docs, or evaluate a real erasure code (Reed-Solomon / RaptorQ)
for lossy links. Fine as a repair-assist layer either way.

### Plan 4 — document the ceiling, with a test that pins it

- Module docs (types/src/utils/protocol_slice.rs header): state the
  ceiling plainly — one loss per pair; two losses in a pair = subtree
  unrecoverable; this is a repair-assist layer, not a primary transport.
- Test that pins the behavior: drop two slices of one pair, assert
  recovery reports unrecoverable (not silent garbage).
- Defer Reed-Solomon/RaptorQ evaluation to a follow-up issue; the
  manifest v2 (#3) is the seam a k-of-n scheme would plug into.

## 5. `reconstruct_payload` invariant assertion

It sorts by `seq_num` and concatenates, relying on "seq order == byte
order across the recursive tree." True today, implicit forever — assert
it (or key the manifest by (offset, len) instead of tree ids).

### Plan 5 — subsumed by plan 3

Manifest v2 keys slices by (offset, len), so `reconstruct_payload`
orders by offset and asserts total length == sum(manifest lens). No
`seq_num` reliance remains. If plan 3 slips, the fallback is a
`debug_assert!(leaves.windows(2).all(|w| w[0].header.seq_num < w[1].header.seq_num))`
in `reconstruct_payload`.

## 6. `--clone` verification: add `git fsck --strict`

The current check (clone → HEAD match → clean tree) proves the
reconstruction is a valid repo. Add `git fsck --strict` on the
reconstructed directory to also catch object-store corruption the clone
might tolerate.

### Plan 6 — fsck before clone

- In `clone_and_verify_reconstructed_repo`
  (p2p/src/fractal/fractal_ip.rs): run `git fsck --strict` on the
  reconstructed directory first; on failure, surface fsck's output and
  stop. Then the existing clone → HEAD-match → clean-tree flow.
- Verify: run the example against the `web/crlf.git` fixture with
  `--clone` (the existing `#[ignore]` networked test path covers the
  sync side; the fsck step is local and runs in the example's own
  verification).

## Later / integration

- Repair-by-id is the natural loss path for a git transport: primary =
  stream, repair = slice requests by id. Keep the protocol id stable.
- `IntegrityManager::persist` format: version it before other tools
  depend on it.

### Plan 7 — persist format versioning (small, do first among laters)

- Add `"v": 1` to the persisted JSON in `IntegrityManager::persist`;
  `load_from_disk` refuses (with a clear error) any version it doesn't
  know. One-line change now, saves a migration later.
