+++
id = "link-told"
part = "link-told"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-client-domain/src/watches.rs",
  "jig/crates/jig-client-domain/src/link.rs",
  "jig/crates/jig-client-domain/src/tests/watches.rs",
  "jig/crates/jig-client/src/client.rs",
  "jig/crates/jig-client/src/tests.rs",
  "jig/docs/design/client/domain/client.status.md",
]
effort = "medium"
minutes = 30
+++

# The link names a watch refused busy

## Goal

A client whose watch is refused Busy can't say so: `ended` in `jig-client-domain/src/watches.rs` takes Busy as retryable, marks the watch Behind and, for `me`, takes the link Offline, and nothing records why. `ctl-told` needs the fact at its deadline, as it has `streams_full`. This task adds that fact to jig and nothing else.

Acceptance:

- `Domain::watches_full()` and `LinkView::watches_full` exist as `temper/docs/plan/watch-room/notes/contracts.md` has them.
- `watches_full` is true from a watch's Busy end until that watch is next answered (a delivery or `Opened`), through its backoff and its reopening.
- It covers `me` and an application's watch alike. An application's watch refused Busy sets it with the link still Live.
- It is false after any other end of that watch (`Dropped`, `Stopping`), after the watch is closed or ended, and after sign-out. A closing watch never counts.
- No `Told` is added or changed, and the link's states go as they do now. No wire field is added.

## Inputs

- `jig/docs/design/client/domain/client.md`, 8.1 ("Dropped is not ended") and 8.2 (the paragraph that names `streams_full` and `watches_full`).
- `temper/docs/plan/watch-room/notes/contracts.md`, "jig's client: a watch refused busy".
- `temper/docs/plan/watch-room/notes/common.md`.

## Owns and may change

Owns the files in the header. No other draft in flight changes them but `ui-pgup/live-kept`, which adds its own lines to `client.status.md`; delete only your line.

Reader and neighbour files: none need changing. `LinkView` is built only in `jig-client/src/client.rs`, `link`, and its readers read fields by name, so a new field breaks nothing. The `Watch` struct is `pub(crate)`: its literals are in `jig-client-domain`, and each takes the new field as a compiler-forced edit.

Departure from your text: delete the one status line named in step 4.

Options, each pre-approved:

- **How it's kept.** Default: a `busy: bool` on `Watch`, set in `ended` from the end, cleared in `opened`, and `watches_full` scanning `self.watches` for a busy watch that isn't closing, as `party` scans them. Reading it on demand needs no clearing elsewhere. If `set_link`'s way (`streams_full`, an `Option` on the domain) proves simpler and meets every criterion, take it and say so.
- **No `matches!` or `_`** in step code. If `WatchEnd` has no `PartialEq`, list its variants, as `ended`'s `why` does.

## Steps

1. In `jig-client-domain/src/tests/watches.rs`, beside the `streams_full` test and using the same `Client::started()`, `c.end`, `c.at(next_deadline(..))`, `c.downs()` and `c.name(..)`, write the failing tests:
   - `a_me_refused_busy_is_named_until_it_is_answered`: `me` ends Busy, so `watches_full` is true and the link Offline. It stays true past the backoff and through the reopening. The next delivery makes it false and the link Live.
   - `an_applications_watch_refused_busy_is_named_with_the_link_live`: the same on a page's watch. The link stays Live throughout.
   - `a_watch_answered_after_busy_is_no_longer_named`: Busy, reopened, and `WatchEvent::Opened` with no delivery, so false.
   - `a_drop_after_busy_is_not_named`: Busy, reopened, `Dropped`, so false.
   - `a_closed_watch_refused_busy_is_not_named`: Busy on a watch being closed, and on sign-out, so false.
2. Make them pass, and add `Domain::watches_full`.
3. Add `LinkView::watches_full` and `link()`'s line. In `jig-client/src/tests.rs`, beside its `streams_full` test, write `the_link_view_names_a_watch_refused_busy`.
4. Delete the `Not yet` line that opens `8.2: watches_full` from `jig/docs/design/client/domain/client.status.md`.
5. Break the rule once by hand: don't clear in `opened`, see a test fail for it, and restore.
6. `cargo fmt`, `python3 -I -B jig/scripts/old_forms.py`, `heavy cargo clippy -p jig-client-domain -p jig-client --all-targets -- -D warnings`, `heavy cargo nextest run -p jig-client-domain -p jig-client`. The change is additive, so its dependents take a check: `heavy cargo check -p jig-client-protocol-link -p temper-client-tools -p temper-client-shell-cli -p temper-client-domain --tests`. Run `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

The implementer's report, plus which way you kept the fact.
