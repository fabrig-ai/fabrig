+++
id = "red-news-surface"
part = "surface"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/red.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/topics.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs"]
effort = "high"
minutes = 90
+++

# A red's news: its type and its words

## Goal

The forge's `News` gains `Red(RedNews)`, and the root words it for the goal's inbox, both forms
(a red after landing; a red of a tier following goal branches), cut to the inbox's bound.
Nothing publishes it yet (`red-news-tell` and `goal-tier-forge` will).

Acceptance:

- `RedNews`, `RedOn` and `JobLog` exactly as `notes/contracts.md`, section 5 has them, in a
  new module `temper-engine-domain-forge/src/red.rs` (its doc cites
  `temper/docs/design/domain/after-landing.md, 7`), exported from the crate's root as `News`
  is; `News` (`boundary.rs`) gains the variant `Red(RedNews)`, last.
- The forge's classifications (`topics.rs`, `classify`; `domain.rs`'s landing classification
  near `publish`) answer `Class::Wakes` for `News::Red`: its publisher sets its class, and
  these arms are compiler-forced only.
- `news_words` (`forge_route.rs`) words `News::Red` as contracts.md, 5 writes, both forms,
  within its `limit`: the header and the test lines first, whole lines only; then `Run: URL`;
  then the logs, the room left shared equally among them, each keeping its end.
- Unit stories in `forge_route/tests.rs`: a red after landing of two tests with a run's link
  and one job's log; a red after landing of `Failures::Many`; a goal branch's red of one new
  and one still failing test; a job whose log could not be read (`[no log]`); a limit that
  leaves room for the header and ids only (no `Run:`, no log), and one that cuts a long log
  from its start, keeping its last bytes.

## Inputs

- `temper/docs/plan/reference-m9/notes/common.md`; `notes/contracts.md`, section 5.
- `temper/docs/design/domain/after-landing.md`, section 7 (the news's words, a member's goal, a
  red of a tier following goal branches).

## Owns and may change

- Owns `red.rs` (new), the `News` variant in `boundary.rs`, the export line in `lib.rs`,
  `news_words` and its tests.
- `topics.rs` and `temper-engine-domain-forge/src/domain.rs`: the compiler-forced `News::Red`
  arm alone. Any other `match` on `News` the compiler names in temper: its arm alone.
- `store-surface` edits `boundary.rs` and `lib.rs` at the same time in other places: keep your
  lines beside `News` and the crate's exports, and touch nothing else there.

## Steps

1. `red.rs` and the variant; the forced arms.
2. `news_words`'s arm, with a helper per form; the cut as acceptance says.
3. Tests: `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(/news/)'` (name your
   stories so the filter selects them), then `-p temper-engine-domain-forge --lib`.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-domain
   --all-targets -- -D warnings`; `heavy cargo check -p temper-engine-forge-world --tests`;
   commit and push.

## Report

As the implementer's text asks.
