+++
id = "bx-story-github"
part = "box-groups"
kind = "feature"
needs = ["bx-init", "fr-make-forge", "forge-asks"]
files = ["temper/crates/temper/tests/fake_backends/walk/go_on.rs"]
effort = "high"
minutes = 45
+++

## Goal

The draft's section 8 story "GitHub gone on after a restart, on `temper box
--fake-backends`' fake GitHub": a person's GitHub set up, cut by the box's
restart after the App's conversion and before its installation, is found
unfinished, and Go on (draft 6.4) takes it to the App's installation, then
to a first chat, with no second manifest. Written to build and to the
design, **not run** (common.md); the report says so.

Acceptance:

- A new walk module `temper/crates/temper/tests/fake_backends/walk/go_on.rs`
  and its `#[test]` in `walk.rs`,
  `github_gone_on_after_a_restart_reaches_a_first_chat_on_the_fake_box`;
  `heavy cargo check -p temper --tests` builds it and `heavy cargo nextest
  list -p temper --test process` lists it.
- The story asserts the end state of draft 6.4's "GitHub goes on by
  default": the App's records kept across the restart (the forge's row
  says its App is made and not installed), Go on offered before Start over,
  one manifest hop only in the whole walk, the installation and setup hops
  after the restart, the repository adopted and the first chat answered as
  `github::one_click`'s `finish` asserts.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D1, section 5:
  `Connect` of an existing forge).
- `temper/docs/design/draft/playground-reset.md`, 6.4 ("GitHub goes on by
  default") and 8.
- `temper/docs/design/protocol/setup.md`, section 17 (GitHub's App: manifest,
  conversion, installation; find "17.2").
- `temper/docs/plan/playground-reset-r4/notes/surveys/s4-box.md`, section 6
  (what conversion keeps, and installation from kept records).
- The walk and its harness: `temper/crates/temper/tests/fake_backends/walk.rs`
  (`Walk`, `Act`, `Seen`, `forges`, `wait_forges`), `walk/github.rs`
  (`one_click`, `own_app`, `finish`, `manifest_acts`), `fake_backends/mod.rs`
  (`the_serve_child_reads_retained_effective_documents_across_restart`: a
  second box on the retained sandbox).
- The reports of `forge-asks` (`Connect`'s existing forge, the `forges`
  rows' finished field), `fr-make-forge` and `bx-init` (the fake GitHub
  forge 2 seeded in the effective document).

## Owns and may change

- Owns `walk/go_on.rs` (new).
- Neighbours: `temper/crates/temper/tests/fake_backends/walk.rs`, the
  `mod go_on;` line and the one `#[test]` function calling `go_on::run()`;
  `Walk` may gain a method to run a second box on the same sandbox if it
  has none (a new method only, its other methods unchanged).
- Options:
  - The cut: stop the first box (its `Plan` or a SIGTERM, as the walk's box
    ends) after the conversion's redirect hop answers 302 and the forge's
    row reads its App made (default), before the installation hop is
    followed; the second box runs on the retained sandbox.
  - Go on: the ctl tool that sends `Connect` for the existing forge (the
    engine's `forge-asks` report names it), default; else `forge install
    2` (it builds the installation link from the kept slug: s4-box.md,
    section 6), and say which in the report.

## Steps

1. `go_on.rs`: `pub(super) fn run()`: `Walk::new("walk-github-go-on")`;
   run 1: accounts and forges read (forge 2 offered, not connected), the
   manifest acts (`manifest_acts`) followed up to the conversion's redirect
   only; the forge's row read: App made, no installation; box stopped.
   Run 2, same sandbox: forges read: forge 2 unfinished, Go on and Start
   over offered (the row's fields as `forge-asks` names them); Go on sent;
   the installation and setup hops visited (`Act::Visit`); then
   `finish`'s adoption, key and first chat. Assert the hops across both
   runs: one manifest, one create, one conversion, then one installation
   and one setup.
2. `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D
   warnings`; `heavy cargo check -p temper --tests`; `heavy cargo nextest
   list -p temper --test process`. Do not run it. Commit, push.

## Report

Fab's sections; that the story was not run; the cut and the Go on ask as
written.
