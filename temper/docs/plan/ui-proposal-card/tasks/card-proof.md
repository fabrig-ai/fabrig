+++
id = "card-proof"
part = "card-proof"
kind = "feature"
needs = ["card-reload", "focus-reveal"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-view/src/cards/proposal.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-fake/src/backends.rs",
]
effort = "high"
minutes = 75
+++

# The render proof, at 1440 and 420, on a fake box

## Goal

The scripted stories didn't show these faults, so a passing story isn't the proof. Drive a fresh fake box through a throwaway render at 1440 and 420: two proposals in one chat, then one rejected from Needs you, then a reload. Fix what it finds. temper/qa walks it on a real model after it lands.

Acceptance, each seen and captured at both widths:

1. **A card's place:** with the fake model's calls not naming the proposal (they don't: no live turn does), the new pending card is the last thing in the chat, showing with no scroll while the person stays at the end, and the reply follows it. The second proposal stands below the first. The tool line and the "Decision made" row aren't confusing it: no "Decision made" beside a card.
2. **A decision from elsewhere:** one proposal rejected from Needs you; going back to the chat shows "Rejected · nothing changes", never "Waiting for an owner", even for a moment you can catch.
3. **A reload:** the card stays at its place with its state line, and "Decision made" rows don't stand beside it. Paging back to the start, if the window has moved, keeps it.
4. **The reason:** a Markdown heading in the reason is rendered, in the card, in proportion: it doesn't outweigh the card's title. Compare the Accept sheet and the mock's `chat.html`.
5. **"Change something":** pressing it with the focus already in the composer scrolls the composer into view, at 420 from a scrolled conversation, and shows "What should change?"; sending brings "Reply to T…" back.

## Inputs

- `temper/docs/plan/ui-proposal-card/notes/common.md`, *Render proofs*.
- `temper/docs/design/client/README.md`, 5.13.
- `temper/docs/design/client/greenfield/chat.html`, the target's look.
- `temper help box`, for a box on fake backends; `temper/crates/temper-fake/src/tests/steps.rs`, near line 60, and `temper/crates/temper-fake/src/backends.rs`, for the fake model's cue "Show drafts and goals for this project.", which calls `propose_settings` and then stops.

## Owns and may change

**Owns:** the `files` above, for what the proof finds, each fix with a focused test of its own at the lowest tier.

**Pre-approved options:**

- **A second proposal from the fake model:** the cue's `against` is fixed at 1, so after an Accept a second run of it goes stale. Prefer driving the two proposals with both pending, the second replacing or standing beside the first as jig says. If the walk needs another shape, add a cue to the fake model's document in `backends.rs`, committed with a test in `tests/steps.rs` only if it is useful beyond this proof; a throwaway edit otherwise, gone before the push.
- **A fault too big to fix here** (more than an hour, or outside the client): record it under `## Left` with its capture's description, and go on with the rest of the proof.
- **Not a tier:** this render, by a throwaway driver, isn't one. Run no tier and no story.

**Not yours:** `docs/design/` (`card-close` closes the entries, and what you learned goes in the report); recordings.

## Steps

1. Build with the recording-free bundle a fake box needs, as `temper help box` says, and start a fresh `temper box --fake-backends` on your builder.
2. Write a throwaway CDP driver, untracked: `Emulation.setDeviceMetricsOverride` for 420, handling `Page.javascriptDialogOpening`. Drive the walk above at each width, on a fresh box each.
3. Capture each numbered state. Screenshots go in `~/src/fabrig/ui-proposal-card/` on your builder, outside any worktree, never committed.
4. Fix what it finds, each with a focused test first where the fault can show there.
5. Delete the driver. If you changed code: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy` of what you changed; `heavy cargo nextest run` of those packages.
6. Commit in small steps that say why, and push. With no code change, say so: nothing to push is a fine end, and the report stands.

## Report

As your text says, and: which builder holds the screenshots and their directory; for each numbered state what you saw at each width; each fault found and its fix; the exact cause, seen on the box, of why a card had no anchor; anything under `## Left`.
