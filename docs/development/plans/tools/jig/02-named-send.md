# Session 02: jig's send under a named key

jig's client design (`docs/design/client/domain/client.md` 4.1 and 6, `contract.md` 3.1, landed at jig f50c3f63) lets a
`Send` carry a name: an engine key the application gives, admitted by the request table in place of a drawn one, and kept in
the record beside the application's key. While the table holds the name, a `Send` naming it is answered *held* at once, with
nothing kept or sent. Today `Send { key, request }` carries no name, `Domain::send` draws every engine key through skein-lib's
`RequestTable::ask`, and `restore` and `not_kept` know a refusal only as `Err` and a request only by its key. The work rests on
skein 01's landing (skein `lib.md` 12): given keys, typed refusals (`Full`, `Held`) and admission numbers. 02.1 repins skein onto
it and adapts jig-client-domain at the least change, behaviour unchanged; 02.2 adds the name and `Outcome::Held`; 02.3 adds the
joint stories. Nothing in jig names a send: the downstream application's one-shot mode (client.md 20), in the downstream
repository's tools plan, is the first. Three increments, in order; each lands alone.

## Read first

- **jig:** `AGENTS.md` and the gate, `scripts/check.sh`. `docs/design/client/domain/client.md`: 4.1 (`Send { key, name,
  request }`) and 4.2 (`Answered { key, outcome }`); 6, "The key is the application's name for a request", "A `Send` may carry a
  name" (its *Held*, *Never loud* and *Not joined*) and "One answer"; 11, "Kept before sent", "Restored at `Start { kept }`" and
  "It names its party"; 20, one-shot. `docs/design/client/domain/contract.md` 3.1 ("One key, one ask", "A name");
  `docs/design/README.md` 6.5 (mistakes are loud); `docs/design/client/domain/testing.md` 2 (the tiers) and 9 (budgets);
  `Cargo.toml` `[workspace.lints]`, `panic = "deny"`: a panic is `assert!`, `unreachable!("why")` or `expect("invariant")`.
- **skein:** `docs/design/lib.md` 12, "Keys", "Not kept" and "Admission and memory"; foundation `development.md` 5.7 (repins:
  "jig moves smith and skein together") and 3.3 (STATUS.md is the designer's); `programming-model.md` 6.3 (the worst case) and
  10.3 (what is out); `testing-strategy.md` 2.1 (step tests), 8 (two suites) and 10 (a test's shape).
- **Plan:** `skein/01-given-keys.md`, its "Interface sketches" (`RequestNumber`, `RequestAdmission`, `RequestRefused<T>`, `ask`,
  `ask_given`, `restore`, `not_kept(key, number)`); `findings/base-repin.md` (the update and check commands); the README's
  "Rules of this plan".
- **Code:** `crates/jig-client-domain/src/requests.rs` (`PendingRequest`; `Domain::send` with its assert, `restore_record`,
  `not_kept`, `flush`); `boundary.rs` (`Event::Send`, `Up::NotKept`, `Down::Save`, `Held`, `Outcome`); `domain.rs`
  (`Domain.table`, `step`); `crates/jig-client/src/client.rs` (`Event::Send`, `store_answer`, `drain`) and `store.rs`
  (`Store::save`, `erase`, `take`); `crates/jig-client-view/src/ui/client/outcome.rs` (`answered`);
  `tests/client-protocol/src/client.rs` (`Story`, `Front`).

## What the design asks

- **client.md 6, "A `Send` may carry a name":** the table admits the application's engine key in place of a drawn one; the
  record keeps it as the engine's key beside the application's; any ask may carry one, a family's included. 02.2 builds it. The
  record already holds both keys (`RequestRecord.key`, `Held.key`), so the store's format does not change.
- **client.md 6, *Held*:** a named `Send` whose engine key the table already holds (the same ask under another application key,
  another ask, or a restored record) is answered held at once, nothing kept or sent; the key stays held until its holder's record
  is erased, and the application asks again then. Within the retention the engine answers the same name and ask with its first
  answer (contract.md 3.1). 02.2 builds the client's half; 02.3 shows the engine's.
- **client.md 6, *Never loud*:** a held name is never the application's mistake. The same application key with the same request
  is still joined; with another request it is still a key conflict, loud. The assert stays, for that case alone.
- **client.md 6, "One answer":** *held* is the client's own outcome, beside not kept, unknown, unconfirmed and limit.
- **client.md 11:** a `Send` before `me` has named the party is answered unconfirmed at once, a named one too, so unconfirmed is
  told before held. `NotStored` retires what the write carried, by its key, and "a request whose record a later write carries
  again is decided by that later write".
- **lib.md 12:** `not_kept` names the key and the admission's number; `ask` and `restore` refuse `Full` or `Held`; a restore of
  a held key is refused `Held`. 02.1 adapts jig to all of it.
- **client.md 20, one-shot:** "a request sent by a run that stopped before its answer is sent again by the next run, under its
  key". 02.3's reload story shows it for a name; the mode itself, and its first host, are the downstream repository's tools
  plan's.

## Choices this plan makes

- **The admission's number stays in the domain, from admission to the not-kept report.** `PendingRequest` gains `admission:
  RequestAdmission`, as `ask`, `ask_given` or `restore` returns it; `Domain::not_kept(key)` finds the owner by `table.owner(key)`,
  and passes its `PendingRequest`'s number to `RequestTable::not_kept(key, number)`. `Up::NotKept { key }`, `Down::Save` and
  jig-client's store stay by key: a store's report names a key's last write (client.md 11, "decided by that later write";
  `Store::take`), and within one table's life that write carried the admission the domain holds under the key, since every
  admission but a restore leaves in a `Save`, and restores come only at `Start`, before any write. Carrying the number through
  `Down::Save`, `Up::NotKept` and the unlinked host would change three interfaces and no behaviour. Numbers live in memory only:
  `RequestRecord` carries none, and a table rebuilt at a restart numbers its restored records from 1 again (skein 01).
- **Joined, or a key conflict.** A `Send` under an application key the domain holds is joined when its request is the same and
  its name is `None` or the key it holds (`admission.key`); another name is another request, `assert!(.., "request key
  conflict")` as today. A next run that sends a restored request again under its key is joined whether or not it repeats the
  name.
- **The order of a send's answers:** joined or loud; then unconfirmed (client.md 11); then held; then limit (skein tells `Held`
  before `Full`).
- **`Outcome::Held` in the reference UI is `unreachable!`.** `jig-client-view` names no send (`intents.rs` sends `name: None`),
  and words for *held* would be design (view.md). `examples/ops` and `testing/jig-client-test-domain` name none either: their
  exhaustive matches give every client-own outcome `RequestProgress::Unknown`, and `Held` joins that arm.
- **No store or wire version.** `TABLE_VERSION` stays 2: a named record is a record whose engine key is the name, and the wire
  carries the engine key already. The link store's `STREAM_KEY` (`b"stream"`, 6 bytes) cannot meet a 16-byte name.
- **smith stays at jig 01's pin.** smith calls no `RequestTable` (`git grep RequestTable` at a277e922 finds none), so `cargo
  update -p skein-lib --precise` moves the one skein git source, for jig and for smith's crates alike (development.md 5.7).
- **The client's world's random applications send no names here;** the downstream application's worlds do, with its one-shot
  mode.

## Rules that bind it

- **Naming.** jig names no repository downstream of it, in code, comments, tests or commit bodies: "the downstream application",
  "the downstream repository's tools plan" (README, "Rules of this plan").
- **STATUS.md is not this session's** (development.md 3.3). jig's row for `client/domain/client.md` already names "20: the
  one-shot mode … 6: a `Send` carrying a name … : the downstream repository's tools plan"; the designer keeps it.
- **Exhaustive matches.** Every match over `Outcome`, `RequestRefused` or `Event` is exhaustive, with no wildcard arm.
- **The lock.** Every cargo command runs. A conflict in `Cargo.lock` is never merged by hand: take
  `origin/main`'s and run 02.1's `cargo update` again (development.md 5.7).
- **The stories' file.** `tests/client-protocol/tests/client.rs` holds 797 lines at f50c3f63, against the 800-line ceiling: the
  named stories go in a module of their own, `tests/client-protocol/tests/client/named.rs`, declared as `client/words.rs` is.

## Increments

### 02.1 jig on skein's given keys
Scope: the repin of skein onto skein 01's last landing (lib.md 12); jig-client-domain follows `RequestTable`'s typed refusals and
admission numbers at the least change, its behaviour unchanged.
Needs: skein's given keys, landed (skein 01's last landing, step skein-01.2); jig on skein's and smith's main (jig 01's last
landing).
Files: `Cargo.lock`; `crates/jig-client-domain/src/requests.rs` (`PendingRequest`, `send`, `restore_record`, `not_kept`, the
`skein_lib` import); `crates/jig-client-domain/src/tests/requests.rs`. No hotspot.
Shape: in the step's worktree, `cargo update -p skein-lib --precise <sha>`, `<sha>` the commit that landed skein-01.2 (the
run's ledger); `git diff Cargo.lock | grep -o 'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' | sort -u` shows only
skein's revision moving; then `cargo check --workspace --all-targets`. `PendingRequest` gains `admission:
RequestAdmission`. `send` matches `self.table.ask(..)`: `Ok(admission)` keeps the request with it; `Err(RequestRefused::Full(_))`
tells `Outcome::Limit`, as `Err(_)` does today; `Err(RequestRefused::Held(_))` is `unreachable!("a drawn key is never held")`.
`restore_record` keeps its checks in order (the invalid family, the application key already held, then `self.table.restore(..)`):
`Ok(admission)` keeps and tells `Kept`; `Full` and `Held` are both dropped and counted, as `.is_err()` is today (the same engine
key in two kept records is now `Held`). It takes the admission as `self.table.restore(..).ok()`: a `match` of `Ok` to `Some`
trips clippy's `manual_ok_err`, and the exhaustive-match rule covers a `match` only. `not_kept(key)` takes `self.table.owner(key)`, then `self.requests.get(&owner)`, then
`self.table.not_kept(key, held.admission.number)`; when that hands back the owner, it removes the request and tells `NotKept`,
as today. An owner whose current request was admitted under another key passes that admission's number, which skein ignores.
Test shape: every existing test passes unchanged; the not-kept path, now by number, runs in jig-client's `tests/store.rs`
(`a_record_not_stored_answers_its_request_not_kept`, `a_failed_write_answers_only_the_requests_it_carried`,
`a_record_carried_again_is_decided_by_its_later_write`) and in the protocol world's
`a_stop_whose_record_is_refused_is_answered_not_kept_and_never_made`. One new unit test in `tests/requests.rs`,
`a_kept_record_whose_engine_key_is_held_is_dropped_and_told`, written as `a_kept_record_past_the_table_s_room_is_dropped_and_told`
is: kept `record(KEY, ENGINE_KEY, PARTY)` and `record(OTHER_KEY, ENGINE_KEY, PARTY)` under `fixtures::limits()`; told `[Told::Kept
{ key: KEY, request: stop() }, Told::Dropped { count: 1 }, Told::Pending { key: KEY }]`; `c.domain.pending(OTHER_KEY)` is `None`.
Under 20 ms.
Proof: joint, the gate's, as a repin's is (development.md 5.7): the protocol world's not-kept and reload stories run over skein's
new table. It is the layer of the existing tests that check it, because the repin adds no behaviour.
Don't: move smith's pin or a third-party crate; call `ask_given` (02.2); change `Up::NotKept`, `Down::Save`, jig-client's `Store`
or `TABLE_VERSION`; change a test's expectation. A break that needs more is a question (development.md 5.7).
Size: at most about 40 lines besides `Cargo.lock` (requests.rs about 25, the test about 15).
Removes: `restore_record`'s `.is_err()`, and the one-argument call of `RequestTable::not_kept`.

### 02.2 `Send { key, name, request }` and `Outcome::Held`
Scope: client.md 4.1 and 6, and contract.md 3.1: the name, given to the table by `ask_given`, a family's ask included, and kept
in the record beside the application key; `Outcome::Held`, told at once while the table holds the name, nothing kept or sent; the
assert, for the same application key with another request or another name; the four exhaustive matches of `Outcome`.
Needs: jig's repin onto skein's given keys, landed (02.1).
Files: `crates/jig-client-domain/src/boundary.rs` (`Event::Send`, `Outcome`), `domain.rs` (`step`'s `Event::Send` arm),
`requests.rs` (`send`); `crates/jig-client/src/client.rs` (`Event::Send`, its arms in `step` and `step_unlinked`);
`crates/jig-client-view/src/ui/client/outcome.rs` (`answered`, both matches) and `src/intents.rs`;
`examples/ops/client-domain/src/family.rs`; `testing/jig-client-test-domain/src/family/client.rs` and `src/first/proposal.rs`.
Tests: `crates/jig-client-domain/src/tests/{fixtures,requests,versions}.rs`; `crates/jig-client/src/{tests.rs,tests/sign_in.rs,
tests/store.rs}`; `crates/jig-client-view/src/tests/{fixtures/client.rs,domain/owner_project.rs,domain/pending.rs,
domain/policy.rs}`; `examples/ops/client-domain/src/tests_client.rs`; `testing/jig-client-test-domain/src/tests/{root.rs,
root/fixtures.rs}`; `tests/client/tests/{fuzzy_memory.rs,shell.rs}`; `tests/client-protocol/src/client.rs`,
`tests/client-protocol/tests/client/goals.rs`, and the new `tests/client-protocol/tests/client/named.rs`. These are the
`Event::Send` sites that list their fields at f50c3f63; the compiler names any other. No hotspot.
Shape: both `Event::Send` variants gain `name: Option<RequestKey>`; every constructor outside the new tests passes `name: None`,
and jig-client's two arms pass it on. `Domain::send`: registers a family's owner, as today; under a held application key, asserts
`held.request == request && named` with "request key conflict", after `let named = match name { None => true, Some(name) =>
name == held.admission.key };` (clippy.toml disallows `Option::is_none_or`), and returns; without an
identity tells `Unconfirmed`, as today; then asks `self.table.ask(key, party, size, held)` for `None`, or
`self.table.ask_given(key, party, size, name, held)` for `Some(name)`, and matches the result: `Ok(admission)` keeps the request, `Full(_)` tells `Limit`,
`Held(_)` tells `Outcome::Held`, nothing saved, sent or told pending. `Outcome::Held` sits last, with its doc line;
`outcome.rs`'s two matches give it `unreachable!("the reference UI names no send")`; `examples/ops` and the test domain add `|
Outcome::Held` to their `RequestProgress::Unknown` arm. The protocol world's `Story` gains `send_named` and `made_under`, the
latter counting `Observed::Made` under one engine key, as the crate's `transient_request` counts it inline. `cargo fmt` runs
after `send`'s signature is pasted from the sketch, which is wider than rustfmt keeps it.
Test shape: in `crates/jig-client-domain/src/tests/requests.rs`, with new fixtures in `tests/fixtures.rs`: `pub(crate) const
NAME: skein_lib::RequestKey = [3; 16];` and `OTHER_NAME = [4; 16]`, given names beside `ENGINE_KEY` (`[1; 16]`) and
`OTHER_ENGINE_KEY` (`[2; 16]`); `Client::send_as(key, name) -> Token`, written as `send` is today, and `send` becomes
`self.send_as(KEY, None)`. The reference:

```rust
#[test]
fn a_named_send_is_saved_and_sent_under_its_name() {
    let mut c = Client::live();
    c.step(Event::Send { key: KEY, name: Some(NAME), request: stop() });
    let downs = c.downs();
    let [Down::Save { record }, Down::Send { key, .. }] = downs.as_ref() else {
        panic!("save then send");
    };
    assert_eq!((record.key, record.request.key, *key), (NAME, KEY, NAME));
}
```

Beside it:
- `a_name_another_key_holds_is_answered_held_at_once`: after `send_as(KEY, Some(NAME))`, `OTHER_KEY` under `NAME` gives no downs,
  and told is exactly `[Told::Answered { key: OTHER_KEY, outcome: Outcome::Held }]`.
- `a_name_a_restored_record_holds_is_answered_held`: `Start { kept }` of `record(KEY, NAME, PARTY)`, then `me`.
- `a_held_name_sent_before_me_names_a_party_is_unconfirmed`: the same restore without `me`: `Unconfirmed`, not `Held`.
- `the_same_key_and_name_sent_again_joins_the_first`.
- `#[should_panic(expected = "request key conflict")] the_same_key_under_another_name_is_a_loud_key_conflict`, with `OTHER_NAME`.
- `a_name_is_admitted_again_once_its_holder_is_answered`: a `Final` answer as in `a_final_answer_is_told_once_with_its_outcome`,
  `c.downs()` and `c.told()` both drained after it, then `OTHER_KEY` under `NAME` is saved and sent under `NAME`.
- `a_name_admitted_again_is_retired_by_its_own_not_kept`: `NotKept { key: NAME }`, `send_as(OTHER_KEY, Some(NAME))`, `NotKept {
  key: NAME }` again; told is `[Told::Answered { key: OTHER_KEY, outcome: Outcome::NotKept }]`: the second admission's number.
- In `tests/versions.rs`, `a_family_send_under_a_held_name_is_answered_held`: with its `live()` and `record()`, an `Ask::Family`
  under `OTHER_KEY` naming the `NAME` that `fixtures::KEY`'s stop holds.

Nine unit tests, as the name meets each of `send`'s four branches, the not-kept path and a family's ask; each under 20 ms. In the
protocol world, the new module's reference story, under 0.5 s:

```rust
//! A send under a name, in front of the engine (client/domain/client.md, 6; contract.md, 3.1).

use super::{PROOF_WAIT, SECOND_STOP, STOP, Story, TASK, fixtures, stop};
use jig_client::Outcome;
use skein_lib::RequestKey;

/// A name the application gives, as an agent's key digested would be.
const NAME: RequestKey = [23; 16];

#[test]
fn a_second_key_naming_a_held_name_is_told_held() {
    let mut story = Story::live(fixtures::SEED);
    story.send_named(STOP, NAME, stop(TASK));
    story.send_named(SECOND_STOP, NAME, stop(TASK));
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), PROOF_WAIT));
    assert_eq!(story.answered(SECOND_STOP), vec![Outcome::Held]);
    assert_eq!(story.made_under(NAME), 1);
}
```

`named.rs` reads the parent's constants and helpers through `super::`, as `words.rs` reads `STOP`; `tests/client.rs` gains only
`#[path = "client/named.rs"] mod named;`.
Proof: joint, `a_second_key_naming_a_held_name_is_told_held`, in front of the real engine over HTTP bytes (the protocol world): a
second application key naming the name the first holds is told held at once; the first is answered, and the engine makes it
once, under its name. Run in the gate.
Don't: add a view word, a `NotSent` variant or a notice for *held* (view.md is design); carry the number on `Up::NotKept` or
`Down::Save`; change `TABLE_VERSION`, the wire, `Held` or `RequestRecord`; join a named `Send` to another application key's
request ("Not joined"); name a send in production code.
Size: about 50 production lines; the tests' `name: None` at the constructor sites aside.
Removes: 02.1's `unreachable!("a drawn key is never held")` arm, now `Outcome::Held`; the assert's request-only condition.

### 02.3 The named stories in front of the engine
Scope: the engine's half of client.md 6 and contract.md 3.1: a named request kept across a restart is sent again under its name
and answered the engine's first answer; a name asked again once its holder is erased gets the first answer; a name sent with
another ask is refused as a conflict, never loudly by the client.
Needs: jig's send under a named key, in place (02.2, landed).
Files: `tests/client-protocol/tests/client/named.rs`. No production code. No hotspot.
Shape: three stories beside 02.2's, on the fixtures there: `Story::reload` keeps the store and starts a new client on it;
`cut_answer` loses the engine's answer; `assert_one_refusal` reads `STOP`'s one refusal and one make for `FIRST_OWNER`. A name
asked again waits for no erase: each `Front` step ends with `reclaim`, so the step that tells the first answer frees the name.
Test shape: `const OTHER_TASK: u64 = TASK + 1;` names another ask, a stop of a task no story starts. The reference, written as
`a_stop_kept_across_a_reload_is_answered_once` is:

```rust
#[test]
fn a_named_stop_kept_across_a_reload_is_answered_once_under_its_name() {
    let mut story = Story::live(fixtures::SEED);
    story.cut_answer();
    story.send_named(STOP, NAME, stop(TASK));
    assert!(story.run_until(|s| s.made_under(NAME) == 1, PROOF_WAIT));
    story.reload();
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), PROOF_WAIT));
    assert!(story.kept(STOP));
    assert_one_refusal(&story);
    assert_eq!(story.made_under(NAME), 1);
}
```

Beside it: `a_name_asked_again_once_its_holder_is_erased_gets_the_first_answer` (`STOP` under `NAME` runs until answered, then
`SECOND_STOP` under `NAME` with the same ask; `answered(SECOND_STOP) == answered(STOP)` and `made_under(NAME) == 1`), and
`a_name_sent_with_another_ask_is_refused_as_a_conflict` (the same with `stop(OTHER_TASK)`; `SECOND_STOP`'s one answer is
`Outcome::Answered(Answered::Jig(contract::Answer::Refused { refusal: contract::Refusal::KeyConflict, .. }))`, and nothing
panics). Each under 0.5 s, in the focused suite.
Proof: joint, the stories themselves, at their own layer: nothing above the protocol world names a send in jig; the downstream
application's one-shot mode, in the downstream repository's tools plan, is the first.
Don't: change production code (a story the client fails is 02.2's fix, raised as a question once 02.2 has landed); add to
`tests/client-protocol/tests/client.rs`; add a seed matrix (one story each; testing-strategy.md 10).
Size: no production lines; about 60 lines of tests.
Removes: nothing.

## Interface sketches

```rust
// crates/jig-client-domain/src/boundary.rs
pub enum Event {
    // Start, Watch, Unwatch, Read, Up and the rest unchanged.
    /// A keyed request under the application's key, and the engine key it names, or none for one drawn (client.md, 6).
    Send { key: Key, name: Option<RequestKey>, request: Ask },
}

/// A keyed request's one answer (client.md, 6).
pub enum Outcome {
    /// Its record could not be written, so it was never sent (client.md, 11).
    NotKept,
    /// The engine's answer: done or refused, jig's or a family's.
    Answered(Answered),
    /// First sent longer ago than the retention: whether the engine made it is unknown.
    Unknown,
    /// Past the client's requests or their bytes; never queued.
    Limit,
    /// Sent before `me` named the party; nothing of it kept or sent.
    Unconfirmed,
    /// Its name is held by another request in the table; nothing of it kept or sent.
    Held,
}
```

```rust
// crates/jig-client/src/client.rs
pub enum Event {
    // The other variants unchanged; `Send` takes no doc line, which cargo fmt would split the enum for.
    Send { key: Key, name: Option<skein_lib::RequestKey>, request: Ask },
}
```

```rust
// crates/jig-client-domain/src/requests.rs
/// A request the table holds, by its application key.
pub(crate) struct PendingRequest {
    /// What was asked, compared when the same key is sent again.
    request: Ask,
    /// Its progress, for showing.
    pending: Pending,
    /// The attempt in flight, if any.
    attempt: Option<Token>,
    /// The engine key the table holds it under, and that admission's number, kept until its answer or not-kept report.
    admission: RequestAdmission,
}

impl Domain {
    /// Keeps a keyed request, joins it to the same, or answers it at once: unconfirmed, held or limit (client.md, 6).
    pub(crate) fn send(&mut self, env: &Env<Limits>, key: Key, name: Option<RequestKey>, request: Ask, out: &mut Queue<Request>);
    /// Retires the request admitted under `key` whose last small-store write failed, under that admission's number.
    pub(crate) fn not_kept(&mut self, key: RequestKey, out: &mut Queue<Request>);
}
```

```rust
// tests/client-protocol/src/client.rs
impl Story {
    /// Sends `request` under the application's `key`, named `name`.
    pub fn send_named(&mut self, key: Key, name: RequestKey, request: Ask);
    /// How many durable answers the engine made under the engine key `key`.
    #[must_use]
    pub fn made_under(&self, key: RequestKey) -> usize;
}
```

## Questions to raise when reached

- **02.1: smith's crates fail to build on skein 01's landing.** skein's `main` moved further than skein 01 between jig 01's pin
  and skein-01.2's commit. The step lands nothing; raise it with the errors, the two revisions and whether smith's `origin/main`
  builds on the new skein. Moving smith is then the supervisor's call (development.md 5.7).
- **02.1 or 02.2: skein 01 landed other names than its sketch's** (`ask_given`, `RequestAdmission`, `RequestRefused`). The landed
  names win; the step follows them, and asks the supervisor to amend this file's "Done when" before landing.

## Done when

Each in `~/src/rust/jig`, after `git fetch`:
- `test -z "$(git show origin/main:Cargo.lock | grep -o 'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' | sort -u | cut -d'#' -f1 | uniq -d)"`
- `git show origin/main:crates/jig-client-domain/src/requests.rs | grep -q -F 'ask_given('`
- `git show origin/main:crates/jig-client-domain/src/requests.rs | grep -q -E 'table\.not_kept\(key, [a-z_.]+\)'`
- `git show origin/main:crates/jig-client-domain/src/requests.rs | grep -q -F '"request key conflict"'`
- `git show origin/main:crates/jig-client-domain/src/boundary.rs | grep -q -F 'Send { key: Key, name: Option<RequestKey>, request: Ask },'`
- `git show origin/main:crates/jig-client/src/client.rs | grep -q -E 'Send \{ key: Key, name: Option<(skein_lib::)?RequestKey>, request: Ask \},'`
- `git show origin/main:crates/jig-client-domain/src/boundary.rs | sed -n '/^pub enum Outcome/,/^}/p' | grep -q -x '    Held,'`
- `git show origin/main:crates/jig-client-domain/src/lib.rs | grep -q -x 'pub const TABLE_VERSION: u16 = 2;'`
- `git grep -q -F 'Outcome::Held' origin/main -- crates/jig-client-view/src/ui/client/outcome.rs`
- `test -n "$(git grep -l --all-match -e 'send_named(' -e 'reload()' origin/main -- tests/client-protocol/tests)"`
- `test -n "$(git grep -l --all-match -e 'send_named(' -e 'Refusal::KeyConflict' origin/main -- tests/client-protocol/tests)"`
- `test -z "$(git grep -l -i temper origin/main -- crates/jig-client-domain crates/jig-client crates/jig-client-view tests/client-protocol)"`
