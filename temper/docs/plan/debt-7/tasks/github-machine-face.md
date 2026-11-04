+++
id = "github-machine-face"
part = "fake-github"
kind = "migration"
needs = ["github-fake-surface"]
files = ["skein/tests/forge/machine/src/github.rs", "skein/tests/forge/machine/src/joint/github.rs", "skein/tests/forge/machine/tests/suite/github_peer.rs"]
effort = "high"
minutes = 100
+++

# fake-github: the machine world's face drives the fake's server

## Goal

`skein/tests/forge/machine/src/github.rs` keeps its public `Fake`, its readers' surface, and loses what makes it a
face: `parse`, `response_head`, `imf`, `finish`'s `Outcome` loop, `domain_reply`, and its App, account, repository
and `Limits` tables. `serve`, `blob` and `page` go through `inline::Inline`. Every machine and setup test answers as
at base. This is the widest face: the setup world and temper's forge world read it.

Acceptance:

- `skein/tests/forge/machine/src/github.rs` has no `Outcome`, `fn parse`, `fn response_head`, `fn imf`,
  `fn domain_reply`, `fn fixture_accounts`, `fn fixture_repositories`, `fn fixture_app`, `Limits {` or `Config {` but
  in a struct update of `fixture::config`.
- `Fake`'s public methods and `WALL_BASE`, `BLOB_ADDRESS`, `BLOB_ORIGIN`, `app_key` and `bearer` keep their
  signatures; no file of `skein/tests/forge/setup` or `temper/tests/engine/protocol-forge` changes.
- The test files change only as `temper/docs/plan/debt-7/notes/fake-github.md`, 4 lists.
- The record run (the note, 3) gives two equal lists, base and branch, over both packages.
- Net lines at most -300.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`; `temper/docs/plan/debt-7/notes/fake-github.md`, whole.
- `skein/docs/design/fake-forge.md`, 10.6.
- `skein/docs/development/debt/D029-fake-github-http-faces.md`.
- Readers: `skein/tests/forge/machine/src/machine.rs` (`respond`, the `Fake` field, `github_fake_mut`),
  `skein/tests/forge/machine/src/fixtures.rs` (`github_access`), `skein/tests/forge/machine/src/memory.rs`
  (`app_key`), `skein/tests/forge/setup/src/host.rs` and `memory.rs` (`Fake`, `WALL_BASE`, `app_key`),
  `skein/tests/forge/setup/tests/suite/github_convert.rs`, `github_reconcile.rs` (`Fake`, `bearer`),
  `skein/tests/forge/machine/tests/suite/github_*.rs`.

## Owns and may change

- Owns: `skein/tests/forge/machine/src/github.rs`.
- `skein/tests/forge/machine/src/joint/github.rs`: `crate::github::fixture_app()` becomes
  `skein_fake_forge_github::fixture::app()`, that one line. `Fake::into_service` stays until `github-joint-face`:
  it returns `self.inline.service`.
- `skein/tests/forge/machine/tests/suite/github_peer.rs`: the `Host` line of the note's section 4, nothing else.
- **`Fake::page`** keeps its contract: `assert!` the request has no `Authorization` line, then `Inline::serve`. It
  records no status, as at base; `serve` and `blob` do.
- **`Fake::statuses`:** the three digits at bytes 9 to 12 of the response.
- **Not yours:** the `fixture_domain` family's content. It takes `fixture::setup` by struct update and stays.
- **`Time`:** `Inline` runs admission on the same clock as calls, where the base's `finish` ran it on
  `wall - WALL_BASE`. A window's `reset` is its end's distance from `wall`, so nothing a test reads moves. Don't
  keep the offset. On a wall below `WALL_BASE`, as `github_peer`'s, the base's clock saturated at zero: a difference
  in `X-RateLimit-Reset` alone there, in the record run, is that. Report it; it is no stop.
- If `Server` refuses a request a test's head carries (a limit of `fixture::limits()`), stop and ask.

## Steps

1. **The record run's base half,** before any change: the note, 3. Keep its lines.
2. `struct Fake { inline: Inline, statuses: Vec<u16> }`. `configured` builds `Config { blob: Box::from(BLOB_ORIGIN),
   third_origin, blob_redirect, app: App { permissions, events, ..fixture::app() }, app_facts: facts, hook:
   Box::from(hook), ..fixture::config(fixture::repositories()) }`, `fixture::limits()`, `fixture::install_three`,
   `fixture_domain(seed, has_logs)`, `Inline::new`.
3. The scenario methods (`installations_mut`, `release`, `redeliver`, `revoke_key`, `installation_ids`,
   `blob_records`, `manifest`) read `self.inline.service`.
4. `serve` and `blob`: `Inline`, then record the status, then `into_vec`. `page`: above.
5. `fixture_domain`: `a::Setup { default, checks, protection, ..fixture::setup(id, name) }`; `fixture_call` through
   `inline::call`, at `Wall::EPOCH` as at base.
6. `app_key` is `fixture::key()`; `bearer` stays. Delete everything the acceptance names.
7. **The record run's branch half,** with the same helper. Diff the lists, then remove the helper.
8. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge-machine-world
   --all-targets -- -D warnings`. This changes what the fake's tests meet, so its readers run
   whole: `heavy cargo nextest run -p skein-forge-machine-world -p skein-forge-setup-world -p
   skein-forge-protocol-world -p temper-engine-protocol-forge-world`, and the seeded ones, `heavy cargo nextest run
   --profile fuzzy -p skein-forge-setup-world`, whose `fuzzy::github_setup` drives this face. Citations. Commit and
   push: the record run leaves no trace in the commits.

## Report

The common note's items; the lines compared in the record run; the packages' nextest totals, base and branch; net
lines. No new test, so no break.

## Answers

2026-11-04. Who repairs `Inline` and the lints? **`github-fake-surface`, on its next run; see its `## Answers`.**
This task's files don't widen. Keep your pushed commit and your files. At the start, and before you push, `git fetch origin` and merge
`origin/debt-7/github-fake-surface`. If that branch still has `Writer` in `src/inline.rs` or has no
`Installations::minted_token`, its repair hasn't arrived: report that and stop, changing nothing outside your files.
Then run what the first run couldn't: the record run's branch half
against the 605 lines of the run's `base-records.txt`, the readers, the fuzzy run, clippy, and the totals.

**Size.** The cap was set by the cut; the push measures -306, 24 short of -330. Compact what can go in the lines this
task owns, keeping `Fake`'s public surface and the `fixture_domain` family, which the spec keeps. Why the cap can't be
counted on: nearly all of the 320 lines left are those two. So it eases to **-300**, under the 25% the plan's owner
allows. Short of that, stop and ask, with the measurements.
