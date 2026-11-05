+++
id = "home-proof"
part = "home-proof"
kind = "feature"
needs = ["home-brief", "home-walk", "status-run"]
files = [
  "temper/crates/temper-fake/src/backends.rs",
  "temper/crates/temper/tests/fake_backends/walk/status.rs",
  "temper/crates/temper/tests/fake_backends/walk.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 90
+++

# The proof: Home beside the mock, and a real walk

## Goal

Home reads as the mock's. Rendered beside `home.html` at 1440, 1000 and 420 wide, full page, and its phone's order, with what differs fixed. A real walk with `temper-fake`: land a change, come back, see the count, the agent's words and the byline.

Acceptance:

- **Screenshots** of every gallery state of this plan's (`temper/docs/design/client/browser.md`, 7.2) and of `greenfield/home.html`, at 1440, 1000 and 420, full page, in `~/src/fabrig/ui-home-status/` on your builder, never committed. Your report names the builder and lists what still differs and why.
- **The phone's order** matches `phone.html`'s Home.
- **The fake model's document** (`backends.rs`, `model_document`) gains a `cued` entry on the status charter's words, whose turn finishes with a sentence, a blank line and a lede.
- **The walk,** `temper/crates/temper/tests/fake_backends/walk/status.rs`, as `walk/goal.rs`: a fresh fake box, a project past To begin; enter Home; a change lands; come back after 30 minutes; the count, and then the agent's words and byline, on the real client. Written so that its wait is only what's needed.
- **What's left in the design's status** names only what truly is; `client/browser.status.md` gains Home's lines if it holds its look's.

## Inputs

- `temper/docs/plan/ui-home-status/notes/common.md`, *Render proofs* and *Reach for the mock*; `notes/contracts.md`.
- `temper/docs/design/client/greenfield/README.md`, `home.html`, `phone.html`.
- `temper/docs/design/client/browser.md`, 2.5 and 7.2.
- The models: `temper/docs/plan/ui-goals/tasks/goals-proof.md` and `box-goal.md`; commit `ff53d07ab4`, which scripted the fake box's goals and walked them.

## Owns and may change

Owns, as `files` lists them, and the goldens by regenerating. May change the view, the stylesheet and the gallery where a render shows a difference, and only there.

Options, each pre-approved:

- **The clock:** the request's "on a fake clock if the tests have one". `temper-client-world`'s has (`home-walk`); a real box hasn't. Real time, 31 minutes, in the background, is the walk's wait for the proof. If that tier can't hold that, the walk's test stops at what a box can show unwaited (the first look, the count at a look after a restart) and the 31-minute run is done once by hand for the proof, its screenshots named in your report; the tier's budget isn't raised.
- **"Over budget is red":** the walk's test goes where `walk/goal.rs` is, its tier's budget unmoved.
- **The mock's words a fixture can't say** (its clock, its numbers): say which in your report; the fixture's own words are enough.
- **A throwaway render script:** yes, untracked, gone before the push.

## Steps

1. **Render** the gallery's states and the mock, side by side; fix what differs; regenerate.
2. **The script,** then **the walk.**
3. **Status lines.**
4. **Check,** the implementer's text; the citations check; then push.

## Report

As `notes/common.md`, Your report, with the builder, the paths and what still differs.

## Rebase (2026-11-05, step merge, foreman ui-home-status/home-proof/foreman-1)

preserve integration’s Home token allocation while retaining the branch’s mapping test
The merge conflicts in `temper/crates/temper-client-view/src/look.rs`.
Task commit `c761ae2307` assigns `HOME_FACTS`/`HOME_BYLINE` tokens 299/300.
Integration commit `bbdb0f14d4` incorporates main’s chat changes and assigns those Home styles 326/327; token 300 now belongs to `TOAST`.
Resolve using integration’s constants and complete `NAMES` ordering, including `temper-home-facts` and `temper-home-byline` at 326/327.
Retain the branch’s `homes_facts_and_byline_keep_their_own_style_names` test and its other task changes.
Verify through `heavy cargo nextest run -p temper-client-view --lib -E 'test(=look::tests::homes_facts_and_byline_keep_their_own_style_names)'` with `TEMPER_BUNDLE=none`; confirm one test runs.
Regenerate affected goldens through their existing tool if the combined token mapping changes them.
