# TODO — p2p/fractal

Findings from the review of `714634c8` (`feat(p2p/example): add --clone to
fractal_swarm_perfect_ip`), in severity order.

## 1. Empty-placeholder slice is a correctness bug (ship-blocker)

`run_fractal_engine` answers a missing slice with
`ProtocolSlice { data: Vec::new(), is_parity: false }`. `Header` carries no
length field, so a receiver cannot distinguish "peer doesn't have it" from
"a zero-length leaf" — a placeholder counted as received silently corrupts
`reconstruct_payload`. Fix: respond with an explicit error/None, or add the
payload length to `Header` and treat length-0 data as absent.

## 2. Wire encoding: JSON → binary

`request_response::json::Behaviour` costs ~2×+ wire overhead plus parse
cost on every ~730-byte slice. Keep JSON as a debug/inspection mode; make
the production codec binary (or raw-leaf CID frames — see #3).

## 3. Per-slice authentication (makes repair trustless)

Slices are trusted by manifest membership; `verify_integrity` only catches
bad parities when siblings are present. `sha256_hex` already exists in the
module — put per-slice hashes in the manifest so every slice is
self-verifying. Better: make each slice a raw-leaf CID, so a serving peer
proves content by address and any gateway can backfill a missing slice.

## 4. One-loss-per-pair ceiling

XOR-tree recovers exactly one missing sibling per pair; a pair losing
children *and* parity is unrecoverable. Document the ceiling in the
protocol docs, or evaluate a real erasure code (Reed-Solomon / RaptorQ)
for lossy links. Fine as a repair-assist layer either way.

## 5. `reconstruct_payload` invariant assertion

It sorts by `seq_num` and concatenates, relying on "seq order == byte
order across the recursive tree." True today, implicit forever — assert
it (or key the manifest by (offset, len) instead of tree ids).

## 6. `--clone` verification: add `git fsck --strict`

The current check (clone → HEAD match → clean tree) proves the
reconstruction is a valid repo. Add `git fsck --strict` on the
reconstructed directory to also catch object-store corruption the clone
might tolerate.

## Later / integration

- Repair-by-id is the natural loss path for a git transport: primary =
  stream, repair = slice requests by id. Keep the protocol id stable.
- `IntegrityManager::persist` format: version it before other tools
  depend on it.
