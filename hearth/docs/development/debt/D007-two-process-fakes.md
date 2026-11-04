# D007: mmdebstrap's fake is written twice, as a fake-machine process and as a hosted one

- **Kind:** duplication
- **Where:** `testing/hearth-fake-mmdebstrap/src/process.rs`, `tests/builder/src/hosted/mmdebstrap.rs`
- **Seen:** 2026-11-04 at `66968a30ef`, new, from a search for repeated blocks across hearth's test code
- **Checked:** 2026-11-04 at `66968a30ef`
- **Value:** low (one fake's printing and resolving to change)
- **Size:** S
- **Status:** open
- **Paid when:** the code the two share, the private `resolve` and what it calls, sits once in `hearth-fake-mmdebstrap`, and each file holds only its way of reading and writing files.

## What

`process.rs` (242 lines) runs simulate against `hearth-fake-machine`.
`hosted/mmdebstrap.rs` (180 lines) runs it behind skein's simulator. Both
sit on the same `face` and `resolve` modules, and on top of them each has
its own private `resolve`: 65 of the hosted file's lines are also in
`process.rs`, in 60 runs of five. They differ in how they reach files:
`start`, `open`, `read` and `write` against the fake machine, or the
hosted file's jobs.

This is the largest test-side copy in hearth. The git pair,
`hosted/git.rs` and fake git's `process.rs`, didn't show one.

## Why it matters

A new output of mmdebstrap has to reach both, or one world checks less.

## What pays it back

Move what they share into `hearth-fake-mmdebstrap`, as functions over
already-read bytes. Each process keeps its own reads and writes.
