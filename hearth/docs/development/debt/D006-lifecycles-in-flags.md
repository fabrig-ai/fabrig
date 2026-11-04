# D006: Lifecycles of the builder's protocol layer and store are spread over flags

- **Kind:** complexity
- **Where:** `hearth-protocol-builder/src/lib.rs`, `hearth-protocol-builder/src/git.rs`, `hearth-protocol-builder/src/client.rs`, `hearth-protocol-builder/src/resolve.rs`, `hearth-protocol-builder/src/read.rs`, `hearth-domain-store/src/lib.rs`
- **Seen:** 2026-11-04 at `66968a30ef`, new, met while checking D001
- **Checked:** 2026-11-04 at `66968a30ef`
- **Value:** medium (a state and what it holds become one value; impossible combinations stop compiling)
- **Size:** M
- **Status:** open
- **Paid when:** a reviewer finds, in the files above, one state enum per lifecycle with what each state holds, no `closing` or `closed` flag beside a state, and `Protocol` holds one current call's state, not several `Option`s that hold in turn.

## What

`skein/docs/foundation/programming-model.md`, 5.4, wants one enum per
lifecycle, "never a lifecycle spread over flags and `Option` fields beside
the state".

- `git::Child` holds `process`, `output`, `error` and `input` as `Option`s,
  `failed`, `closing` and `closed` as flags, and a `Closing` struct of four
  more.
- `Protocol` holds `child`, `mode`, `keeping`, `resolving` and `ran`, each
  `Option`, as well as `call`: about 24 `self.` sites read them, and
  `Protocol` asks each in turn whether it is empty.
- `resolve.rs` has `closing` twice, with `ended` beside one;
  `client.rs` has `active`, `refused` and `finished`.
- `Store` has `phase`, and beside it `usable`, `said`, `finding` and four
  `Option`s.

The machines' `State` enums follow the idiom; this is the layer above them,
so it is a sample, not an audit.

## Why it matters

The types don't say which fields hold together: `git::Child` may be
`closing` with no process, and only an `assert!` keeps a second `keeping`
out. The simulator finds a bad mix only if a seed reaches it, and each
handler has to re-derive where it stands.

## What pays it back

`git::Child` first: it is one lifecycle, small, and `Resolve` repeats it.
Then `Protocol`'s current work as one enum, then `Store`. No plan should
take it before D001, which rewrites the same callers' pacing.
`skein_lib::Lifecycle` fits a component its owner closes, if one appears.
