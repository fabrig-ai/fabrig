+++
id = "away-proof"
part = "away-proof"
kind = "feature"
needs = ["goal-asks", "away-walk"]
files = [
  "temper/crates/temper-fake/src/backends.rs",
  "temper/crates/temper/tests/fake_backends/walk/away.rs",
  "temper/crates/temper/tests/fake_backends/walk.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/pages/settings.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/domain/forge.status.md",
  "temper/docs/design/domain/agent.status.md",
  "temper/docs/design/protocol/configuration.status.md",
  "temper/docs/design/protocol/store.status.md",
]
effort = "high"
minutes = 90
+++

# The proof: the mock beside it, and a real walk on the fake box

## Goal

The request's proof, whole: `temper/docs/plan/ui-away/inputs/request.md`, *How the cut shapes it*. What it finds it fixes, within this plan's look.

Acceptance:

- **The walk,** `temper/crates/temper/tests/fake_backends/walk/away.rs`, on the fake box, copying `walk/goal.rs` and `walk/status.rs`: Away on; a scripted goal calls `decided` and carries on; Home shows it; Overturn; the goal asks with `message`, as a question; answer it on Home. And Away off: `decided` says Ask, and the goal's question waits. Add its script to `temper-fake/src/backends.rs`' `model_document`, by a cue as `walk/goal.rs`'s is: the fake's model holds no such turns yet.
- **Renders,** on the builder: `home.html`'s away state against the client's `home-away`, `home-away-one`, `home-away-overturned`, at 1440, 1000 and 420 wide, full page; the switch on and off at each width, `rail-away-on` among them; `settings-away`; toasts open. The client's own fonts are what's compared. Saved under `~/src/fabrig/ui-away/`, outside any worktree; the report says which builder.
- **What the renders find,** fixed: a rule, a word, a token. A difference you keep is named in the report, with why.
- **Every status line this plan owns is closed:** `client/README.status.md`, `browser.status.md`, `domain/forge.status.md`, `agent.status.md`, `protocol/configuration.status.md`, `store.status.md`. What's left says why. `python3 -I -B tools/citations/citations.py --base origin/main` is clean.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md` (*Render proofs*), `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5 (*Away*); `browser.md`, 7.2.
- `temper/crates/temper/tests/fake_backends/walk/goal.rs` (`text`, `show_goal`), `walk/status.rs` (`words`, `show_status`, `show_home`), `temper-fake/src/backends.rs`.

## Owns and may change

Owns, as `files` lists them. May also change: the gallery and goldens for what a fix moves; `temper-fake/src/tests/demo.rs` for the new script; `temper/docs/help/` pages by the rule of `notes/common.md`.

Options, each pre-approved:

- **The real clock:** the walk's switch is on with `until` far off, or none, so no wait passes it: past `until` is `away-walk`'s, on a fake clock.
- **A fake box's cue that can't tell the second question from the first:** cue on the overturn's words, "I overturn your decision".
- **A fault the walk finds in another task's code:** fix it in the lowest tier that shows it, with a test there, and say so: that crate is yours for that fix.
- **Over budget:** does the walk belong in this tier; can it be made faster; otherwise your report says the measurements, as `notes/common.md`'s rule and `AGENTS.md`, 5 say. No budget rises.

## Steps

1. The script, the walk: `heavy cargo nextest run -p temper --test fake_backends -E 'test(walk::away)'`, its full name as the suite has it.
2. A throwaway render script (`notes/common.md`), untracked and gone before the push. The renders, and their fixes.
3. The statuses.
4. The check of the implementer's text. Push.

## Report

As `notes/common.md`, Your report: which builder, the path of the screenshots, what they found and what was fixed.
