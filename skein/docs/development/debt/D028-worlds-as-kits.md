# D028: World crates are used as kits by other test crates

- **Kind:** tests
- **Where:** `skein/tests/tls`, `skein/tests/llm`, `skein/tests/json`, `skein/tests/forge/machine`, `skein/tests/forge/client`, `skein/tests/http`, `skein/tests/echo`, `skein-codec/src/lib.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T5, worlds double as kits; phase 1 item 7) and `reviews/2026-10-31-skein-areas.md` (A0)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 6 and S4)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** no skein package depends on a world crate under `skein/tests/`; what a second crate needs from a world lives in a kit under `skein/testing/`; no `#[path]` reaches across a crate's root.
- **Measure:** `rg -c '^skein-(echo|forge-client|forge-machine|http|json|llm|tls)-world\b' skein -g Cargo.toml | awk -F: '{s+=$2} END{print s}'`: 16 at base

## What

- 16 manifest edges point at seven world crates: `skein-llm-world` (4),
  `skein-tls-world` (4), `skein-json-world` (3, one from the step crate
  `skein-llm`'s dev-dependencies), `skein-forge-machine-world` (2),
  `skein-forge-client-world`, `skein-http-world` and `skein-echo-world`.
- Two `#[path]`s cross a crate's root: `skein-codec/src/lib.rs` compiles
  `skein/tests/codec/src/generated/v1.rs` (see D034), and
  `skein/tests/fake-peers/tests/suite/github_memory.rs` compiles a file
  of `skein/tests/fake-forge`. The review counted 10; most are gone.
- `skein/docs/foundation/testing-strategy.md`, 7, moves a harness
  when a second user needs it, and `skein/docs/design/testing.md`, 6,
  puts skein's shared harnesses in `testing/`.

## Why it matters

A world's private pieces become another crate's interface, so a change
to one world breaks others, and test builds pull worlds they do not run.

## What pays it back

Move the shared pieces into `testing/` kits (TLS `pki`, `server` and
`drive`; the LLM fixtures; JSON `generate` and `reference`; the forge
machine world's GitHub fixtures; HTTP's `reference`), then a gate check
that no package depends on a `-world` crate under `skein/tests/` and no
`#[path]` crosses a crate. jig's world edges are jig's
`jig/docs/development/debt/D031-worlds-as-kits.md`.
