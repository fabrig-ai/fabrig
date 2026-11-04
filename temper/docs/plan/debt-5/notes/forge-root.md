# debt-5: what skein-forge's dialects share

Binding for the `skein-forge-root` part. The layout is `skein/docs/design/forge.md`, 4.4; the item is
`skein/docs/development/debt/D010-github-on-forgejo-modules.md`.

## The measure

```
rg -c 'forgejo::' skein/crates/skein-forge/src/github -g '!**/tests/**' -g '!tests.rs' | awk -F: '{s+=$2} END{print s+0}'
```

44 at base, on these names: `id::unsigned` 20, `document::Body` 17, `time` 13, `path` 2,
`decode::Malformed` 1 (53 names on 44 lines).

## The root's modules

| Root module | Holds | From | Task |
|---|---|---|---|
| `limits` | `Limits`, `Problem { Zero, String, Depth }`, `check(&Limits, depth: u32)` | both `limits.rs` | `forge-limits` |
| `time` | the whole of `forgejo/time.rs` | `forgejo/time.rs` | `forge-neutral-modules` |
| `path` | the whole of `forgejo/path.rs` | `forgejo/path.rs` | `forge-neutral-modules` |
| `decode` | `Malformed` | `forgejo/decode.rs` | `forge-neutral-modules` |
| `text` | `Body` and its methods | `forgejo/document.rs` | `forge-neutral-modules` |
| `plan` | `Failure`, `transport` | both `plans.rs`, both `plans/failure.rs` | `forge-plan-failure` |
| `users` | `Users`, `UserEntry` | both `plans/tables.rs` | `forge-users` |

## Every public path stays

A dialect keeps its name for each moved item by `pub use crate::..`, so nothing outside skein-forge
changes: `skein-fake-forge-github`, `skein-fake-forge-protocol`, `skein-forge-setup`, skein's worlds and
temper keep compiling with no edit. **No task edits a fake forge.** If yours would have to, stop and
ask. A re-export under `src/forgejo` is no site of the measure. Under `src/github`, code names
`crate::time`, `crate::path`, `crate::text`, `crate::decode`, `crate::limits`, `crate::plan` and
`crate::users`.

`skein-forge` is `no_std` outside tests: root modules keep `alloc`'s imports. Each new root
module is `pub mod` in `src/lib.rs`, with a module doc citing `skein/docs/design/forge.md`, 4.4.

## Calls the cut made

- **`Problem` keeps `String`.** One check needs one name: `TokenizerString` goes, because its only
  reader outside `limits.rs` is one line of `skein-forge/tests/suite/forgejo_bounds.rs`. temper names only
  `Problem::Depth`. The item's Paid-when asks for this; it is the one public name that changes.
- **One check's order is GitHub's,** with `json.string == 0` a `Zero`. For Forgejo this only relabels a
  refusal: such limits were refused already, as `TokenizerString`.
- **`Bounds` and "order" stay,** though the item's last section names them and its Paid-when does not.
  GitHub's `Bounds` has `inner`, which Forgejo would never read, against `forge.md`, 4.2; "order" is
  written three times inside GitHub and not at all in Forgejo, so it is no twin.
- **The field walkers stay:** u64 and u32 masks, different `Want`s, and `github/decode/walk.rs` says
  it is independent. Merging them is a redesign.
- **`forgejo::id::unsigned` stays** for Forgejo's readers and the fakes. GitHub calls
  `skein_lib::bytes::decimal`, which is what it wraps.
- **The user table's clock stays in each `Tables`:** `Users` takes `used` from `Tables::tick`, so a
  table keeps one clock and D006's measure stays 2. GitHub's guard for a zero capacity becomes the
  one form; for Forgejo, a zero-capacity keep no longer panics, which nothing relies on.
- **`skein-forge-setup/src/host/limits.rs`** has a like copy (about -14). The item does not name that
  crate, so it stays, and the plan says so.
- **D005 and D006 stay open.** D010 moves `time.rs` whole and pays only D006's last clause.
  `forge-root-close` corrects their text. D011 and D013 are not paid; a root name here must leave
  D011 free to repoint the fake later.

## Size

About -50 non-test lines, by the cut's count: `limits` -24, `plan` -8, `users` -5, forge-connection
-24, the new modules +10. `git mv` counts as zero. The item's "about 1.5k" counted walkers, flat
enums and fakes, which this part does not touch.

## What runs

Each task, once it has built what it names:

```
heavy cargo nextest run -p skein-forge
heavy cargo check --tests -p skein-forge-connection -p skein-forge-client -p skein-forge-setup -p skein-fake-forge-github -p skein-fake-forge-protocol -p github-conformance -p forgejo-conformance -p skein-forge-machine-world -p skein-forge-protocol-world -p skein-forge-setup-world -p temper-service
heavy cargo clippy -p skein-forge --all-targets -- -D warnings
```

Each count above zero. No existing assertion changes unless the task names it.
