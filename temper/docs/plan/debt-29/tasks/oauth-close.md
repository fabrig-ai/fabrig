+++
id = "oauth-close"
part = "oauth-owner"
kind = "feature"
needs = ["oauth-one-driver", "oauth-one-restart"]
files = ["skein/tests/oauth-accounts/tests/suite/baseline.rs", "skein/tests/oauth-accounts/tests/suite/main.rs", "skein/tests/oauth-accounts/tests/suite/sign_in.rs", "skein/tests/oauth-accounts/src/sign_in/cuts.rs", "skein/docs/development/debt/D048-oauth-accounts-world-copies.md"]
effort = "medium"
minutes = 30
+++

# The baseline goes, what bites stays, and D048 says what's left

## Goal

The end of this plan's part of skein's D048. The digests go: they trip on any kit change, so they can't stay. What a harness slip could change silently stays as literal facts, for the stories whose only check is the `Judge`. And the item narrows to what's left, naming what blocks it.

Acceptance:

- `tests/suite/baseline.rs` is deleted, with its `mod` line and `sign_in/cuts.rs`'s rows; `rg -n 'D048 baseline|oauth-close' skein -g '*.rs'` finds nothing.
- A table in `tests/suite/sign_in.rs`, of at most 60 lines, holds literal expected facts for the sign-in stories that are `drop(run(..))` today and that a harness slip could silently change: `Omit`, `Retry`, `CloseKeeping`, `Public`, `Restart`, `TwoPorts`, and the `Return`s that show the same. It replaces those `drop(run(..))` tests rather than sitting beside them. One body checks every row alike.
- **It bites:** break by hand what one row reads, say the owner line that records a fact its story expects, see that row go red, and restore. Report what you broke and what failed.
- `skein/docs/development/debt/D048-oauth-accounts-world-copies.md` reads as below, with the counts measured on your branch, and stays open.
- `python3 -I -B tools/debt/debt.py` exits 0.

## Inputs

- `temper/docs/plan/debt-29/notes/oauth-world.md`.
- `skein/docs/development/debt/D048-oauth-accounts-world-copies.md`.
- `skein/docs/foundation/testing-strategy.md`, 10: "Rows carry data", and the table's body never branches on a label.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the files in the header. A story whose facts won't fit a row, or that a run shows carries an unstable fact, is left out and named in your report; the 60 lines are this task's estimate, within the common note's rule on caps. Run the baseline once before you delete it, to see it green at the last.

**If `oauth-one-driver` took its smaller landing,** the item's text also names the client half of the two drivers as left, with its size.

## Steps

Measure, on your branch:

- `rg -c 'story ==' skein/crates/skein-oauth-accounts skein/tests/oauth-accounts | awk -F: '{s+=$2} END{print s}'` (75 by the survey's count)
- the same for `story !=` (15)
- `rg -U -c 'matches!\(\s*[\w.]*story' skein/tests/oauth-accounts/src | awk -F: '{s+=$2} END{print s}'` (37)
- `rg -c 'match (self\.)?story\b' skein/tests/oauth-accounts/src | awk -F: '{s+=$2} END{print s}'`

Rewrite the item, keeping its header's **Seen** lines, **Kind**, **Value** and **Status**, and its **Paid when** as it stands but for the owner it now has:

```md
# D048: The oauth-accounts world switches on stories

- **Where:** `skein/tests/oauth-accounts/src`
- **Size:** L
- **Paid when:** the oauth-accounts world's scenarios are data the referee reads, not branches on a story name inside the harness (`skein/docs/foundation/testing-strategy.md`, 10).
- **Measure:** `rg -c 'story ==' skein/crates/skein-oauth-accounts skein/tests/oauth-accounts | awk -F: '{s+=$2} END{print s}'`: <measured>

## What

The world has one owner harness and one `Process`, one protocol driver and one restart wrapper (the plan debt-29). It still branches on `story ==` <n> times. The measure undercounts: `story !=` adds <n>, `matches!` on a story <n> across lines, and about <n> `match` blocks and the label helpers `again`, `confidential`, `paste::story`, `stops_at_issuer`, `delivery_ready`, `callback_permitted` and `return_location` carry more. A search that reads 0 doesn't pay this.

## Why it matters

A story switch hides each scenario's behaviour inside the harness, so adding a story means editing the harness in many places.

## What pays it back

Rows, in this order, keyed by `Story`, with one `match` giving a story its row: for fixtures (about 35 sites, level in lines), for private faults and files (about 16; `testing-strategy.md`, 7 names faults as scenario data), for owner scripts and gates (about 57, the hard one), and for expected values (about 19, with `Option` fields, since the simulated tier doesn't check posts for `Cancel`, `Abort` and `Timeout`, and mustn't start to). The cut's estimate is 270 to 405 minutes, and level in lines.

## What blocks it

Its Value is low and the rows are level in lines, so the plan debt-29 left them while fabrig/ui has priority over builder places. And `PasteThenListener`'s hold can't honestly become fields: it orders three parties across two tiers, and the tiers use different predicates (the browser waits for `Fact::Taken`, the wire for `pasted`, one observe turn earlier), so unifying them retimes the story, and about five settings true in one row would be a renamed flag. `Restart`, `ListenerThenPaste`'s close and the browser's stage machine are the same in smaller. They need a step script interpreted in each tier, roughly a rewrite of `owner.rs`, `paste.rs` and the browser's stage logic, or a recorded departure naming them as one scenario's own test code, which is fabrig/debt's to agree.
```

Then the note's **Common to the tasks**, without the baseline command once it's gone. Commit, push.

## Report

As your text says, with the four measures, the rows kept and left out, what the break showed, and the part's net lines across its five tasks if you can read them from the trunk.
