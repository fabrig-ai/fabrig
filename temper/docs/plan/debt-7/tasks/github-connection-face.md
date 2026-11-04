+++
id = "github-connection-face"
part = "fake-github"
kind = "migration"
needs = ["github-fake-surface"]
files = ["skein/crates/skein-forge-connection/src/tests/github_files/fake.rs"]
effort = "high"
minutes = 50
+++

# fake-github: the step harness's face drives the fake's server

## Goal

`skein-forge-connection/src/tests/github_files/fake.rs` stops parsing requests, encoding heads and looping over
`Outcome`: `Fake::answer` hands the wire's last request to `inline::Inline::serve`, and its tables come from
`fixture`. The contents tests answer as they do at base. One face of four in
`skein/docs/development/debt/D029-fake-github-http-faces.md`.

Acceptance:

- `rg -l 'Outcome::Admit' skein/crates/skein-forge-connection/src` prints nothing.
- `fake.rs` has no `fn encode`, no `service::Request`, no `service::Config {` and no `domain::Config {`, and is at
  most 140 lines.
- `src/tests/github_files.rs` is unchanged. The package's lib tests pass, with the count they have at base.
- Net lines at most -250.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`; `temper/docs/plan/debt-7/notes/fake-github.md`, 1 to 3 and 5.
- `skein/docs/design/fake-forge.md`, 10.6.
- `skein/docs/development/debt/D029-fake-github-http-faces.md`.
- `skein/crates/skein-forge-connection/src/tests/fixtures.rs`: `request_bytes` holds every request of the
  connection, so `answer` is handed more than the one to serve.

## Owns and may change

- Owns: `fake.rs`. Nothing else in the crate changes; `github_files.rs`'s calls (`Fake::new`, `with_files`,
  `answer`, `object`, the five fields) stay.
- **Values this face gives up, none of them read here:** the App's webhook, the App hook's URL and the signed-in
  person take the fixture's; the account table gains `alice`. No test of the file reads them; if one does, stop.
- **What stays exact:** repository id 1, `acme/widgets`, `Reach::All`, the token's scope, seed 5, `files` and
  `page_size` as `with_files` raises them, `content_bytes` 8192, `files` at least 4, `f::BODY_ROOM`,
  `crate::limits::documents(&f::limits())`, installations `{1, 2, 1}`, pages `{1, 2048}`.

## Steps

1. Replace `service` and `domain` with `inline: Inline`; keep `env` for `call`, or build it there.
2. `with_files`: `fixture::domain_config()`, then the three limits above; `fixture::setup(1, fixture::PATH)`.
   `github_service(page)` returns a `(Service, Limits)` from `fixture::config(fixture::widgets(1))` and
   `Limits { http: Limits { body: u64::from(f::BODY_ROOM), ..fixture::limits().http }, ..fixture::limits() }` with
   the fields above named; `fixture::install_one(&mut service, Wall::EPOCH)`; `mint` as at base.
3. `drive` and `call` go through `inline::call`.
4. `answer` keeps only what picks the last request: every request here is a GET with no body, so it is the bytes
   after the last `\r\n\r\n` but one. `self.inline.serve(request, Wall::EPOCH)`.
5. Delete `encode`, `config` and the literal in `github_service`.
6. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge-connection
   --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-forge-connection`. Note the two peaks
   `largest_github_compare_page_stays_under_the_ceiling` and `largest_github_pull_files_page_stays_under_the_ceiling`
   print, at base and on the branch: a reply now carries a `Date`. A ceiling passed is a stop: ceilings are the
   test's, not yours. Citations. Commit and push.

## Report

The common note's items; the two peaks, base and branch; the package's nextest total, base and branch; net lines.
No new test, so no break.

## Answers

2026-11-04. Who repairs `Inline`'s response buffer? **`github-fake-surface`, on its next run, with the 17 clippy
errors in its crate; see its `## Answers`.** This task's files don't widen. Keep your pushed commit and your files. At the start, and before you push, `git fetch origin` and merge
`origin/debt-7/github-fake-surface`. If that branch still has `Writer` in `src/inline.rs` or has no
`Installations::minted_token`, its repair hasn't arrived: report that and stop, changing nothing outside your files.
Then run what the first
run couldn't: the whole package, clippy, and the two peaks. The output now grows in a `List`: a ceiling passed is still
a stop. Net lines stand at -258 against -250.
