+++
id = "window-story"
part = "window-on"
kind = "feature"
needs = ["history-ctl", "window-seam"]
files = [
  "tools/tiers/run",
  "tools/tiers/test_run.py",
  "temper/scripts/bundle.sh",
  "temper/tests/client-browser/tests/browser_all/browser_pages.rs",
  "temper/docs/plan/browser-budget/figures/window-story.md",
]
effort = "high"
minutes = 70
+++

# The tier's bundle holds a smaller window, and the stories pass it at both ends

## Goal

The request's second lever. Turn on `small-window` in the tier's bundle and fit the reading stories to it, in one commit: the same mechanism at an eighth of the bytes a page draws and a find walks. The stories must keep proving the window, so they pass its bound by more than they do today, not less.

Acceptance:

- `tools/tiers/run`'s temper bundle builds with `--features loopback-navigation,small-window`, and `tools/tiers/test_run.py`'s temper row says the same. `temper/scripts/bundle.sh` takes `--small-window`, beside `--loopback`, in its options and its usage line.
- **The long story's history holds at least three windows, 393,216 bytes of its replies' text,** so it passes the bound by at least two windows' worth. A check in the test says so, from the constants, so that a later edit can't shrink it silently.
- **It passes at both ends.** Reading to the start lets the newest go (`gone(reply_end(messages))`, as today). Reading back to the end lets the earliest go, which is new: a `gone` of the first message's paragraph once the end is back. Each is seen in Chromium, and both come back when read to.
- Each behaviour in `notes/common.md`'s *What stays proved* keeps its step and its assertion, `before-later` to `after-later` among them.
- `earlier_words_stay_put_without_browser_scroll_anchoring`'s history stays inside the window: the bound isn't its subject, and a window dropping lines would change what it proves. Its paragraphs still span its two measurements.
- **The story says why the window is lower,** in a comment where the constants stand: the tier's bundle holds an eighth of a person's window, the mechanism is the same, a person's bundle keeps the whole bound, and a long chat at that size is read by hand (`temper/docs/design/client/browser.md`, 4.3). The comment saying sixteen replies cross a 1-MiB window goes.
- The four stories pass by name, alone, three runs each, and `heavy cargo nextest list --profile browser-temper -p temper-client-browser-world` still lists seventeen.
- `temper/docs/plan/browser-budget/figures/window-story.md` holds three runs of each story before your change and three after, and the long story's CPU split after, as `notes/measuring.md` says.
- The `**Not yet:** 4.3, *The tier's window*` entry leaves `temper/docs/design/client/browser.status.md`.

The targets, which aren't limits: the long story about 12 CPU s or less, `latest_returns` about 6 or less, `earlier_words` about 5 or less. Report what you measure either way.

## Inputs

- `temper/docs/design/client/browser.md`, 4.3, *The tier's window*.
- `temper/docs/design/client/README.md`, 5.15.
- `temper/docs/design/draft/chat-reading.md`, 5.
- `temper/docs/plan/browser-budget/notes/common.md`, `temper/docs/plan/browser-budget/notes/contracts.md`, `temper/docs/plan/browser-budget/notes/measuring.md`.

## Owns and may change

Owns the files in the header. `temper/docs/design/client/browser.status.md` loses the one entry above. `tools/tiers/design.md` names no bundle features, so it stays.

Not this task: the view, the domain or the shell (`window-seam` made the surface); `history.rs`, unless a call it makes needs a longer list of `then`; `face.rs`.

Options, each decided:

- **How many messages:** default eight, each answered by about 50 KB, since a story's cost grows with its messages and the window caps what a page draws whatever a reply weighs. You may choose another split if it measures lower, with the three-window check holding and each reply under the fake's 65,536-byte answer bound. Say what you tried.
- **`POSITION_PARAGRAPHS` and `MIDDLE_PARAGRAPHS`** stay as they are unless a position assertion needs otherwise: they, not the code line, make a reply's height.
- **Each reply gets an end of its own,** `End of reply N.`, with a cue per message, replacing today's shared "Reading paragraph" cue and its repeated "End of reply 1.": the window's foot now falls on a middle reply, and a kept box needs a line it can name. `b"End of reply 15."` goes with it.
- **Reading forward across several windows** takes several later reads, so `see(reply_end(messages))` after one burst of PageDown won't do. Add `read_to_end`, the mirror of `read_to_start`, stepping through the messages. Keep the one read held by `-STOP` while its first burst reaches the foot, and measure `before-later` and `after-later` there. The return to the head before Latest is read the same way, if one burst of PageUp no longer reaches it.
- **A key count or a step** may change wherever the run shows it must; no assertion goes without another proving the same thing.
- **The other thirteen stories** run on this bundle too and can't be run here. `rg with_replies temper/tests/client-browser/tests` and read `browser_call_line.rs`'s `demo_asked` turns: if any of them could hold more than 131,072 bytes in one chat, name it in your report. Expect none.

## Steps

1. Before any edit: three runs of each story, for the *before* arm.
2. The feature line, `test_run.py`, `bundle.sh`. `python3 -I -B tools/tiers/test_run.py` runs directly: it builds nothing.
3. `reading_tier`, its constants, the check and the comment. Then `reading_position`, one story at a time, each run by name as it comes right. The long story's `eprintln!` of its kept boxes is there to read: use it.
4. Three runs of each story for the *after* arm, the long story's split, and the figures.
5. `cargo fmt`; `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-browser-world --lib`; the list above. Delete `target/browser-budget/`, make sure throwaway code is gone, commit and push. The flip of the feature and the stories' change are one commit, so that no commit between has a window its stories don't fit.

## Report

The before-and-after medians with their load, the messages and reply size you chose and why, each step that went or changed and what proves it now, what the split says is left, and your implementation and check minutes apart.

## Answers

2026-11-06 (temper/ui). Neither at first: make the window smaller. 128 KB was only my example. Set document_bytes to 64 KiB in the tier and scale the replies to match, so the history still passes the bound by at least two windows' worth at both ends: about 192 KiB of distinct replies, which fits the fake's 262,144-byte joint limit with its wrappers and also costs less CPU, the point of this plan. Keep the replies distinct. Each reply stays well under the window, so no turn reads as too large. Leave temper-fake alone. Only if 64 KiB hits another bound, such as a minimum on document_bytes or a line's room, extend SCRIPT_BYTES to 512 KiB in this task, say why in the report, and keep 128 KB.

## Rebase (2026-11-06, step merge, foreman browser-budget/window-story/foreman-1)

resolve browser.status.md by keeping incoming public-history status and deleting only the tier-window gap
The merge conflicts only in `temper/docs/design/client/browser.status.md`.
Task commit `d8c8d7ff9027` deletes the two-line `**Not yet:** 4.3, *The tier's window*` entry.
Integration commit `9ad66bd18383` adds the adjacent `**Not yet:** 8.3` public-history entry, causing the overlapping hunk.
Rebase onto the landing target; preserve its public-history entry and all unrelated status text, while retaining the task's two-line deletion.
If the target includes main commit `c68899a3ad6e`, preserve its expanded public-history paragraph, 8.3 departure and revised 8.4 entry instead of restoring their older wording.
No source change is needed for this refusal.
