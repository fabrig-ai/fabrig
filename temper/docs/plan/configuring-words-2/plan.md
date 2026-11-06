# configuring-words-2: plan

What a real walk of the configuring chat still found. The request is `inputs/request.md`; the words are `notes/configuring.txt`. temper/ui started this and owns the configuring chat, so the calls below are decided here. Nothing here was run on a real model: `walk = false`, and temper/qa rewalks `rp3` after it lands.

## What was found

**1. "Nothing has changed": the stock words are the source, not a tool.** Read, not reproduced on a model:

- **No tool's result says it.** `propose_settings` answers `proposed as tN/K to the deployment's owners`; outcomes are `accepted: revision N made`, `rejected: REASON`, `withdrawn as stale: the newest is N`; a note's write gives its revision and lines. `pending-proposal`'s second story pins this.
- **The pages say it as a fact, on request:** `chat.md`, `charter.md`, `proposal.md`. Nothing tells a run to report it.
- **The instruction can't hold, as written.** "Once, where it may seem so, e.g. a pending proposal" names an example and no place, so each reply decides again; a run can't tell whether "may seem so" applies, and before any proposal nothing rules it out.
- **The likely driver, inferred:** the document rule's "Say in one sentence what changed" sits beside a charter that opens "Change nothing yourself". After a note's write, a reply must say what changed, and of the settings, nothing did.

The fix gives it one place (the reply that first gives a proposal's number, or when they think something applied), forbids it before a proposal and as a reply's close, and makes the document rule say "what changed in it". Only a rewalk shows whether that holds.

**2. Withdrawing: the design settles it, and the design was wrong about the way out.** `temper/docs/design/domain/agent.md`, 8.1, has the agent never call `withdraw`: it's in the decide family, which policy 1 doesn't give (`protocol/configuration.md`, 13.4). So the reply says how a person does it: an owner rejects it under Needs you, with a reason the chat hears.

That section also said a proposal it no longer wants "is replaced by a fresh one". **It can't be.** `propose_run` preflights `proposal_problem`, and `admission_problem_staged` refuses a task that holds a pending proposal, answering `Refused: busy: a proposal is pending (at most 1 at once): wait for its decision, or withdraw it`. So a run told to change or withdraw a waiting proposal met two refusals with no way on, the second of which it had been told to try. This cut corrected 8.1, and `pending-proposal`'s first story reproduces it in the lowest tier.

Granting the decide family was rejected: it would also let a chat decide proposals and escalations, across every chat under policy 1. Splitting `withdraw` from it, or letting a fresh proposal replace a waiting one, is jig's and beyond this request's "temper".

**3. Small slips.** "the deployment operator" comes straight from the stock's "the operator configures them". "the stock settings" comes from the pages it reads, which rightly say it, so the stock now names both as words not to say. `How PROJECT works` can't be filled: a brief gives the project's number and nothing gives a run its name. The title is now the literal `How this project works`, which `configuring_chat.rs`'s story already wrote.

`DATE`, in `_Draft, DATE, with the user._`, is the same fault, found by reading: no brief or smith text gives a run the date. It goes from both `configuring.txt` and `chat.txt`, the one sentence in this plan beyond the configuring chat, because the two share the rule and 8.3 says so.

**4. The admin's words.** The steps sat behind "Then:", after a refusal's sentence, with no one named for adopting. They now apply whenever someone is to approve or join, before or after acceptance, which also covers the teammate of item 3. Each step says who does it, and adopting again is the person's, on Settings, Repositories, "Adopt a repository". `pending-proposal` proves that page does it, which the previous cut left unconfirmed.

## The words had no room

**Charter 3 can't grow.** `the_stock_brief_with_a_whole_spec_fits_both_receiving_bounds` holds its host part at or under charter 1's: 2 bytes of room today. The budget is the user's, so nothing moves, and the words are 31 bytes shorter than before. These went, each because a tool or page the run already has says it:

| Cut from `configuring.txt` | Where it still is |
|---|---|
| `first: 1-based start line; remove: …; body: …` | `note`'s schema |
| `Use join true to continue the line…` | `note`'s schema |
| `Ways: drafts, …`, `way starts goals…`, `Questions: none, foreman or up…` | the read itself, and `settings-fields` |
| `writes of a few KB, first/remove, join true…` | `propose_settings`' `note` description |
| `An owner accepts it whole, one revision.` | `propose_settings`' description |
| `Keep "How PROJECT works" separate.` | `propose_settings`' description |
| what each page of `read_docs` is for | the page names, and `read_docs`' index |

**For the rewalk to watch:** a document built in parts now rests on `note`'s schema for its line arguments, and a project's ways on `settings-fields` for their values.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| words | `stock-words` | none | the words, the failing tests, the figures, `host-tools.md` |
| pending | `pending-proposal` | none | four stories that what the words say is so; `proposal.md` |

Both start at once. **Critical path:** either alone, about an hour. Neither reads the other's code: `configuring.rs` keeps its phrase `adopt again, reread inventory and propose afresh`.

## Overlaps I couldn't avoid

- **`stock/configuring.txt`, `tests/stock.rs`, `tests/stock_brief.rs`:** `model-catalogue-1/configuring-told`, `stock-seven` and `model-check/trial-charter` move the same figures, and `configuring-told` adds a rule to the same words. Charter 3 has 31 bytes of room after this plan, so **`configuring-told` will have to shorten too**; a catch-up regenerates the figures rather than hand-merging.
- **finding-400,** in the same words and charter, isn't on a branch yet. It's taken by catch-up and not fixed here; its sentences stay beside ours.
- **`temper/tests/client/tests/suite/configuring_chat.rs`:** `model-catalogue-1/configure-picker` also adds there. This plan appends one story.

## Left, and named

- **jig's busy refusal still says `or withdraw it`** to a run that can't. jig is out of this request; the stock tells the run otherwise.
- **`withdraw` is still offered to charter 3,** and refused by its authority: whether a charter declares only what it names is `agent.md`, section 10's open question.
- **The chat's card has no Reject, and "Change something" can't revise a proposal that waits.** Draft 5.2 draws both. It's recorded in `draft/configuring.status.md`; making it good is the client's and jig's, and the rewalk may show it's worth doing, since a person pressing "Change something" now gets words instead of a revision.
