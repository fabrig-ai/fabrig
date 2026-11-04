+++
id = "forgejo-path-segments"
part = "forgejo-paths"
kind = "feature"
needs = ["walker-depth"]
files = ["skein/crates/skein-forge/src/forgejo/path.rs", "skein/crates/skein-forge/src/forgejo/tests.rs", "skein/tests/forge/protocol/tests/suite/protocol.rs"]
effort = "medium"
minutes = 20
+++

# The fake Forgejo reads a deep path

## Goal

`skein-forge`'s `forgejo/path.rs`, `read`, holds a path in `List::with_capacity(12)`, so a target past 12
segments is `Unread::TooLong`, which every router maps to `Malformed` and the fake answers 422. A file
seven components deep is refused. Only `forgejo/server/` reads targets this way, so this is the
fake Forgejo's fault, not a client's. (Bug 11, re-checked: holds.)

Acceptance:

- `read` sizes its segments by `limits.target_bytes`: each segment costs a `/` and the target's length is
  checked first, so it stays bounded, and `List` allocates only as it fills.
- `Unread::TooLong` stays for a target's length and its query. The query's cap of 8 pairs is as it is.
- The tests below fail on the old code.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/forge.md`, 4.1 (`target_bytes`), 5.1 ("A file's path keeps its slashes").

## Owns and may change

- Owns: `src/forgejo/path.rs`, `src/forgejo/tests.rs`, `skein/tests/forge/protocol/tests/suite/protocol.rs`.
- No reader changes: `path::read`'s callers under `forgejo/server/` keep it, and no test asserts the cap.

## Steps

1. In `src/forgejo/tests.rs`, modelled on `raw_file_targets_keep_slashes_and_encode_each_other_byte`: a
   `Request::Raw` with path `a/b/c/d/e/f/g.md` through `client::encode` and `server::route` equals the
   request. In `protocol.rs`, modelled on `directories_and_the_root_read_through_both_http_fakes`:
   `World::new(5, 0)`, `world.tree` with a file seven components deep, `Read::File` is `Answer::File`. Run
   on the old code and quote the failures.
2. The one line.
3. `heavy cargo nextest run -p skein-forge --lib -E 'test(/forgejo/)'`, then the focused tests of
   `skein-forge` and `skein-forge-protocol-world`. `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`,
   `heavy cargo clippy -p skein-forge --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the tests showed on the old code.
