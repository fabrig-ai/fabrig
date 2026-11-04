# D016: Small primitives are retyped beside skein's

- **Kind:** primitives
- **Where:** `smith-domain-session`, `smith-domain-tools`, `smith-protocol-llm`, `smith-agent-service/src/service/stages.rs`, `smith-inline-agent/src/domain.rs`, `smith-protocol-program/src/mcp/json.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in skein's review (`skein/docs/development/debt/reviews/2026-10-31-skein-areas.md`: the lib area, findings F1 and F2)
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 13, Q5)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (one spelling per idea; fewer impossible failure branches)
- **Size:** S
- **Status:** open
- **Paid when:** smith converts native lengths with `skein_lib::len_u64` wherever the conversion cannot fail, and has no private `earliest`, `earlier` or byte `join` beside skein's.
- **Measure:** `git ls-files 'smith/*.rs' | grep -v generated | xargs grep -oE 'u64::try_from\([a-z_.()]*len\(\)\)' | wc -l` is 90

## What

- **Lengths.** The quality run adopted `len_u64` at 120 sites. 90
  `u64::try_from(x.len())` remain, mostly as `match` or `.ok()?` that handle
  a failure no supported target has: 9 in `session/compaction.rs`, 6 in the
  session's `llm.rs`, 5 each in `smith-protocol-llm/src/endpoints.rs`,
  `smith-domain-tools/src/kit.rs`, `session/accounting.rs` and the
  session's `record.rs`.
- **Earliest deadline.** `earliest(&[Option<Time>])` in
  `smith-agent-service/src/service/stages.rs:64` and `earlier` in
  `smith-inline-agent/src/domain.rs:180`, beside
  `skein_lib::time::earliest` (`skein/crates/skein-lib/src/time.rs`).
- **Concatenation.** `join(parts: &[&[u8]]) -> Box<[u8]>` in
  `smith-protocol-program/src/mcp/json.rs:105`; skein's review counts 27
  such copies across the trees and none in lib.

## Why it matters

Each hand form is a branch to read and test that cannot be taken, or a
second spelling of something skein has. skein's review found the same lag
in jig and temper.

## What pays it back

Mechanical: `len_u64` where the conversion cannot fail, keeping checked
arithmetic around it; skein's `earliest` folded over the list. The `join`
goes when skein grows a concatenation helper: that part waits on skein's
`skein/docs/development/debt/D007-joined-and-measure.md`.
