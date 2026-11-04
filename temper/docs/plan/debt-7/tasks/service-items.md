+++
id = "service-items"
part = "service-clients"
kind = "docs"
needs = ["temper-clients-host", "jig-clients-host"]
files = ["jig/docs/development/debt/D005-accounts-host-copied.md", "jig/docs/development/debt/README.md", "temper/docs/development/debt/D001-reference-service-copy.md", "temper/docs/development/debt/README.md"]
effort = "medium"
minutes = 30
+++

# Settle D005 and D001

## Goal

All seven parts have shipped. Read D001's **Measure** and D005's **Paid when**, and leave the two items true: paid and deleted,
or rewritten to say what's left. Nothing here aims at a number, and this task changes no code.

**What to expect.** The designers' estimate, from reading, is about 330: the adapters each service keeps share lines by
design (accounts about 25, the store's about 80, the workers' between 40 and 80), and the clients' pass stays in each
service, about 150. Branch B below is the likely one. Report the figure you read, whichever it is.

Acceptance:

- Either both items are gone with their README rows, or both are rewritten, with **Checked** set.
- `python3 -I -B tools/citations/citations.py --all` reads 0.
- **Size cap:** the branch's net lines are at most 0: a rewritten item is no longer than it was. Measure it, and stop and
  ask if it's passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`.
- `jig/docs/development/debt/D005-accounts-host-copied.md`, `temper/docs/development/debt/D001-reference-service-copy.md`, whole.
- `jig/docs/design/protocol/service.md`, 11, and its subsections.

## Owns and may change

- Owns: both items, and each tree's `docs/development/debt/README.md`, their rows alone. This is the common note's
  paying rule, for those files.
- No code, no design document. `README.status.md` and `service.status.md` are already right, by the tasks before this one: if one
  isn't, name it under `## Left`.
- D001 has no reader but D005 and that status file, whose citation `temper-clients-host` removes. If a citation is
  left, fix it in this branch.

## Steps

1. D001's measure, by the item's command and per file. `T/server.rs` is gone, so `sed` says so on stderr and `server`
   reads 0: that's the file's true value, not a failure. Write down the sum, and each file's figure.
2. D001's **Paid when** has two halves: `temper-service` composes the protocol layer from a jig crate jig's reference uses
   too, and keeps only temper's own parts; and the measure falls below 200.
3. D005's **Paid when**: read each of the seven parts in `J` and `T` (`notes/service-parts.md` gives the paths), and check that
   neither service holds its own copy: `jig_protocol_accounts::host`, `jig_protocol_worker_agents::{sources, route}`,
   `jig_protocol_store::host`, `jig_protocol_engine_workers::host`, `jig_protocol_engine_clients_http::server` and
   `jig_protocol_engine_clients::host` are what each service's file composes.
4. **Branch A: both hold.** Delete both items and their rows.
5. **Branch B: either doesn't.** Rewrite both, keeping each file's form (the `- **Field:**` head, `## What`, `## Why it matters`,
   `## What pays it back`). In each:
   - **Checked:** today (`date -u +%F`), at the commit you read (`git rev-parse --short=10 HEAD`).
   - **Paid when:** say what's left to do, not what's done. If the parts are all composed and only the measure's bound
     fails, D001 says the bound is what's left, and D005 names what of jig's is still written twice.
   - **What:** what's left, by name: the pass in each `clients.rs`, the adapters' own lines (`publish`, `complete`, `limits`,
     `sizing_tests`), the bundle's copies (`T/clients/bundle.rs` and `J/server/bundle.rs`) and `clients/translation.rs`, which the
     measure can't see. Say which of them can move and which are an application's by `jig/docs/design/protocol/service.md`, 11.
   - D001's **Measure:** the command as it is, with today's value, per file in the text. The item's 1,369 was 1,388 at this plan's base.
   - D001's and D005's stale facts go: the `Debug` drift in `program_sources.rs` (both hide it; and the file has moved);
     "the seven hold 3,587 lines" and the per-file counts, for today's; D001's "D009 is the same pattern at the client"
     (no such item); "`temper-service/src/accounts.rs` is 440 lines" and the like. D005's "written so that it can be copied" is
     not what `service.md`, 7 says now.
   - **Value** and **Size:** what's left is worth, by your reading. A measure of a few hundred lines of shared braces and
     adapter calls isn't "about 1,400 lines": say so.
   - **Status:** open. D001's "(waits on jig's D005)" goes.
   Update the row's title in the README if the item's title changes.
6. The citations check. Commit and push.

## Report

The measure by file and its sum, which branch you took and why, what you found of each part, and net lines. No test runs:
this task changes no code.
