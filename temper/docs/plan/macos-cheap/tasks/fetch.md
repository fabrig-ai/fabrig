+++
id = "fetch"
part = "fetch"
kind = "feature"
needs = ["pin-move"]
files = ["temper/crates/temper-forge-setup/src/fetch.rs", "temper/crates/temper-forge-setup/src/tests/fetch.rs", "temper/crates/temper-forge-setup-shell/src/fetch.rs", "temper/crates/temper-forge-setup-shell/src/process.rs", "temper/crates/temper-forge-setup-shell/tests/suite/fetch.rs", "temper/crates/temper/src/forge.rs"]
effort = "high"
minutes = 110
+++

# `temper forge fetch` on macOS: Homebrew's bottles, as a URL

## Goal

On macOS, `temper forge fetch` takes each binary out of its pinned bottle on `ghcr.io`: the anonymous token to `ghcr.io` alone, one redirect, the archive checked by its digest, its one member taken out by `/usr/bin/tar`, checked by the binary's digest, renamed into the cache. Linux's fetch does what it does today, byte for byte.

Acceptance:

- the machine's table test covers macOS's pins, each failure of `temper/docs/design/protocol/setup.md`, 6.6, and the redirect's rules, and Linux's existing cases pass with their `Request::Get` carrying `authorization: None`;
- a test in `temper-forge-setup-shell` runs a bottle end to end on Linux: a local TLS peer that answers `307` to a second local TLS peer that serves a `.tar.gz` the test made, with `/usr/bin/tar`, into a real cache directory: the token on the first request alone, the binary `0o755` at its name, no `.archive` and no `.part` left;
- the same test's refusals leave nothing under the binary's name, and neither side file;
- both crates reach rustls, so their Darwin compile is `habits`' and the workflow's; this task's code holds nothing that compiles on Linux alone.

## Inputs

- `temper/docs/design/protocol/setup.md`, 6.1 and 6.6.
- `skein/docs/design/forge.md`, 4.3.
- `temper/docs/plan/macos-cheap/notes/common.md`; `temper/docs/plan/macos-cheap/notes/contracts.md`, "The pin" and "The fetch".

## Owns and may change

Owns `temper-forge-setup/src/fetch.rs` and its tests, `temper-forge-setup-shell/src/fetch.rs` and `tests/suite/fetch.rs`; in `temper-forge-setup-shell/src/process.rs`, the extraction alone (see Answers).

Readers: `temper/crates/temper/src/forge.rs`, its `fetch`: the `platform` argument alone. `temper/crates/temper/tests/support/fetch.rs` and `tests/suite/forge.rs`: only if `Event` or `Request` reach them.

Pre-approved:

- If `temper-forge-setup-shell/clippy.toml` disallows `std::process::Command`, take a scoped `#[expect(clippy::disallowed_methods, reason = "...")]` on the extraction, as `process.rs` and the suite do. (It does not disallow it today.)
- The redirect's `Location` is parsed in the machine: `https://HOST[:PORT]/PATH...`, the rest of it kept whole as the path, query among it, using what `machine.rs`'s `origin` already does if that fits. If the path is empty or the scheme is anything else: `Failure::Redirect`.
- A status other than `200` or a redirect on a bottle is `Failure::Gone`, whose words name the URL and the status and end `: give the binary by path, or a mirror`. Statuses `301`, `302`, `303`, `307` and `308` with a `location` are redirects. A redirect answered to a Linux pin stays `Failure::Status`, as today.
- A file left by an interrupted fetch at `<file>.archive` or `<file>.part` is written again from its start, as `open` does.

Nothing else. A step crate holds no `cfg(target_os)`.

## Steps

1. Machine, per `notes/contracts.md`: `pins` gains `platform`; for `Macos` it refuses an architecture other than `arm64` with `Failure::Architecture`, keeps the origin `ghcr.io:443` or the mirror's (no token), the path `/v2/homebrew/core/<forgejo|forgejo-runner>/blobs/sha256:<archive digest>`, the member `<name>/<version>/bin/<name>`, and the file as on Linux. `digest` in the machine keeps its parsing, given `Platform` and `Form`.
2. Phases: `Head` takes a redirect once per pin; a second is `Failure::Redirect`. The body is hashed against the bottle's digest when there is one, and against the pin's otherwise. `Extract` carries the same bound.
3. Host: `Get` sends `authorization` as a header when there is one. `Answered` carries the response's `location` (`Response`'s lookup by name). `Extract`: closes the archive's file, opens `part` as `open` does, and calls `process::extract` (`temper-forge-setup-shell/src/process.rs`), which runs `/usr/bin/tar -xzOf ARCHIVE MEMBER` with a cleared environment, stdin null, stdout piped, stderr null, and reads it `CHUNK` at a time, hashing and writing, counting against `bound`. Past the bound: kill and reap tar, `Extraction::Bytes`. A failed status or an empty output: `Extraction::Failed`. Keep the part file's handle for `Finish`.
4. `crates/temper/src/forge.rs`: `Platform::Macos` when `cfg!(target_os = "macos")`, `Platform::Linux` otherwise.
5. Tests as in the acceptance. Two peers from the suite's `server`; the tarball by `/usr/bin/tar -czf` in the test's scratch, its digests taken in the test, so the `Pin`s are the test's own, not `pins`'.
6. `cargo fmt`; the old-forms counts; clippy of the three packages; `heavy cargo nextest run -p temper-forge-setup -p temper-forge-setup-shell -p temper`. Commit and push.

## Report

As the implementer's text has it, with how the tar invocation behaves on GNU tar, and anything in it that would differ on bsdtar.

## Answers

2026-11-04. Q: the boundary test `process_and_file_locks_stay_in_their_declared_modules` rejects `std::process` in `src/fetch.rs`: widen `boundary.rs`, or move extraction into `process.rs`?

Move it into `process.rs`. `boundary.rs` stays as it is.

- `temper/docs/design/protocol/setup.md`, 6.4 makes `process.rs` the one place temper uses `std::process` outside skein, and the guard holds that; nothing in 6.6 asks for a second place. `sign_in_key` there is the precedent: a blocking child writing to a file its caller opened.
- Move `extract` from the pushed `src/fetch.rs` to `src/process.rs` as it stands, public with its `Extraction` answer: `pub fn extract(archive: &[u8], member: &[u8], file: &mut fs::File, bound: u64) -> Extraction`, with its read buffer as large as `fetch`'s `CHUNK`. The `Request::Extract` arm calls it, and `src/fetch.rs` keeps no `std::process`. Nothing else in `process.rs` changes, except that its first line may name the extraction too.
- Then run the step 6 run to its end: the 1,690 tests the first run left unobserved are this task's to observe, and the bite checks need no repeat beyond the moved function's own test still passing.
