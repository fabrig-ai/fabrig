# D048: The oauth-accounts world switches on stories

- **Kind:** tests
- **Where:** `skein/tests/oauth-accounts/src`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T5, copies between worlds) and `reviews/2026-10-31-skein-areas.md` (A10)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 4)
- **Checked:** 2026-11-06
- **Value:** low
- **Size:** L
- **Status:** open
- **Paid when:** the oauth-accounts world's scenarios are data the referee reads, not branches on a story name inside the harness (`skein/docs/foundation/testing-strategy.md`, 10).
- **Measure:** `rg -c 'story ==' skein/crates/skein-oauth-accounts skein/tests/oauth-accounts | awk -F: '{s+=$2} END{print s}'`: 80

## What

The world has one owner harness and one `Process`, one protocol driver and one restart wrapper (the plan debt-29). It still branches on `story ==` 80 times. The measure undercounts: `story !=` adds 15, `matches!` on a story 39 across lines, and about 10 `match` blocks and the label helpers `again`, `confidential`, `paste::story`, `stops_at_issuer`, `delivery_ready`, `callback_permitted` and `return_location` carry more. A search that reads 0 doesn't pay this.

## Why it matters

A story switch hides each scenario's behaviour inside the harness, so adding a story means editing the harness in many places.

## What pays it back

Rows, in this order, keyed by `Story`, with one `match` giving a story its row: for fixtures (about 35 sites, level in lines), for private faults and files (about 16; `testing-strategy.md`, 7 names faults as scenario data), for owner scripts and gates (about 57, the hard one), and for expected values (about 19, with `Option` fields, since the simulated tier doesn't check posts for `Cancel`, `Abort` and `Timeout`, and mustn't start to). The cut's estimate is 270 to 405 minutes, and level in lines.

## What blocks it

Its Value is low and the rows are level in lines, so the plan debt-29 left them for higher-priority work. And `PasteThenListener`'s hold can't honestly become fields: it orders three parties across two tiers, and the tiers use different predicates (the browser waits for `Fact::Taken`, the wire for `pasted`, one observe turn earlier), so unifying them retimes the story, and about five settings true in one row would be a renamed flag. `Restart`, `ListenerThenPaste`'s close and the browser's stage machine are the same in smaller. They need a step script interpreted in each tier, roughly a rewrite of `owner.rs`, `paste.rs` and the browser's stage logic, or a recorded departure naming them as one scenario's own test code, which requires agreement on the testing contract.
