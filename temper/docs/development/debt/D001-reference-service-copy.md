# D001: temper-service carries a drifted copy of jig's reference service

- **Kind:** duplication
- **Where:** `temper-service/src/accounts.rs`, `temper-service/src/program_sources.rs`, `temper-service/src/server.rs`, `temper-service/src/store.rs`, `temper-service/src/worker_route.rs`, `temper-service/src/clients.rs`, `temper-service/src/workers.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig review of that day (`jig/docs/development/debt/reviews/2026-10-31-jig.md`), findings A4 F3, A11 F6 and A11 F7
- **Seen:** 2026-11-03 at `41d63da8c3`, in jig's backfill (`jig/docs/development/debt/D005-accounts-host-copied.md`, the jig side: the parts shipped as product code)
- **Checked:** 2026-11-03 at `a6a8155d35`
- **Value:** high (about 1,400 lines fewer to keep in step by hand; a fix in jig's service reaches temper)
- **Size:** M
- **Status:** open (waits on jig's D005)
- **Paid when:** temper-service composes the engine's and worker's protocol layer from a jig crate that jig's reference service uses too, and keeps only temper's own parts; the measure falls below 200.
- **Measure:** `for f in accounts program_sources server store worker_route clients workers; do comm -12 <(sed 's/^ *//' temper/crates/temper-service/src/$f.rs | sort -u) <(sed 's/^ *//' jig/testing/jig-test-service/src/$f.rs | sort -u) | wc -l; done | awk '{s+=$1} END {print s}'` (bash): 1369

## What

jig's design has its reference service, `jig-test-service`, "written as
step code, so that it can be copied" (`jig/docs/design/protocol/service.md`,
7). temper copied it, and the copies have drifted. Seven files of
`temper-service/src` share 1,369 of their distinct lines with
`jig-test-service`'s files of the same name (the measure; the seven hold
3,587 lines). By file, counting lines that are not a lone brace or a
comment: `program_sources.rs` 124 of 142, `accounts.rs` 237 of 315,
`server.rs` 200 of 264, `worker_route.rs` 181 of 312, `clients.rs` 271 of
657, `workers.rs` 160 of 426, `store.rs` 132 of 329.

The drift is visible: temper's `program_sources.rs` hides its environment
entries from `Debug`, because they may carry credentials, where jig's
derives `Debug` over them; its path bound comes from temper's settings
where jig's is a literal 1024.

Part of it is paid. `f396d0c6e3` routed temper's accounts through jig's
composition, so `accounts.rs` fell from 1,295 lines to 440, and
`temper-service/src/tokens.rs` now names every token range the review
found hand-written.

## Why it matters

A fix or a change of jig's protocol layer lands in jig's copy and has to
be found and made again in temper's, by hand. The two already disagree
on what `Debug` shows. Reviewers read the same 1,400 lines twice.

## What pays it back

It waits on jig's `jig/docs/development/debt/D005-accounts-host-copied.md`:
jig turns the copied parts (the store's host, the accounts' component,
the program sources, the server, the clients' component, the workers'
side) into product code both services compose, the review's "jig-test-service
is jig machinery", a change to `jig/docs/design/protocol/service.md`, 7.
This item is temper's side: once they ship, temper-service composes them,
deletes its copies and keeps its own parts: its forge, setup and webhook
wiring, its settings, its token table. It unblocks nothing else here;
D009 is the same pattern at the client.
