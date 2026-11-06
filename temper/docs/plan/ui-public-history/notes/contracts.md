# ui-public-history: the run's binding shapes

What crosses between `queue-merges` (the forge's projection) and `board-words` (the view). Neither task needs the other:
each builds to this note. The design is `temper/docs/design/client/forge.md`, 11.1 and 11.2, and
`temper/docs/design/client/browser.md`, 8.3.

## A queue merge on the wire

No new record, field or variant: `temper-client-protocol-wire`'s `public-v1` is unchanged, and so is its golden.

- **In `PublicLandings.landings`,** after the landings by turn:

  ```
  PublicLandingLine { members: [each logged change's place], phase: Landed, landed: [the logged commit] }
  ```

  `landed` holds that one commit once, however many changes the log holds for it.

- **As `PublicLanding`,** read at `/public/projects/<project>/landings/<task>` through any of its changes:

  ```
  PublicLanding {
      phase: Landed, hold: None, preparation: 0,
      members: [PublicMember { task: its place, base: None, candidate: None, landed: Some(commit), gates: [] }, ...],
      stale_proofs: 0, holds_turn: false,
  }
  ```

- **`PublicChange.in_landing`** stays false for it: it is a member of no landing.

## What the view draws from it

The view's code for a line and a landing page already draws these records. A line reads `landed · task 3f2a9c01 · 789ada1`,
the task and the commit each a link to `Public::Landing`. The landing page reads `landed`, then the member's row,
`task 3f2a9c01 · Its change: 789ada1`, and `0 stale proofs`, whose line this plan leaves as a landing by turn's form has it.

## The words, as the request writes them

| Word | Text |
|---|---|
| the board's new line, under `PublicBoardLede`, in `look::PUBLIC_LEDE` | `Work that ended before this project went public, or before temper last restarted, isn't shown.` |
| `PublicNoGoals` | `No goals to show yet.` |
| the empty landings, on the board and the landings page | `No landings to show yet.` |

`Word::GoalsNone`, the signed-in Goals page's "No goals yet.", is another word and does not change.
