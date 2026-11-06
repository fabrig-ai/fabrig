+++
id = "card-close"
part = "card-close"
kind = "docs"
needs = ["card-proof"]
files = [
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/journeys/**",
  "temper/docs/design/client/mockups/STATUS.md",
  "temper/docs/design/STATUS.md",
  "temper/docs/plan/ui-proposal-card/plan.md",
]
effort = "medium"
minutes = 40
+++

# Close the status entries and record the journeys anew

## Goal

The plan's code is in. Bring the design's status up to date with it, and record the journeys this plan's wire and exchange changes made stale, once, since each task left them for here.

Acceptance:

- These **Not yet** entries in `temper/docs/design/client/README.status.md` are gone, where the code now matches: 5.13 *The card in a chat*'s reason and "Change something"; *Where a card stands*; *"Decision made"*; a decided or read-less card's state lines; *A decided card after a reload*.
  **Departure from your text:** these status files are yours. The design documents aren't: if a section turns out wrong, report it.
- The entry that no live turn names its proposal stays, reworded only if `card-proof` showed otherwise.
- Any journey whose recording went stale is recorded anew and its print refreshed, by `again NAME`.
- `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing new broken.

## Inputs

- `temper/docs/plan/ui-proposal-card/notes/common.md`, *Recording flakes*.
- `DESIGN.md`, 2, on status files.
- `temper/docs/design/client/journeys.md`, 8 and 8.1, on a recording gone stale and `again`.
- The reports of this plan's tasks, as their branches' commits and `plan.md` show them.

## Owns and may change

**Owns:** the `files` above. **Departure from your text:** `docs/design/` status files and the journeys are yours, and `again` drives a browser on a fake box as a recording's own tool: it is no tier.

**Readers, each edited only as said:**

- `temper/docs/design/client/journeys/*/README.md`, only if a walk's steps changed.
- A help page isn't yours unless a generated reference went stale.

**Pre-approved options:**

- **A journey that makes no proposal** may not change: `again` tells. Don't re-record what isn't stale.
- **"a key mark caught a moving page":** retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so.
- **A walk that no longer holds** (a control renamed): edit its setup (`journeys.md`, 8.2); a change of the walk itself is walked anew by a throwaway driver on a fresh fake box.
- **An entry you can't honestly close** (the code doesn't match): leave it, and report why.

**Not yours:** code; `README.md`'s text.

## Steps

1. Read each entry against the code and close it; `temper/docs/design/STATUS.md`'s summary if its paragraph names any.
2. `heavy cargo run -p temper-client-journeys -- again NAME` for each stale journey; the print's diff read, holding only what this plan changed, times and loopback ports.
3. `heavy cargo nextest run -p temper-client-journeys`; the citation check.
4. In `plan.md`, nothing but a last line saying the plan closed.
5. Commit in small steps that say why, and push.

## Report

As your text says, and: each entry closed or left, with why; each journey recorded anew or found fresh, with its print's diff in a line; anything under `## Left`.
