# D015: Flat forge request, answer and node enums are re-matched in every module

- **Kind:** complexity
- **Where:** `skein-forge/src/forgejo/request.rs`, `skein-forge/src/forgejo/client.rs`, `skein-forge/src/forgejo/server.rs`, `skein-forge/src/github/decode/walk.rs`, `skein-forge/src/github/request.rs`, `skein-fake-forge-forgejo/src/server.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T3) and `reviews/2026-10-31-skein-areas.md` (A1, A2 F1, A5 F1)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 3)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** L
- **Status:** open
- **Paid when:** Forgejo's `Request` and the server's `Answer`, and GitHub's `Node` and `Request`, nest per area or family and are matched once by their dispatcher; the modules below it match only their own family; the "exhaustive request selection" `too_many_lines` expects are gone.
- **Measure:** `rg -c 'unreachable!' skein/crates/skein-forge/src/forgejo -g '!**/tests/**' -g '!tests.rs' | awk -F: '{s+=$2} END{print s}'`: 93 at base

## What

- Forgejo's `Request` (`forgejo/request.rs`) has 73 variants; the
  client dispatches it to eight modules and each re-matches the whole
  enum, ending in `unreachable!`: 93 in Forgejo's non-test code, and 16
  `too_many_lines` expects under `forgejo/`.
- The fake Forgejo repeats its setup group of variants in each
  exhaustive match (`Request::DeleteRepoHook` appears 16 times in
  `skein-fake-forge-forgejo/src`).
- GitHub's decoder `Node` (`github/decode/walk.rs`) has 85 variants;
  `Node::` appears 765 times under `github/`, and one decoder holds every
  family's state.

## Why it matters

Each new route adds an arm to a dozen matches, most of them
`unreachable!`, and a reader cannot see which module owns a variant.
The lint that should break the build on a missed variant instead forces
the dead arms.

## What pays it back

Nest the enums: `Request::Setup(setup::Request)` and its siblings,
`Node::App(app::Node)` as `Protection` already is, and decoders and
plans as enums of per-family state. Mechanical, one family per task.
The neutral pieces sit at skein-forge's root (`skein/docs/design/forge/README.md`,
4.4).
