# D004: The reference root is half jig wiring, and the test root copies it

- **Kind:** coupling
- **Where:** `examples/ops/domain/src`, `testing/jig-test-domain/src/lib.rs`, `testing/jig-test-domain/src/routes.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T4, T2, A12 F1, F2, F4, A11 F3, F4, F5; order of work, item 11)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 11, "roots that decide")
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** high (`ops` becomes a template an application can copy)
- **Size:** L
- **Status:** open
- **Paid when:** `ops`'s root holds only `ops`'s own routing and translation; the hub↔fleet relay, payload tokens, charter assembly and timers it shares with the test root live once in jig, and both roots use them.
- **Measure:** `find jig/examples/ops/domain/src -name '*.rs' -not -name 'tests*.rs' -not -path '*/tests/*' | xargs cat | wc -l` (3,836 at base)

## What

- `ops`'s domain is 3,836 non-test lines (4,011 before the quality run; the review aimed at about 2,000); `route.rs` alone is 1,446.
- It keeps its own classifier of host events (`admission.rs`, `Class`), decodes delegate JSON and assembles alert words; `jig/docs/design/examples.status.md` and `jig/docs/design/domain/root.status.md` record this as a departure.
- The test application's root has another shape: `lib.rs` 1,370 lines against `ops`'s 38, plus `routes.rs` 1,310, with the same `Now` arms (D001).

## Why it matters

`jig/docs/design/README.md`, 6.3 and 6.7 make `ops` the reference root, one shape for
every root. A root that is half jig wiring cannot be copied, and each copy drifts.

## What pays it back

After D001: a jig "engine link" part owning payloads, host calls, assignment, brief and
turn backoff maps; a link or core classifier for host events; the test root rebuilt in
`ops`'s shape. The status files' departures close with it.
