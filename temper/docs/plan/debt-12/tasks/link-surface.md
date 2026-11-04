+++
id = "link-surface"
part = "link"
kind = "interface"
needs = []
files = ["skein/crates/skein-link/**", "skein/tests/link/tests/suite/plain.rs", "skein/tests/link/tests/suite/close.rs", "skein/docs/design/link.status.md"]
effort = "high"
minutes = 40
+++

# The link keeps bytes out of `Debug`, and adds `due` and `Upload::of`

## Goal

`Upload` holds the OAuth token POST body and `Link` queues it, and both derive `Debug` over those bytes, where
`skein/docs/design/oauth.md`, 4 says such bytes stay out of `Debug`. Hand-write both. Add the two additions the
owners' tasks build on. This is the surface only: `link-ending` reshapes the state.

Acceptance:

- `format!("{:?}", ...)` of an `Upload` and of a `Link` shows no byte of a body or a queue, and tests pin it.
- `Link::due` and `Upload::of` exist as `temper/docs/plan/debt-12/notes/contracts.md`, 1 says, with tests.
- `heavy cargo nextest run -p skein-link -p skein-link-world` passes.
- `heavy cargo clippy -p skein-link -p skein-link-world --all-targets -- -D warnings` passes.
- `skein/docs/design/link.status.md` no longer lists what this task built.
- Net production lines: about +30 at most.

## Inputs

- `temper/docs/plan/debt-12/notes/common.md`
- `temper/docs/plan/debt-12/notes/contracts.md`, 1 and 3
- `temper/docs/plan/debt-12/inputs/link-review.md`, finding 2
- `skein/docs/design/link.md`, 3.1 and 3.4
- `skein/docs/design/oauth.md`, 4

## Owns and may change

- **Owns:** `skein/crates/skein-link/` and, for this wave, the link world's `tests/suite/plain.rs` and
  `tests/suite/close.rs`.
- **May change:** `skein/docs/design/link.status.md`, step 5 only. This departs from the implementer's text, for
  that file.
- **Not yours:** the owners' crates, `Ending`, `held`, `pump`, `close`, `Link::socket`: `link-ending` has them.
- **Options:**
  - `Debug` is derived by types that hold a `Link` or an `Upload` (the forge connection's `Connection`, the link
    world's `World`). They compile against the hand-written impl and change nowhere.
  - If `Option::map` is refused, as skein's `clippy.toml` says, `Upload::of` is a `match` with one
    `#[expect(clippy::manual_map, reason = "...")]`.
  - If a test can't name `Env<Limits>` in `src/tests.rs`, it goes in the link world, which has `env()`.

## Steps

1. `Upload`'s and `Link`'s `Debug`, by hand, with `core::fmt`, in the face of a derive, and lengths in place of
   bytes:

   ```rust
   // Upload { bytes: 5, at: 0 }
   // Link { socket: None, encrypted: false, events: 1, cipher: 1, ending: Open }
   ```

   `tls` is shown only as `encrypted`. `Ending` keeps its derive.
2. `Link::due` beside `next_deadline`, a `match self.ending` that lists every variant; `Upload::of` beside
   `new`. Export nothing else from `lib.rs`.
3. Tests, with `#[expect(clippy::use_debug, reason = "...")]` where the lint asks:

   - In `skein-link/src/tests.rs`, `an_upload_shows_its_length_and_no_byte`, and `upload_of_keeps_a_body_and_no_body`.
   - In `plain.rs`, `a_link_shows_its_queues_lengths_and_no_byte`: a plain `Link` with a `Send` and an
     `Up::Bytes` queued, both of the canary `b"secret-token"`.
   - In `close.rs`, `due_says_the_close_deadline_has_come`: false while open; false a nanosecond before
     `DEADLINE` with the room held; true at it; false once `fire` has aborted.

   **Compare the whole string.** The derive prints bytes as decimals (`[115, 101, ...]`), so a search for
   `"secret"` passes against the code this task replaces. `assert_eq!` on the whole `Debug` text, as in step 1.
4. Break each by hand once: put the `#[derive(Debug)]` back on each type (the two canaries), and `>=` to `>` in
   `due`.
5. `skein/docs/design/link.status.md`: from **Not yet**, delete `Link::due`, `Link`'s `Debug`, `Upload::of` and
   `Upload`'s `Debug`, and keep the rest word for word.
6. Fmt, clippy, tests, citations. Commit and push.

## Report

The report of your implementer text, and: what you broke and what failed; net production lines.
