+++
id = "client-ways"
part = "acts"
kind = "feature"
needs = ["ways-wire", "client-views"]
files = ["temper/crates/temper-client-domain/src/forge/stock_ways.rs", "temper/crates/temper-client-domain/src/forge.rs", "temper/crates/temper-client-domain/src/pages.rs", "temper/crates/temper-client-domain/src/acts.rs", "temper/crates/temper-client-domain/src/tests/forge_stock_ways.rs", "temper/crates/temper-client-domain/src/tests.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 180
+++

## Goal

Give Ask temper's page the project's stock ways and their acts in the
client's domain: watch `ways` while the page is open, offer "Use this",
"Keep it" and "Yours" by 5.13's rules, and send each act as one jig
`ChangeSettings` of the read's items, so that the look request's plan
(`look-ask-ways`) only draws them (`temper/docs/design/client/README.md`,
5.13, *A way's acts*).

Acceptance:

- `forge::stock_ways` (the name `forge::ways` is kind 7's): the watch of
  `Watch::Ways(WaysSubject { project })` in the project's scope, as
  `forge/changes.rs`'s `watch` is made, opened when `Address::Ask {
  project }` opens and released when it closes; its `WaysPanel` decoded
  (`decode_ways_panel`), checked for its project, kept as `StockWays`;
  an ended or undecodable watch kept as ended.
- `Pages::stock_ways(&self) -> Option<&StockWays>`; `StockWays::tabs()`:
  each way in the read's order, its index, name, line, path and
  `current`, then, where `current` is none, `Yours`, current, its path the
  read's `own`; `StockWays::act(index) -> Option<WayAct>`: `Use` on a way
  that isn't current, `Keep` on the current way while To begin's "Tell
  temper how you work" is left (Home's facts, `configured` false, as
  `pages/home.rs` computes it), none on `Yours`, none where `may_change`
  is false, none for every way while an act is out.
- `pages::Act::UseWay { project, way: u32 }` and `pages::Act::KeepWay {
  project }` send at once jig's `Ask::ChangeSettings { against, items,
  running: Running::Stays }`: `against` the newest revision of the
  session's `settings` watch, `items` the way's `items` as jig's
  `SettingItem`s (`Setting::Application { kind, scope, number }`, `Set`
  of `SettingValue::Application(value)` or `Removed`), as `forge/ways.rs`'s
  `send` builds its one item. A `stale` refusal is sent once more against
  the revision it names; another refusal is kept on `StockWays` as jig's
  refusal, for the page to say, until the next act or a newer document;
  an answer clears it. An act whose way is not offered that act sends
  nothing.
- A way's `delegation` adds nothing here (task `ways-delegation`).
- Tests (`src/tests/forge_stock_ways.rs`): the watch's scope and life with
  the page; `tabs` with a current way and with `Yours`; `act`'s table
  (not current, current with the step left and done, `Yours`,
  `may_change` false, an act out); `UseWay` sends one `ChangeSettings`
  with the way's items in order, the newest revision and `Stays`; a
  `stale` retried once; a second `stale` and a refusal kept; a document
  of another project ends the watch.
- The gallery is `views-draw`'s: its fake world leaves the new watch
  unanswered (its arm is `ways-wire`'s), and its Ask samples keep their
  checks.
- `temper/docs/design/client/README.status.md`: the sentence "5.13, *A
  way's acts*: Ask temper's page watches no `ways`, and has no "Use
  this", "Keep it" or "Yours"." becomes "5.13, *A way's acts*: a way's
  `delegation` adds no policy item."

## Inputs

- `temper/docs/plan/ui-ask-m3a/notes/common.md`; `notes/contracts.md`,
  sections 3 and 6 (binding).
- `temper/docs/design/client/README.md`, 5.13 (*A way's acts*, *Its
  acts*), 3.5 (*To begin*'s second step, *Ask temper's page*), 4.6 (*A
  project's ways*); `temper/docs/design/protocol/configuration.md`, 13.10.
- The patterns: `forge/changes.rs` (a project's watch) and
  `pages/changes.rs` (its life with a page); `forge/ways.rs`'s `act`,
  `send` and its refusal handling; `pages/home.rs`'s `configured`.

## Owns and may change

- **Owns:** `forge/stock_ways.rs` (new) and its tests; the two acts and
  the page's watch in `pages.rs`.
- **Neighbour edits:** `forge.rs`'s `mod` line; `acts.rs` if acts are
  listed there.
- **Options, pre-approved:** if `Act` variants are mapped to request
  keys or words in another of the domain's tables, add their rows. Do not
  draw anything in `temper-client-view`: the page's look is the next
  plan's; if the compiler asks for an arm there, answer it with nothing
  drawn.

## Steps

1. The watch and its life with the Ask page.
2. `StockWays`, `tabs`, `act`.
3. The acts and their outcomes.
4. The tests; break one rule (offer `Keep` with the step done) to see its
   row fail, and restore.
5. The status sentence.
6. `cargo fmt`; `heavy cargo clippy -p temper-client-domain --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-client-domain`; `heavy cargo check --tests -p
   temper-client-view -p temper-client-world -p temper-client-gallery`.
   Commit and push.

## Report

As the implementer's text asks.
