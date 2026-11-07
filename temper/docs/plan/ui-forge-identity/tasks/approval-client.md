+++
id = "approval-client"
part = "approval"
kind = "feature"
needs = ["approval-wire"]
files = [
  "temper/crates/temper-client-domain/src/forge/approvals.rs",
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/forge/me.rs",
  "temper/crates/temper-client-domain/src/forge/ways.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/pages/needs.rs",
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-domain/src/pages/chat/sheet.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests.rs",
  "temper/crates/temper-client-domain/src/tests/approvals.rs",
  "temper/crates/temper-client-domain/src/tests/pages/needs.rs",
  "temper/crates/temper-client-domain/src/tests/pages/home.rs",
  "temper/crates/temper-client-domain/src/tests/forge_ways.rs",
]
effort = "high"
minutes = 100
+++

# The client watches the approvals and merges them into Needs you

## Goal

A change waiting on a person's review on the forge is a card under Needs you (`temper/docs/design/client/README.md`, 3.5, *An approval*). In `temper-client-domain`: the forge side watches the subject `approvals` once for the session and keeps its rows; the needs session merges them with jig's inbox for Needs you, its project and goal filters, Home's count and the rail's; Later takes them. And the forge side reads a proposal's items for its landing rules too, keeping `owned_approval` for its card's sentence (same, 5.13). The view draws none of it yet: that's `approval-card`.

Acceptance:

1. `forge::approvals::{Row, Yours, Approvals}`, `Side::approvals`, `Side::owned_approval`, `pages::needs::Need`, `Session::needs`, and `Session::shown` and `Session::count` changed in place, are as `temper/docs/plan/ui-forge-identity/notes/contracts.md`, *The approval read*, *The client*, writes them, with doc comments in the design's words. `Session::open` stays as it is.
2. The watch: `Scope::Own`, window unused, opened once from when `me` is first delivered while the link is live, as `trials` and `dismissed` are; refused or ended, kept so with no rows until the next sign-in; closed at sign-out. Its document decodes whole or the watch ends.
3. The merge: the inbox's open cards in its order, then the rows its filter takes (project: the row's `project`; goal: the row's `goal`; any kind: none), those marked later last; none until `inbox.known()`.
4. Counts: the rail's (`count`) adds the project's rows, cut where `more` is above 0; Home's `Needs::Count` counts `needs`, cut likewise.
5. Later, and every needs act that looks a card up among the open ones, finds an approval's card by `ObjectKey::Task(change)`.
6. A proposal whose review names an application item of kind 1 or 2 has its items read, as one of kind 6 or 7 does; `owned_approval` answers what the read said.
7. The tests of Steps, 6 bite; `heavy cargo nextest run -p temper-client-domain` passes; `heavy cargo check --tests -p temper-client-view -p temper-client-gallery -p temper-client-world -p temper-client-shell-terminal` passes; clippy clean for `temper-client-domain`.

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/common.md`.
- `temper/docs/plan/ui-forge-identity/notes/contracts.md`, *The approval read*: *The wire* and *The client* bind.
- `temper/docs/design/client/README.md`, 3.5: *Needs you*'s *An approval*, *What the client keeps*'s *The rail's count* and *The approvals*, and Home's *Needs you* region; 4.6, *The changes waiting on a person's review, the subject `approvals`*; 5.13, *What a rule on a box holds against*.

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours, compiler-forced edits only** (each named in the report):

- `temper/crates/temper-client-view/src/pages/needs.rs` (~44): `session.shown(inbox)` becomes `session.shown(inbox, root.forge().approvals())`, drawing a `Need::Card(card)` as it drew `card` and nothing for `Need::Approval(_)`, which `approval-card` draws. `temper/crates/temper-client-view/src/frame.rs` (~840): `count(root.client(), root.forge().approvals(), project)`. The view's `open(inbox)` callers stay.
- Any other caller of `shown` or `count` the compiler names (the gallery's `tests.rs`, `temper/tests/client`'s suites): the same, taking `root.forge().approvals()`.

**Options, each with its default:**

- Where `needs::act` gets the approvals: `pages::step` can't reach the forge side. Default: `needs::act` gains `approvals: &forge::approvals::Approvals`; `chat::sheet::step` (`pages/chat/sheet.rs`, the page boundary that already supplies root facts) routes `Event::Act { act: Act::Needs(act) }` itself, calling it with `root.forge.approvals()`, and returns; `pages.rs`'s `Act::Needs` arm passes an empty one (a `const` or `Approvals::default()`), so tests that step `pages::step` alone keep working on jig's cards. If `sheet::step` can't take it cleanly, route `Act::Needs` in `route.rs` as `Act::Away` is, and say so.
- The handle: `me.rs`'s `APPROVALS: u64 = 7`, after `TRIAL_TASK`.
- `Row::since`'s type: whatever `pages::inbox::waiting_since` answers, so the view says its age as it does a card's; converted from the wire's nanoseconds.
- `kind`: `forge::forges::Kind`, from the wire's `Kind`.
- The rows' state: `enum { Unknown, Rows { rows, more }, Ended }` inside `Approvals`, as `forges::State` is; `rows()` empty for all but `Rows`.
- Its words: none here. The domain has no words for an approval; the view's are `approval-card`'s.

## Steps

1. `forge/approvals.rs`, after `forge/trials.rs`' shape: `watch(limits) -> Option<Watchable>` (`Watchable::Family { scope: c::Scope::Own, window: c::Window { rows: 0, after: None }, record }`, the record `wire::watching` of `w::Watching { watch: w::Watch::Approvals }`); `configure(side, limits, out)`, called in `route.rs`'s `forge_me` after `trials::configure`; `told(...)` routed from `forge.rs`'s `told` as `trials::told` is; its document read by `wire::decode_approvals` after the family and version checks, each `w::ApprovalRow` into a `Row`. `Side` gains its `approvals` and its open flag; `side.changed()` on each delivery.
2. `pages/needs.rs`: `Need`, `Need::object`, `Session::needs`, and `shown` and `count` taking the approvals (the contract). `count` keeps its counting of jig's rows as it is and adds the rows whose `project` is the one asked, `cut` where either is cut.
3. `pages/home.rs`, `facts`: `Needs::Count { count: needs(inbox, approvals).len(), cut: inbox's cut or approvals' more }`.
4. `needs::act` (option above): `Later` and `Open` find the object among `needs(inbox, approvals)`.
5. `forge/ways.rs`: `proposals`' `needed` takes `c::Setting::Application { kind: 1 | 2 | 6 | 7, .. }`; `proposal_values` (or the answer's handler) keeps the answer's `owned_approval` on its `ProposalRead`; `Side::owned_approval(number)` answers it, false while unread.
6. Tests, each biting:

   ```rust
   // tests/approvals.rs, after tests/trials.rs' helpers (opening, deliver, closes):
   #[test]
   fn approvals_are_watched_once_for_the_session_and_end_with_it() {
       // after me: one Own-scoped approvals watch; its rows arrive and read back field by field;
       // a document with a trailing byte ends the watch, no rows, not reopened;
       // sign-out closes it; the next sign-in opens it again.
   }

   // tests/pages/needs.rs, on a root (as count_root does), with approvals delivered:
   #[test]
   fn needs_take_the_approvals_after_the_inbox_by_its_filter() {
       // (filter, inbox rows, approval rows (project, goal), later, expected objects in order)
       // unfiltered: inbox cards, then rows; project 1: its rows alone; goal 9: rows with goal 9;
       // kind question: no rows; an approval marked later goes last; inbox not known: nothing.
   }
   ```

   `standing_count_filters_projects_subtracts_answered_keeps_later_and_reports_cut_and_end` gains rows: an approval of the project counts, one of another project doesn't, `more` above 0 cuts. `later_moves_last_keeps_its_count_and_advances_shown`, or a row of the new table, holds Later on an approval card. `tests/pages/home.rs`' `needs_count_falls_when_answered_and_keeps_later_cards` gains an approval row in the count. `tests/forge_ways.rs`: a proposal naming only a kind-2 rule is read, and `owned_approval` answers `true` and `false` as its answer says. Bites: drop the goal filter, see its row fail; drop kinds 1 and 2 from `needed`, see the read row fail; restore.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain`; the `cargo check` of acceptance 7; `heavy cargo nextest run -p temper-client-view` for the forced edits. Commit ("temper-client-domain: watch the approvals and merge them into Needs you, and read a proposal's rules for owned_approval"), push.

## Report

As the implementer's text says, with each rule broken for a bite and the failure seen, and each forced edit in the view by file.
