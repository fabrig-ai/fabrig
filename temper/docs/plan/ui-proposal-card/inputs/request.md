# ui-proposal-card: a settings proposal's card in a chat, as a real model makes it

temper/qa walked `ui-chat-accept` on a real model (`gpt-6.1-sol`, 1440 and 420 px, `f89ecece19`). Accept in the chat works: three proposals, two accepted, one rejected from Needs you, each read back in ctl. These are what a person met on the way, and I've looked at the screenshots. The scripted stories don't show them, so don't take a passing story as proof; see Proof below.

## What's wrong, and my ruling on each

1. **A new proposal's Accept sits above the old accepted card.** A person stays in the chat and asks for a second proposal. The view rests on the old "✓ Accepted · new work follows it", with the new pending card off-screen above it, so they can think it's done. Likewise, **the chat's reply lands above the card**, off-screen at 420.
   **Ruling:** a card stands in the conversation where it was made, and what comes later comes after it. Keep the anchor at the call that made it, where its turn names it. A card with no anchor today goes after the conversation (`pages/chat.rs`, `proposal_anchor` and the tail around line 173): instead it stands after the turn that was running when it was made. Then a new pending card is the last thing in the chat by construction, and the reply after an Accept follows the card. The chat stays at its end as usual, so the new card shows without scrolling.
   Find out why these cards had no anchor with a real model (the likely reason is that its calls don't name the proposal in `call.objects`) and say so in the report. If they can name it cheaply, do that as well. If a card's making time isn't available when it's decided or re-read, add what's needed: formats may change.
2. **A rejected proposal's card says "Waiting for an owner", "Owners of this deployment", with no acts,** after Reject from Needs you, to the owner who rejected it. Seen once, at 420.
   By my reading, not proved: `settings_proposal` (`cards/proposal.rs`) finds the stale pending proposal in `chat.settings_proposals`, `settings_state` falls through to `Pending`, and with no Accept offered the card takes the waiting branch. Prove the route first.
   **Ruling:** a decision made elsewhere (Needs you, another tab, ctl) reaches the chat's card without a reload. A rejected card ends in a muted state line, "Rejected · nothing changes", the same way an accepted one ends. A withdrawn card with nothing replacing it says "Withdrawn · nothing changes". "Waiting for an owner" shows only while the proposal is pending and this person can't act.
   Un-ignore `an_owner_rejects_a_chats_pending_proposal_under_needs_you` (`temper/tests/client/tests/suite/configuring_chat.rs`). Its ignore reason says jig refuses the rejection, but the real walk rejected from Needs you; if the refusal is still real, fix it here and say so.
3. **Decided proposals vanish from the chat on reload,** even paging to the start: only "Decision made" rows are left.
   **Ruling:** a decided proposal's card stays at its place after a reload and paging back, with its state line, read as the pending ones are. A card and a "Decision made" row never both say it: where the card stands there's no row. This also settles the walk's note that "Decision made" shows too early on a proposal's card.
4. **Literal `## temper enforces` in the card,** where the Accept sheet renders it. **Ruling:** the card renders the reason's Markdown by the path the sheet uses.
5. **"Change something" shows no visible effect:** focus was already in the composer. **Ruling:** pressing it focuses the composer, scrolls it into view, and sets its placeholder to "What should change?". The placeholder goes back once something is sent or the proposal is decided.

"charter 1" stays, as 5.13 says.

## Design

Bring `temper/docs/design/client/README.md` with it: 5.13's "The card in a chat" and 3.5's line on where a proposal's card stands. `chat-reading` is forge's: `chat-reading-c` changes how a chat reads back and may land while you work. Keep to the card's own code, and on a catch-up take its reading window as it is. Older's thin first page isn't this plan's.

## Tests

Focused, in the view, the client domain and the joint world, each shown red without its fix:
- a card with no anchor stands after the turn that made it, a later reply after it, and a second proposal below the first;
- a decision from Needs you reaches an open chat's card;
- the state lines for rejected and withdrawn;
- a decided card after a reload, and no "Decision made" row beside it;
- the reason's heading rendered in the card;
- the placeholder set and restored.

## Proof

A throwaway render at 1440 and 420, on a fake box, driving: two proposals in one chat, then one rejected from Needs you, then a reload. Make the fake model's calls not name the proposal, as a real model's didn't, or the proof won't show case 1. temper/qa walks it on a real model after it lands.

## Sources

- `temper/crates/temper-client-view/src/cards/proposal.rs`
- `temper/tests/client/tests/suite/configuring_chat.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
