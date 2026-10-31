+++
id = "rm-domain"
part = "forge-remove"
kind = "feature"
needs = ["forge-families", "jig-acts", "fr-domain", "cl-wire"]
files = [
  "temper/crates/temper-engine-domain-forge/src/remove.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/remove.rs",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/demo.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain/src/route/kept.rs",
  "temper/crates/temper-engine-domain/src/route.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/route/applications.rs",
  "temper/crates/temper-engine-domain/src/route/held.rs",
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/domain.rs",
  "temper/crates/temper-engine-domain/src/restart.rs",
  "temper/crates/temper-engine-domain/src/decision.rs",
  "temper/crates/temper-engine-domain/src/tests/remove.rs",
]
effort = "high"
minutes = 90
+++

## Goal

`RemoveForge { forge }` decided in the forge connector: in one decision
the forge is ended at once (for any forge, as the playground's forge is
ended today), its writer and flights erased, and the rest kept as
`KeptAct` rows (one jig `Cancel` per change on its repositories, the
box's group removal, the secrets' erase), each sent after the decision
commits and again at each start until answered; the forge's record is
erased once no kept act names it. The root routes a kept `Cancel` to
jig's `Act::Cancel` and releases the other two as root requests the
service answers (`rm-service` builds the service side).

Acceptance (new tests, all in the two crates' unit tests):

- `temper-engine-domain-forge`, `src/tests/remove.rs`:
  - `removing_a_forge_ends_it_at_once_and_keeps_its_acts`: a connected
    forge with an adopted repository and a live change; `RemoveForge`
    answers `Removing`; in the same output the record is saved with
    `removing: true`, its `Writer` row and each of its flights erased,
    one `KeptAct` saved per change (`Cancel { project, task }`), one
    `EraseSecrets`, and `RemoveGroup` under the box for an owned or fake
    forge only; `forge_ended(forge)` is true; the forges document lists
    it not; `readable_repositories`/`writable_repositories`/`home` omit
    its repositories while `repository(..)`/`repository_tag(..)` still
    find them.
  - `a_removing_or_unknown_forge_is_refused_unknown` and
    `the_playgrounds_forge_is_refused_not_offered` (a `Playground` row
    naming it, either state; and today's demo forge while the demo runs).
  - `kept_acts_are_sent_after_the_decision_and_again_at_each_start`:
    `SendKeptAct` once per row in the deciding output, and again on
    `Event::SendKeptActs` after a restore.
  - `the_record_is_erased_with_its_last_kept_act`: answering each act
    (`KeptActAnswered { keep: false }`, `GroupRemoved`, `SecretsErased`)
    erases its row; the last also erases `Key::Forge(forge)`;
    `keep: true` erases nothing; a number once given and erased stays
    ended (`forge_ended` true, no record).
  - `a_removed_forge_refuses_connect_and_set_up`: `Connect`/`SetUp`/
    `Install`/`Accept`/`Reachable` on it answer as on an ended forge
    today.
- `temper-engine-domain`, `src/tests/remove.rs`:
  - `a_kept_cancel_is_jigs_cancel_under_its_key`: the root turns
    `SendKeptAct(Cancel)` into `core::Event::Act` with
    `people::Act::Cancel { project, task, grace: None, reason }` under
    the row's key and the forge connector's number.
  - `cancel_outcomes_keep_or_erase_as_the_demo_end_does`: `Again`,
    `Refused(NotOffered)` and `Refused(Role { held: None })` keep the
    row (the two refusals with a secret-free note); `Compound`,
    `Refused(Unknown)`, `Refused(Ended)` and other refusals erase it.
  - `group_and_secrets_acts_are_released_after_their_commit`:
    `Request::RemoveGroup { forge }` and `Request::EraseSecrets { forge }`
    leave the root only after the deciding commit.
- Every package depending on either crate builds; their tests pass
  unchanged in meaning (the ended-demo tests included: today's demo end
  still ends its forge).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 2 (the
  `ForgeRecord`, `KeptAct`, the rules of numbers and removal) and 5
  (`RemoveForge` and its answers).
- `temper/docs/design/draft/playground-reset.md`, 4 (whole) and 9 (the
  second default: a removed forge's repositories kept).
- `temper/docs/design/domain/forge.md`, 4.1 (an ended forge: what no
  longer lists, calls, hints).
- `temper/docs/design/protocol/setup.md`, 8.4 (the demo end's kept
  acts: keys, resend at start, answers kept or erased; this task's
  pattern).
- `temper/docs/design/domain/root.md`, 4 (held deliveries) and 9.1
  (restart).
- `jig/docs/design/domain/people.md`, 5.3 (the deployment's acts and
  their keys).
- The code: `temper-engine-domain-forge/src/demo.rs` (`forge_ended`,
  `acts`, `send_acts`, `act_answered`), `src/kept.rs` (forge-families'
  held rows and request makers), `domain.rs`'s `retire_ended`,
  `erase_flight`; `temper-engine-domain/src/route/demo.rs`
  (`send_end_act`, `end_acted`, `end_note`) and `restart.rs` near its
  `SendDemoActs`.

Assumed from other tasks: forge-families' `ForgeRecord`, `KeptAct`,
`KeptActKind`, `Playground`, `Domain::forge_record`, `forge_records`,
`forge_next`, `playground`, `kept_acts`, and `kept::{save_forge,
erase_forge, keep_act, erase_act}`; fr-domain's records in place of
today's configured forges (whatever map fr-domain keeps for them, a
forge is known iff `forge_record(forge)` is some); jig-acts'
`people::Act::Cancel`, `ActKind::Cancel`; cl-wire's
`w::Outcome::{Removing, RemoveRefused}` and `w::RemoveRefusal`.

## Owns and may change

Owns the files in the header: in the forge crate the new module
`src/remove.rs` and its tests; in the root the new module
`src/route/kept.rs` and its tests.

May change, as the work needs:

- The forge crate's `boundary.rs` (the vocabulary below), `domain.rs`
  (the dispatch arms, the new volatile fields, `retire_ended`'s guard,
  the restore's tail), `demo.rs` (`forge_ended`'s body), `lib.rs`
  (module and exports), and every reader of `forge_ended` or of
  `DemoState::Ended` in the crate (`adoption.rs`, `items.rs`,
  `topics.rs`, `restore.rs`, `reconcile.rs`, `connect.rs`) where a
  reader needs more than the predicate's new body.
- The root's `boundary.rs`, `domain.rs`, `route.rs`,
  `route/forge_route.rs` (the forge request arms), `route/applications.rs`,
  `route/held.rs`, `restart.rs`, `decision.rs` (the new deliveries'
  accounting) and `store.rs` (forced arms).
- Forced arms per `notes/scope.md`, in any crate and test world:
  exhaustive matches on `forge::Ask`, `forge::Answered`, `forge::Event`,
  `forge::Request`, `root::Request`, `root::Delivery`,
  `root::ApplicationAnswer`. The survey's readers of `root::Request`
  (each lists `StopGroup`): `temper-service/src/route.rs`,
  `src/tests/{engine,fixtures,setup_github}.rs`;
  `temper/tests/engine/domain/src/{escalation,walking,direct,forge_routes,roles,protocol}.rs`;
  `temper/tests/engine/forge/src/world.rs`,
  `tests/suite/simulation.rs`; `temper/tests/engine/protocol-forge/src/protocol.rs`,
  `tests/suite/protocol.rs`; `temper/tests/system/src/{composed,forge}.rs`;
  `temper/tests/client/src/{translate,translate/forge}.rs`. Put each new
  arm on its own line next to the reader's `Unsettled` (or its first)
  arm, never next to `StopGroup`/`SetUpGroup`, which `bx-serve`
  rewrites. In `temper-service/src/route.rs` the two new requests are
  a no-op arm with the comment `// rm-service routes these`.
- `temper-service/src/family.rs`'s answer match (forced by
  `ApplicationAnswer::RemoveForge`): map `Removing` to
  `w::Outcome::Removing`, `NotOffered`/`Unknown` to
  `w::Outcome::RemoveRefused` with `w::RemoveRefusal::{NotOffered,
  Unknown}`; place the arm first in the match. The request arm
  (`w::Request::RemoveForge`, cl-wire's `Undecodable` bridge) stays:
  `rm-service` replaces it.
- If fr-domain left `Domain.ended`, `configured` or a successor map
  keyed by forge, remove the removed forge's entry where it would keep
  the forge listed.

## Steps

1. **Vocabulary** (forge crate `boundary.rs`; put `Ask::RemoveForge`
   last in `Ask`, the new `Event` variants right after
   `Event::BoxClosed`, the new `Request` variant right after
   `Request::SendDemoAct`, so the `pg-` tasks' edits of the demo's
   variants do not touch these lines):

   ```rust
   pub enum Ask { /* ... */ RemoveForge { forge: u16 } }
   pub enum Answered { /* ... */ RemoveForge(RemoveAnswer) }
   /// playground-reset.md, 4.
   pub enum RemoveAnswer { Removing, NotOffered, Unknown }
   pub enum Event {
       /// The root's answer to a kept act it sent to jig.
       KeptActAnswered { key: [u8; 16], keep: bool },
       /// Send every kept act again: the core is open after a start.
       SendKeptActs,
       /// The box removed the forge's group (contracts.md, 3).
       GroupRemoved { forge: u16 },
       /// The service erased every secret item of the forge.
       SecretsErased { forge: u16 },
   }
   pub enum Request { /* ... */ SendKeptAct(KeptAct) }
   ```

   Root `boundary.rs`: `Request::RemoveGroup { forge: u16 }` and
   `Request::EraseSecrets { forge: u16 }` (doc: released after the
   decision that keeps the act; the service answers with the forge
   events above); `Delivery::RemoveGroup { forge }` and
   `Delivery::EraseSecrets { forge }` mapped by `delivery_output` to
   those requests (as `Delivery::Bound` maps to `Request::Bound`);
   `ApplicationAnswer::RemoveForge(forge::RemoveAnswer)`.

2. **Ended, for any forge** (`demo.rs`'s `forge_ended`, kept where it
   is so every reader follows): true when the forge's record is
   `removing`, or when the forge has no record and its number is below
   `forge_next()` (given once, gone), or for today's ended demo forge
   (kept until `pg-gone`). `retire_ended` runs whenever any forge is
   ended (drop its guard on the demo state; call it from the removal's
   decision). Walk survey d's reader table (section 2) once: each call
   site keeps its meaning with the general predicate; where a site reads
   `DemoState::Ended` directly to mean "this forge ended", make it read
   `forge_ended`. The service's `crate::forge::ended` mirror reads the
   root's predicate and needs nothing.

3. **The decision** (`src/remove.rs`):

   ```rust
   pub(crate) fn ask(d: &mut Domain, env: &Env<Limits>, request: Token, token: Token,
                     retention: Duration, forge: u16, out: &mut Queue<Request>);
   pub(crate) fn answered(d: &mut Domain, env: &Env<Limits>, key: [u8; 16], keep: bool,
                          out: &mut Queue<Request>);
   pub(crate) fn group_removed(d: &mut Domain, env: &Env<Limits>, forge: u16, out: &mut Queue<Request>);
   pub(crate) fn secrets_erased(d: &mut Domain, env: &Env<Limits>, forge: u16, out: &mut Queue<Request>);
   pub(crate) fn send(d: &Domain, out: &mut Queue<Request>);   // every kept act, key order
   ```

   `ask`: no record, or `removing`: `Unknown`. The `Playground` row
   names the forge (either state), or the forge is today's running
   demo's: `NotOffered`. Otherwise, in this order, in one output:
   - save the record with `removing: true` (`kept::save_forge`);
   - select its repositories (`d.repositories` whose `provider.forge`
     is the forge) and, for each, its changes (`changes_for`), keeping
     `KeptAct { key, forge, act: Cancel { project: repository.project,
     task } }` per change;
   - `RemoveGroup` when `d.under_box` and the record's `mode` is
     `Owned` or `Fake`; `EraseSecrets` always;
   - erase the forge's writer (`d.writers`, `Key::Writer(forge)`) and
     each flight whose entry's repository is on the forge (join
     `d.flights` to `d.entries`; `erase_flight`); answer pending
     setup/connect requests for the forge as an ended forge's are;
   - `retire_ended`; a `Request::MarkedForges` (or whatever refreshes
     the forges document today) so watchers drop the row;
   - `SendKeptAct` for each new row;
   - answer `Removing`, held as setup answers are (a volatile
     `remove_answers: Map<Token, (Wall, RemoveAnswer)>` with the
     request's retention, read by `application_answer`), then
     `Request::ApplicationDone`.

   Keys: `sha256("temper forge removal\0" || forge as 2 bytes BE ||
   purpose || number as 8 bytes BE)`, first 16 bytes; purposes
   `b"cancel"` (number the task), `b"group"` and `b"secrets"` (number
   0). A key already kept is not kept twice.

   `answered`/`group_removed`/`secrets_erased`: erase the matching row
   unless `keep`; then, when no kept act names the forge, erase its
   record (`kept::erase_forge`). The record's repository rows stay
   (draft 9's default: `rm-project` erases them with their project).

4. **Dispatch** (forge crate `domain.rs`): `Event::Application` with
   `Ask::RemoveForge` calls `remove::ask`; the four new events call
   their functions; `connect.rs`'s `unreachable!` admissions list
   `RemoveForge`; `application_answer` returns `RemoveForge`.

5. **The root** (`src/route/kept.rs`):
   - `pub(crate) fn send(domain, row: forge::KeptAct)`, from
     `forge_route.rs`'s `SendKeptAct` arm: `Cancel { project, task }`
     becomes `core::Event::Act { reply_to: ReplyTo::new(token),
     connector: domain.config.forge_connector, key: row.key, act:
     Box::new(people::Act::Cancel { project, task, grace: None,
     reason: b"Its forge was removed".as_slice().into() }), carried:
     None }`, the token held in a new `kept_replies: Map<Token,
     forge::KeptAct>` (capacity: the forge limits' kept-act bound) from
     a sequence of its own, disjoint from the demo end's; `RemoveGroup`
     and `EraseSecrets` are emitted as `Delivery::RemoveGroup`/
     `EraseSecrets` (held behind the commit).
   - `pub(crate) fn acted(domain, env, decision, to, outcome)`: from
     `Delivery::DemoActed` when `to`'s token is in `kept_replies` (the
     demo end's `end_acted` keeps its own tokens): keep on `Again`,
     `Refused(NotOffered)` (note: "the forge connector is not granted
     cancel (engine.acts)"), `Refused(Role { held: None })` (note: "no
     first owner configured"); erase on every other outcome, a refusal
     other than `Unknown`/`Ended` noted by its name (reuse
     `demo::refusal_name`). Notes go out as today's `Delivery::DemoNote`
     with the prefix `forge removal: task N not cancelled: `, at most
     256 bytes. Then `forge::Event::KeptActAnswered { key, keep }`.
   - `restart.rs`: push `forge::Event::SendKeptActs` beside today's
     `SendDemoActs`.
   - `route/applications.rs`: `Ask::RemoveForge` is of the deployment's
     scope; `answer` maps `Answered::RemoveForge`.
6. **Tests** as the acceptance names, built on the crate's existing
   fixtures (`src/tests/fixtures.rs` in the forge crate; the root's
   `src/tests/` helpers). Then, one at a time:
   - `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/remove::/)'`,
     then the crate's whole `--lib`;
   - `heavy cargo nextest run -p temper-engine-domain --lib`;
   - `heavy cargo nextest run -p temper-engine-domain-world -p temper-engine-forge-world -p temper-engine-protocol-forge-world -p temper-service -p temper-system-world -p temper-client-world`
     (the readers of `forge_ended` and of the root's requests: a change
     of meaning);
   - `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-domain -p temper-service --all-targets -- -D warnings`
     and every other package you edited;
   - `python3 -I -B scripts/old_forms.py` from `temper/`.
7. Commit (vocabulary and arms; the decision; the root; tests) and push.

## Report

The standard report, plus: each `forge_ended` reader whose code
changed and why; how pending setup/connect requests of a removed forge
are answered; whether the forge client child holds a writer the removal
cannot drop (and what then guards it); the kept-act bound you used and
its check; every forced arm outside the owned files.
