+++
id = "transcript-host"
part = "names"
kind = "feature"
needs = []
files = [
  "skein/tests/http/transcripts/forgejo-repository-captured.http",
  "skein/tests/http/transcripts/forgejo-repository-captured.expect",
]
effort = "medium"
minutes = 15
+++

# The captured Forgejo transcript takes the reserved host

## Goal

`skein/tests/http/transcripts/forgejo-repository-captured.{http,expect}` name the private forge.
They take `git.example.org`, as the sibling `forgejo-issues-page-chunked` has it, and the tests read
them on equal terms.

Acceptance:

- Neither file holds `ekanayaka`.
- The `.http`'s chunk size is `979`: its 9 uses of the host each lose a byte, so `982` (2434 bytes)
  becomes 2425. Line endings stay CRLF; every other byte stays.
- The `.expect`'s body says what the `.http` gives; its header comment says what is true now.
- `skein-http-world`'s tests pass in `default` and `fuzzy`, with no expectation, limit, seed or
  count relaxed, and the same tests run.

## Inputs

- `skein/tests/http/src/transcript.rs`: the format; there is no size or digest kept anywhere.
- `skein/tests/http/transcripts/forgejo-issues-page-chunked.http`: the sibling.
- `hearth/docs/plan/fail-stop-names/notes/common.md`.

## Owns and may change

- Owns the two files, nothing else. The rest of the JSON stays, `ai@noreply.localhost` and the
  avatar's digest among it. The other transcripts' `free@localhost` is left alone (the common note).
- Readers: every transcript is read by `transcript::all()`, from
  `skein/tests/http/tests/suite/transcripts.rs`, `stack.rs`, `fuzzy/client.rs` and `fuzzy/events.rs`.
  The fuzzy test draws its settings from the bytes' length, so a shorter transcript draws other
  settings. If `transcripts_cut_mutated_and_failed_read_as_the_reference_reads_them`'s tally then
  fails, that's a question with the failure's log: never change its seeds or rounds.

## Steps

1. Replace the host in both files byte for byte, the `ssh://git@...:10128` URLs included, and set
   the chunk size line to `979`. Check the arithmetic by reading the file back: the chunk's length
   from the size line to its CRLF is 2425.
2. The `.expect`'s comment holds the curl command with the new host, and "bytes as they came" stops
   being true, so:

   ```text
   # Captured from the user's own forge, an unauthenticated read of a public
   # repository: curl --http1.1 --raw -i https://git.example.org/api/v1/repos/ai/temper
   # on 2026-10-04, through its nginx, bytes as they came but for the forge's host,
   # written git.example.org here, and the chunk size that follows from it. A Forgejo
   # repository, chunked in one chunk, its JSON ended by an LF.
   ```

3. `heavy cargo nextest run -p skein-http-world`, then `heavy cargo nextest run -p skein-http-world
   --profile fuzzy`, and compare the counts with what the package ran before your change.
4. `cargo fmt` has nothing to do here; skein's `old_forms.py`; the common note's checks. Commit and
   push.

No new test: the change is a fixture's bytes, read by the tests above.

## Report

The commit, each profile's count, and that no test was relaxed.
