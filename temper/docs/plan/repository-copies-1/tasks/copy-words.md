+++
id = "copy-words"
part = "copy-plan"
kind = "feature"
needs = ["copy-surface"]
files = ["skein/crates/skein-forge-setup/src/host/connection.rs", "skein/crates/skein-forge-setup/src/host.rs", "skein/crates/skein-forge-setup/src/tests/host.rs"]
effort = "high"
minutes = 45
+++

# A copy's words and its deadline

## Goal

A `Migrate` Forgejo refuses says why, and a `Copy` past its deadline stops `TimedOut`, in the host
(`skein/docs/design/forge-setup.md`, 4.10 and 6.3).

Acceptance, in `skein/crates/skein-forge-setup/src/tests/host.rs`, each driven through HTTP bytes as
that file's others are:

- `a_refused_migrate_says_forgejos_message_before_its_stop`: a 422 with `{"message":"..."}`: the events
  up are `Event::Said { run, words }`, then `Event::Stop` at `Repository`, `Refused(422)`.
- `a_refused_migrate_with_no_message_says_nothing`: an empty body, a non-JSON body: the `Stop` alone.
- `a_refused_migrates_words_are_cut_at_a_character`: a message past `string_bytes`, its last character
  several bytes long.
- `a_401_says_nothing`: its body is not read.
- `no_other_refusal_says_anything`: a `CreateRepository` answered 422 with a message.
- `a_copy_past_its_deadline_stops_timed_out`: a `Migrate` unanswered at `copy_deadline`: `Stop` at
  `Repository`, `TimedOut`, the connection aborted.
- `a_removed_forges_copy_stops_down`: `Remove` during a `Migrate` keeps `Down`, as `Abort` does.
- The tests that pin `Down` for other plans at their deadline still pass, unchanged.
- `heavy cargo nextest run -p skein-forge-setup -p skein-forge-setup-world` pass.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, S2.
- `skein/docs/design/forge-setup.md`, 4.10, 6.3 ("A forge's words"), 6.5.

## Owns and may change

- The files of the header; `skein/crates/skein-forge-setup/src/host/run.rs`, what `cut` needs of a run.
  `copy-machine` runs beside this on `copy.rs`, `create.rs` and `run.rs`'s `Machine` arms: stay off
  those.
- **The body:** `skein_forge::forgejo::client::error_decoder` reads it, as
  `skein/crates/skein-forge-connection/src/connection.rs` does. No new document.
- **`cut`:** today it steps `Reply::Down`, so the machine stops `Down` and a later `halt` is a no-op.
  Give `cut` its why, or split it: the deadline's call passes `TimedOut` for a `Copy` and `Down` for
  every other plan; `Abort` and `Remove` pass `Down`.
- Pre-approved:
  - `Event::Said` goes up as the connection's result is taken, so it precedes the `Stop` in the same
    `up` queue; if the entry points' bound of 4 events is pinned by a test, it stays within it.
  - A malformed body, or one with no `message`, gives `Reply::Status(status)` with no words, never
    `Reply::Down`.

## Steps

1. The connection keeps its status while the error decoder runs, for a `Migrate`, and hands back words
beside the status. 2. `host.rs` raises `Event::Said`. 3. `cut`'s why. 4. The tests, each biting: read a
401's body, and pass `Down` from the deadline, each once. 5. The implementer's checks; commit and push.

## Report

As the implementer's text asks.
