+++
id = "small-words"
part = "small-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/stock/configuring.txt",
  "temper/crates/temper-service/src/stock/chat.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-view/src/tests/change.rs",
  "temper/crates/temper-client-domain/src/forge/change.rs",
  "temper/crates/temper-client-domain/src/forge/words.rs",
  "temper/crates/temper-client-domain/src/tests/change.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "medium"
minutes = 25
+++

# Four sentences a person misread

## Goal

Change four things temper says, and nothing else: the configuring agent stops raising a forge identity nobody asked about, a chat looks on the forge before saying a repository isn't there, Home says "Nothing needs you." once, and a repair of a semantic conflict blames nothing.

Acceptance:

- `configuring.txt`'s sentence reads, whole: `An unknown forge identity cannot count for approvals; raise a person's forge identity only when approvals are what's being asked about or proposed.`
- `temper-engine-smith/src/settings.rs` doesn't change: the inventory's fact stays as it is, and `temper/tests/engine/domain/tests/suite/configuring.rs` still reads it.
- `chat.txt` gains this paragraph, directly after the one beginning `For the person's chat:`: `A repository that isn't in your checkout may still be on the forge: call read_branch with its repository before you say it isn't there.`
- Home's empty *Needs you* reads, whole: `Work carries on, and this page tells you what changed when you come back.` The headline's `Nothing needs you.` doesn't change, nor does Needs' or Inbox's own empty line (`Word::InboxNeedsEmpty`).
- A semantic conflict's line reads, whole: `Repairing: CI failed at its updated head.` It has no hole, and names no base.
- The other two repair lines, its chip and its step (`Queued`) don't change. `w::Reason::Semantic` keeps its name and its place in the protocol: only the words go.

## Inputs

- `temper/docs/design/domain/agent.md`, 8.1, *Its instructions*, the first bullet.
- `temper/docs/design/protocol/configuration.md`, 13.4, the row of charter 1.
- `temper/docs/design/client/README.md`, 3.5, Home's *Needs you*, the sentence beginning `Empty, once`, and 5.10, the `repairing` row.
- `temper/docs/plan/ui-small-1/notes/common.md`.

## Owns and may change

Owns the files in the header, each only for the lines below.

Readers, surveyed by `rg` across temper and jig:

- `Word::HomeNeedsEmpty`: `temper/crates/temper-client-view/src/pages/home.rs`, unchanged, and the rows in `tests/home.rs` near lines 1361 and 3654, and `temper/tools/gallery/src/tests.rs` near line 937.
- `ChangeWord::RepairSemantic`: `forge/change.rs` near line 138, where it stops calling `landing_line`, which keeps its other callers; `forge/words.rs` near lines 681 and 783, where it leaves the arm of words with a name's bound; `temper-client-domain/src/tests/change.rs` near lines 50 and 71; `temper-client-view/src/tests/change.rs` near line 225. `temper-client-tools`' `repair:` line takes the same words and names no semantic row.
- `temper/crates/temper-service/src/tests/stock.rs` near line 298 pins `configuring.txt`'s length: give it the new length.
- The Markdown readers are not touched: no file of `.config/gate/read-by-tests.txt` holds these words.

Departs from the implementer's text: three status files are open, each to drop the **Not yet** entry this closes:

- `temper/docs/design/domain/agent.status.md`, 8.1, *Its instructions*.
- `temper/docs/design/protocol/configuration.status.md`, 13.4, charter 1.
- `temper/docs/design/client/README.status.md`, the entry beginning `3.5, Home: its empty`. The **Design departs** entry beside it stays.

Options, each pre-approved:

- If another pinned figure follows `chat.txt`'s or `configuring.txt`'s size (a stock limit, a bound's row), set it to what the new words need, and name it in the report. A figure a stock limit can't hold is a red: shorten the new sentence's wording before you raise any limit.
- If a catch-up finds `configuring.txt` changed by `ui-proposal-2` or `model-catalogue-2`, keep their lines and edit only this sentence.

## Steps

1. `temper/crates/temper-service/src/stock/configuring.txt`: the paragraph beginning `First: settings without a member`, the sentence `An unknown forge identity cannot count for approvals.`, as in the acceptance. Don't move it.
2. `temper/crates/temper-service/src/stock/chat.txt`: the new paragraph, so that it stays before `If your brief says you are an advisor`.
3. `temper/crates/temper-service/src/tests/stock.rs`: a row in the phrase table holding `An unknown forge identity cannot count for approvals` (near line 1437) for `only when approvals are what's being asked about or proposed`, and an `opening.contains` line in `the_chats_opening_sends_plain_edits_directly_to_change` for `call read_branch with its repository before you say it isn't there`.
4. `temper/crates/temper-client-view/src/words.rs`, `Word::HomeNeedsEmpty`, and its rows in `tests/home.rs` and the gallery's `tests.rs`.
5. `temper/crates/temper-client-domain/src/forge/words.rs`: `(ChangeWord::RepairSemantic, &[b"Repairing: CI failed at its updated head."], &[])`, and `forge/change.rs`'s arm as `w::Reason::Ci => W::RepairCi`'s is. Update the rows in both `tests/change.rs`.
6. Narrow tests: `heavy cargo nextest run -p temper-service --lib -E 'test(stock)'`, `heavy cargo nextest run -p temper-client-view --lib -E 'test(home) | test(change)'`, `heavy cargo nextest run -p temper-client-domain --lib -E 'test(change)'`, and the gallery's row by `heavy cargo nextest run -p temper-client-gallery --lib -E 'test(home)'`.
7. The closing checks of your text. Commit and push.

## Report

The implementer's report, with any figure or golden you changed and why, and which rule you broke by hand for the two new phrase rows.
