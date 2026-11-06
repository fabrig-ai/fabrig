+++
id = "chat-walk"
part = "prove"
kind = "docs"
needs = ["chat-decides"]
files = [
  "temper/docs/plan/ui-chat-accept/walk.md",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 50
+++

# The configuring chat's whole path in a browser

## Goal

Prove the fix as a person meets it: on a fresh fake box, at 1440 and at 420, the cue, the card, Accept, the rail changing, and the chat still answering afterwards. No product code is expected; a fault the walk finds is reported, or fixed under the option below.

Acceptance:

- At each width, on its own fresh fake box (`--fake-backends`), a screenshot and a page's text for each of:
  1. ChatGPT first run finished; Settings, Start; the cue "Show drafts and goals for this project." sent, before any settings change.
  2. The card, pending, with Accept and "Change something", and the rail's Needs you count at 1.
  3. Accept's sheet: Escape leaves it with no change; Accept again; the primary.
  4. The sheet gone, no refusal; the card says "✓ Accepted · new work follows it"; the count gone; the rail has Goals, and Chats has its Drafts filter.
  5. A further message in the same chat, answered by the fake model: a new reply below it, in whatever words its script (`temper-fake`'s `model_document`) gives.
- At each width, on another fresh box, Reject from Needs you with a reason still completes, and what the chat's card then says is recorded.
- `temper/docs/plan/ui-chat-accept/walk.md` holds a row per step and width: seen, and its picture's path. Screenshots are in `~/src/fabrig/ui-chat-accept/`, outside any worktree.
- `README.status.md`'s **Not yet** entry for 3.6, *The sheet*, says "and Accept on a configuring chat's proposal from *Needs you* and from the chat itself, by hand at each width, which no test keeps".

## Inputs

- `temper/docs/plan/ui-walk-3/driver/README.md`, how to build and run; `again.py`'s chat follow-up is B1's reproduction, `cdp.py` the browser.
- `temper/docs/plan/ui-walk-3/findings.md`, B1 and W3.
- `temper/docs/design/client/README.md`, 5.13, *The card in a chat*; 3.6, *The sheet*.
- `temper/docs/plan/ui-chat-accept/notes/common.md`, *Render proofs*.

## Owns and may change

Owns: `temper/docs/plan/ui-chat-accept/walk.md`. May change: the one entry of `temper/docs/design/client/README.status.md` above, which departs from the implementer's text because the walk is what changes it.

The driver stays as it is: a throwaway script of yours, built on it, is untracked and gone before the push.

Options, each pre-approved:

- If a step's fault is in the client, small and within `temper-client-domain`'s `pages/chat.rs` or `temper-client-view`'s `cards/proposal.rs`: fix it with a test red first, and walk that step again.
- If the chat doesn't answer after acceptance: read the box's log and `temper ctl task activity` first. A startup fault after acceptance (`Agent(Unstarted)`, a resume refused) is `finding-400`'s (`notes/common.md`): if it has landed, catch up and walk again; if not, keep the logs in `~/src/fabrig/ui-chat-accept/`, walk everything else, report `Done:`, and name it under `## Left`. If the cause is another one, it's a `Question:`.
- A "key mark caught a moving page" is retried up to three times, each on a fresh box; then widen the mark's wait in your script, and say so.

## Steps

1. Build as the driver's README says, through `heavy`, one command at a time.
2. Write the script; walk 1440, then 420 by `Emulation.setDeviceMetricsOverride`; then the Reject boxes.
3. Write `walk.md`, edit the status entry; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Delete the script. Commit, saying why, and push.

## Report

As the implementer's text says, and: the builder and the screenshots' folder; each difference from the mock's `chat.html` you saw on these states; what the chat's card says after a rejection; whether `finding-400` was met.
