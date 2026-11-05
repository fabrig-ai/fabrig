# debt-16: the sixteenth debt round

Three parts (`inputs/request.md`), from the spot review of debt-13 (`inputs/debt13-review.md`), read and not run. All are
in this plan, but for what's skipped below with its reason.

## Parts and tasks

| Request | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| 1 | `wall-due` | `wall-due` | | medium | 15 | finding 5: `expiry <= env.wall` at the call sites |
| 1 | `core-small` | `core-small` | | medium | 20 | finding 10, jig's half |
| 1 | `side-out` | `side-out` | | high | 75 | finding 2: `Side` out of `Waiting` in `drive_calls` |
| 1 | `own-exchange` | `own-exchange` | `side-out` | high | 50 | finding 4 |
| 1 | `test-layout` | `test-layout` | `own-exchange` | medium | 50 | finding 8, and finding 10's `remove` |
| 2 | `now-link` | `now-link` | | high | 80 | finding 3: `Now::Link(LinkNow)` |
| 2 | `leaves-back` | `leaves-back` | `now-link` | medium | 20 | finding 6: two helpers back to the roots |
| 2 | `charter-parts` | `charter-parts` | `leaves-back` | medium | 40 | finding 7: `RootParts`, `Assignment` into core |
| 3 | `temper-shared` | `temper-root-shared` | | medium | 30 | job assignment, settled relay, sender, `new_work` |
| 3 | `temper-shared` | `service-identity` | | medium | 15 | `run_identity`, with a vector |
| 3 | `temper-shared` | `composed-sender` | | medium | 10 | `Party::sender` in the composed world |
| 3 | `job-refused` | `job-refused` | | medium | 20 | `ToJob::refused` |
| 3 | `ops-protocol` | `ops-protocol` | | high | 45 | `jig-ops-protocol-observability` |
| 3 | `d004-rewrite` | `d004-rewrite` | all of part 3, `charter-parts` | medium | 25 | D004 stays open for the test root; three status files |

## Critical path

`side-out`, `own-exchange`, `test-layout`: 175 minutes. `now-link`, `leaves-back`, `charter-parts`, `d004-rewrite` is 165
beside it. Nothing crosses between the two chains.

## Findings, checked against the code

- **2, right, but for its last sentence.** All 23 sites hold. `a_call_of_the_other_dialect_is_unavailable` stays:
  `skein/docs/design/forge-connection.md`, 7.1 says a call whose variant isn't its forge's is answered as a call to a
  forge that has gone, and that arm is the one place the pair is met, with no `expect` and no `unreachable!`.
- **3, right.** 13 construction sites. Its "can't fail" is overstated: the pointer assertion fails only on a copy, and it
  goes with the `Err` path anyway. Temper's root does match the six, so it follows, as the request allows.
- **4, right, and one site more.** `own` clears in `connect`, `own_closed` and `assign`; `shed` doesn't clear it but does
  `state.forget()`, so it resets too. Plain removal of `h.pump()` fails three of the test's four terminals, because a bare
  `Connection` has no hold on its exchange: two terminals move up to a component test.
- **5, right.** `Wall` is `Ord`; `wall_due` differs from the comparison only at a saturated `now`, the wrong answer its
  test pinned.
- **6, half right.** `no_workspace` and `complete_brief` go back. `settled_answer`'s `Option` is wrong: the test root takes
  the answer back from the `Err`, and temper becomes its third caller. A by-value `Option` loses the answer, and a
  by-reference one copies a body of up to 1 MiB. `leaves.rs:48` isn't a fourth helper: it's `no_workspace`.
- **7, mostly right.** Drop `answered` and `messages`, rename, drop the dependency. By-value is wrong: `ops` validates
  through `hosted` and then keeps the same value, and `Assignment` isn't `Clone`.
- **8, right.** The three `#[path]` moves make `connection/tests.rs` the connection's own tests.
- **9, skipped: the saving isn't there.** The two blocks do match as text, but over two `Body` types and two `Heard` types,
  and step code bans generics and `macro_rules!`. `Payload::closes` can't be written without a `Body`. `Exchange::release`
  replaces three two-line resets with one four-line function on each dialect: +1 line each, not -12.
- **10, right.** `Counters::commit_header` stays: temper's `decision.rs` calls it.

## Decisions

- **`Now::Link` isn't a public break.** The request says to stop if it is. `Now` is `pub`, but the workspace is `publish =
  false` at `0.0.0` on path dependencies, so nothing outside this repository can read it; nothing of it is stored or on a
  wire; no design promises it is stable; and temper, its one reader outside jig, changes in the same branch. It's a
  refactor that a later commit can undo. The same goes for `hub_link::now`'s signature, the two helpers and `Given`. The
  spec stops if a stored form, a wire form or a golden text moves. The plan's owner can overrule this.
- **`ops`'s protocol layer is a new crate its root calls in the step, not a hand-out.** The item assumed a layer that
  doesn't exist: `jig-ops-protocol-observability` is in `examples.md`, 2 as designed but unbuilt. A hand-out by `Output` and
  `Event`, as `ReadResult` is, would put the connector's `Save` rows and `ProcedureStep` in two commits, which moves commit
  counts, and the request says behaviour stays. A crate the root calls keeps one decision, and makes "delegate JSON and
  alert words live in `ops`'s protocol layer" true. `jig-ops-domain` already calls `jig-protocol-charter` and
  `jig-protocol-tools` in the step (`hosts.md`, 8: "A root calls them and decides none of it"). The hand-out stays recorded
  as a departure from `root.md`, 11 with its reason, and in D004. `examples.md`, 2 and 4 are written in this cut.
- **`Class` isn't adopted.** The request lists it, but it means a new dependency and changing the public `completes`, whose
  callers read a `bool`. `new_work` is the shared part `root.md`, 3 and `root.status.md` name.
- **Temper's `message`, `inbox_sender`, `inbox_words` and `answered` stay.** The first three render other words than
  `jig_protocol_charter::message`, and bytes don't move. `answered` waits on `hosts.md`, 12, whose default is "it keeps its
  own". `decision.rs` needs nothing: `decision_header` is `wrote || dirty`, and temper's is `persist_header && dirty`.
- **`ToJob::refused`, and no `Delivery::refused`.** The first takes 8 lines from each of two roots' templates; the second
  saves none. `Prepare` has one caller.
- **`Assignment` moves into `jig-domain-core`,** with `pub use` in the hub link. All its fields are core's or core's
  dependencies, the hub link can't link smith, and `jig-domain-host` and `jig-domain-link` are ruled out by `graph.rs`.

## One writer per crate

Owners run in chains: `skein-forge-connection`, and `jig-domain-core` with `jig-domain-hub-link`. Three crates take reader
edits from a chain while their owner runs, in hunks that don't meet: `jig-ops-domain` (`now-link` and `leaves-back` in
`core_now`'s arms, `ops-protocol` in the connector routing, `job-refused` in `link.rs`), `temper-engine-domain` (`now-link`
in `route/now.rs`, `temper-root-shared` in `link.rs` and `admission.rs`), and `jig-test-worker`. Holding those would make a
175-minute wait for no merge risk: a task waits only for the code it builds on.

## Overlaps with the drafts beside this one

- **`look-round-4`:** none. `temper-client-*` and `temper/tools/gallery` aren't touched.
- **`debt-14`:** `Cargo.lock` and the root `Cargo.toml` only, mechanical. `ops-protocol` and `charter-parts` touch them.
- **`hearth-memory-2`:** none.
- **`reference-walks`, `v0.1.0-ladder`, `configuring-walks`:** queued, with no files known yet.
