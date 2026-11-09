+++
id = "stop-asking"
part = "client"
kind = "feature"
needs = ["needs-why", "keep-asking"]
files = [
  "temper/crates/temper-client-domain/src/pages/stop_asking.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/needs.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/forge/dismissed.rs",
  "temper/crates/temper-client-domain/src/forge/ways.rs",
  "temper/crates/temper-client-domain/src/tests/pages/needs.rs",
  "temper/crates/temper-client-domain/src/tests/dismissed.rs",
  "temper/crates/temper-client-view/src/pages/needs.rs",
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/pages/inbox.rs",
  "temper/crates/temper-client-view/src/shared/mod.rs",
  "temper/crates/temper-client-view/src/shared/reach.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/needs.rs",
  "temper/crates/temper-client-view/src/tests/toast.rs",
  "temper/crates/temper-client-view/src/tests/inbox.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/**",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/needs.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 150
+++

# Stop asking and Keep asking on a need's reason

## Goal

A need whose reason asked offers **Stop asking** on its line: opened in place, the kit's offers for its reach, each drawn as the narrowed table's change in words; an owner of the project applies one with one press (`ChangeSettings` of kind 8), anyone else proposes it as their own (jig's `Propose`); the need in hand still waits. **Keep asking** keeps a `KeepAsking` fact, and the line stops offering Stop asking for that subject for 7 days (`temper/docs/design/client/README.md`, 3.5, *Needs you*, *Stop asking* and *Keep asking*).

Acceptance:

1. **Offered,** `pages::stop_asking::offered(root, need, wall)` (sketch): `Some` of the kit's `offers(classes, reach)` where `needs::why` gives `Why::Reach` whose reach asked, `Side::classes(project)` is read, `offers` is not empty, the person's dismissals are delivered (`forge::dismissed`'s state), no Keep asking of that project and the reach's subject is out, and none is held whose `at` is less than 7 days before the wall; `None` otherwise. The subject is `Subject::Kind(reach.kind)` for a reason `Kind`, `Subject::Set(n)` for `Set(n)`.
2. **Open,** `Act::Open { object }`: opens that need's offers, or closes them if open; one is open at a time. It closes on another card's `Open`, on the card's answer, Later or Undo, on leaving the page, and once Keep asking is sent. `Pages::stop_asking()` holds the open one: its object, the `Words` field's words, the act out and its refusal (3.5, *What the client keeps*).
3. **Apply,** an owner's (`me`'s project row for the card's project `HeldRole::Owner`), `Act::Apply { offer }`: the kit's `narrowed(classes, reach, offer, words)`, then `forge::ways`' set path with `Asked::Classes(table)` for that project, so that it sends jig's `ChangeSettings` of one item of kind 8, running work staying, against the newest revision, and sends a `stale` refusal once more. A refusal of `narrowed` sends nothing and says so as a refusal under the offers. Answered: the toast "Applied: the next one is decided and listed. This one still waits for you." and the offers close; the value asked shows in the project's classes slot at once (the open project's or `needs-why`'s card slot).
4. **Propose,** anyone else's, `Act::Propose { offer }`: jig's `Ask::Propose { project: 0, action: ProposeAction::Settings(ProposeSettings { against, items }), reason }`, `against` the newest revision the person has (as the set path takes it), `items` the one `SettingItem` of kind 8, scope the card's project, number 1, set to `narrowed`'s table as `forge::ways` encodes kind 8 (`classes_record`), `reason` "Stop asking in PROJECT: let agents decide OBJECT, then tell whoever they would have asked." (OBJECT `shared::object`'s). Answered with a proposal: the toast "Proposed: it applies once an owner accepts it. This one still waits for you."; refused: jig's words under the offers, as a settings row's refusal.
5. **Keep asking,** `Act::Keep`: `forge::dismissed::ask(PersonFact::KeepAsking { project, subject, at: wall })`, one request; the client's `PersonFact` gains `KeepAsking { project: u32, subject: temper_protocol_reaches::Subject, at: Wall }`, decoded and encoded through the wire's `KeepAsking` (`subject_from`, `subject_record`), and `Side::keep_asking(project, subject) -> Option<Wall>` answers the newest held `at`. Answered: the toast "It keeps asking. Stop asking comes back in 7 days."; refused or lost: "temper couldn't keep that. Try Keep asking again."; unknown: nothing said, the subject answers it.
6. **Drawn** where `needs-why` draws the line: "Stop asking" at its end, a button in `.link`'s look, `State::Expanded` while open, bound to `Act::Open`; open, `temper-why-offers` under the line (a new look name, `look::WHY_OFFERS`): a row an offer in the kit's order, its words by `shared::offer` (sketch) as 3.5 words them, at its right "Apply" or "Propose" (`.btn.small`), the `Words` offer's field "Which changes" (placeholder "fixes that keep the wire format") over its row, its act disabled until it holds 1 to 120 bytes; then "Keep asking" (quiet, small). Every act disabled while one is out; a refusal under the rows. In the terminal: the same as numbered controls. Rules under `/* ==== ui: needs ==== */`.
7. **The toasts** are `needs::Toasted`'s new variants, drawn in `frame.rs` as the others are.
8. **Rows:** one new table test in the domain, beside `src/tests/pages/needs.rs`'s (a new behaviour): `offered` (a reach asked: offers; decided, fixed kind, `Always`: none; classes unread: none; dismissals undelivered: none; a Keep asking 6 days old: none, 8 days old: offers; another subject's: offers); `Open`'s closing rules; Apply's sent `ChangeSettings` item and its value, Propose's sent `Ask::Propose` whole (project 0, the one kind 8 item, the reason's words), Keep's `Dismiss`; owner and member choose Apply and Propose; in `src/tests/dismissed.rs`'s fact tables (about lines 107 and 200), `KeepAsking` delivered and asked; in the view, rows of existing tests: `src/tests/needs.rs`'s parts table (the link, its expanded state, each offer's words and act for an owner and a member, the field's disabled act, Keep asking), `src/tests/toast.rs`'s toast table (the four words), `src/tests/inbox.rs` the terminal's numbered offers, `src/tests/look.rs` the new name.
9. **The gallery** (rows only): `needs-stop-asking` (an `implementation` reach, an owner, "Apply") and `needs-stop-asking-set` (a reach of a set with two patterns, one matched, a member: the words field, the split, "Propose") (`temper/docs/design/client/browser.md`, 7.2), built in `temper/tools/gallery/src/needs.rs` with the classes delivered, opened by `Act::Open` as a state's act; in `samples.rs`, `names.txt` and `each_state_holds_what_its_name_says`.
10. **Generated files:** terminal goldens that now show "Stop asking" regenerated by their test with `SKEIN_CODEGEN_WRITE=1`, each named.
11. **Runs,** counts in the report: `heavy cargo nextest run` for `-p temper-client-domain`, `-p temper-client-view`, `-p temper-client-gallery`, `-p temper-client-world`, `-p temper-client-tui`, `-p temper-client-shell-terminal` and `-p temper-client-journeys`; `heavy cargo check --tests -p temper-client-browser-world -p temper-one-process-world -p temper`; `heavy cargo check -p temper-client-shell-web --target wasm32-unknown-unknown`; clippy on the packages changed, and on `temper-client-shell-web` for wasm32.
12. **Status:** delete the **Not yet** entries naming `ui-reaches/stop-asking` in `temper/docs/design/client/README.status.md` and `browser.status.md`.

## Inputs

- `temper/docs/plan/ui-reaches/notes/common.md`, all of it.
- `temper/docs/plan/ui-reaches/notes/contracts.md`, section 1 (`Offer`, `offers`, `narrowed`, `Subject`, `Refusal`), section 3 (`classes_record`, `subject_from`, `subject_record`) and section 5 (`KeepAsking` on the wire).
- `temper/docs/design/client/README.md`, 3.5: *Needs you* (*Why it reached you*, *Stop asking*, *Keep asking*), *What the client keeps* (*Stop asking*), *Home*'s *To begin* (*"Skip"*, the same `Dismiss` flow); 4.6, *A person's facts, the subject `dismissed`*; 5.13, *Its acts*.
- `temper/docs/design/domain/forge.md`, 5.5 (`KeepAsking`, a later one replacing the earlier) and 5.10 (*Stop asking*).
- jig's `jig/docs/design/client/domain/contract.md`, 3.5 (*Proposing is explicit*) and the `Propose` row of 3.2's table; jig's `jig/docs/design/domain/people.md`, 5.1 (`Propose`, a change of settings, in the deployment's scope: a role in the deployment or a project) and `jig/docs/design/domain/engine.md`, 17.6. A settings proposal names project 0 (`jig/tests/conformance/tests/suite/client_settings_proposals.rs`).
- The code: `temper-client-domain`'s `pages/home.rs`, `skip` (the `Dismiss` flow to copy), `forge/dismissed.rs` (`PersonFact`, `ask`, the fact queries), `forge/ways.rs` (the set path), `pages/switcher.rs`, `owned` (a project's owner from `me`), `pages/needs.rs` (`Toasted`, `say`, the closing points); the view's line from `needs-why` and `shared/reach.rs`.

## Owns and may change

Owns the `stop_asking` module and its acts, the client's `KeepAsking` fact, the offers' drawing and words, the toasts, the rows and states, the goldens regenerated, and the status entries.

Sketch, binding at its outer layer:

```rust
// temper/crates/temper-client-domain/src/pages/stop_asking.rs, `pub mod stop_asking` in pages.rs, `Act::StopAsking(stop_asking::Act)` in pages' `Act`
//! Stop asking and Keep asking on a need's reason (temper/docs/design/client/README.md, 3.5, Needs you).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act { Open { object: ObjectKey }, Words { words: Box<[u8]> }, Apply { offer: temper_protocol_reaches::Offer }, Propose { offer: temper_protocol_reaches::Offer }, Keep }
#[derive(Debug)]
pub struct Open { /* the open need's object, project, reach, words, out, refusal: the implementer's */ }
impl Open {
    #[must_use] pub fn object(&self) -> ObjectKey;
    #[must_use] pub fn words(&self) -> &[u8];
    #[must_use] pub fn busy(&self) -> bool;
    #[must_use] pub fn refused(&self) -> Option<&jig_client::contract::Refusal>;
}
impl Pages { #[must_use] pub fn stop_asking(&self) -> Option<&Open>; }
#[must_use] pub fn offered(root: &crate::Domain, need: super::needs::Need<'_>, wall: Wall) -> Option<Box<[temper_protocol_reaches::Offer]>>;
#[must_use] pub fn owner(root: &crate::Domain, project: u32) -> bool;

// temper/crates/temper-client-view/src/shared/reach.rs
/// An offer's words: the narrowed table's change (README 3.5, Needs you, Stop asking).
#[must_use] pub fn offer(w: &Words, classes: &temper_protocol_reaches::Classes, reach: &temper_protocol_reaches::Reach, offer: temper_protocol_reaches::Offer) -> Box<[u8]>;
```

May change, each only as said:

- `temper/crates/temper-client-domain/src/forge/dismissed.rs`: an arm `keep-asking` left for the wire's `keep_asking` fact (if it left one) is replaced by the real decoding.
- `temper/crates/temper-client-view/src/frame.rs`: the toasts' arms and the input binding the words field needs, as the reply form's field is bound.
- A test or fixture elsewhere that matches pages' `Act`, `Toasted` or `PersonFact` exhaustively (found by the compiler): the arm, named in the report.

Options, each decided:

- **The set path is keyed to the open project** and a card of another project's Apply cannot go through it: give `forge::ways`' set path the card slot's project as it has the open one's, and say how; never a second sender.
- **The newest revision is unknown** (the `settings` document not delivered): Apply and Propose are disabled, as 5.13's controls are until read.
- **A `Propose` answer that is a refusal naming no proposal could be accepted** (a deployment whose policy names none): say jig's words, as any refusal; no special case.

## Steps

1. Read the Inputs. The client's `KeepAsking` fact and its rows (acceptance 5's fact).
2. `offered`, `Open` and the acts (acceptance 1 to 5) with their table; bite: drop the 7-day bound, see the 6-day row fail, restore; and send Propose under the card's project in place of 0, see its row fail, restore.
3. The view: the link, the offers, the words, the toasts (acceptance 6, 7) and the view's rows; the gallery's states (acceptance 9); the goldens (acceptance 10).
4. The runs of acceptance 11; the status entries; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; commit; `python3 -I -B tools/citations/citations.py --base origin/main`; push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report: `offered`'s rule as written, the open state and its closing points, Apply's path (and the set path's project if it changed), Propose's ask, Keep asking's fact, the words, the rows and their bites, the options taken, the goldens regenerated, and the runs' counts.
