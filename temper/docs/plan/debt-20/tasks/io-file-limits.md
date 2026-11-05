+++
id = "io-file-limits"
part = "io-states"
kind = "feature"
needs = ["io-signals", "io-pipe", "io-file-states"]
files = ["skein/crates/skein-io/src/file_layer.rs", "skein/crates/skein-io/src/file_layer/**", "skein/crates/skein-io/src/lib.rs", "skein/crates/skein-io/src/tests/**", "skein/docs/design/io.status.md", "skein/docs/development/debt/D018-io-pipe-and-file-states.md", "skein/docs/development/debt/D049-file-layer-thin-forms.md", "skein/docs/development/debt/README.md"]
effort = "high"
minutes = 50
+++

# skein: `FileIo` takes `Env` and `Limits`

## Goal

Pay `skein/docs/development/debt/D018-io-pipe-and-file-states.md`. `FileIo` is made from a `Limits` and driven with an `Env<Limits>`, like `Io` (`skein/docs/design/io.md`, 5). The old names stay as thin forms, since `hearth-service-builder/src/lib.rs` is another plan's and no reader moves in this one. That's written down in a new item.

```rust
// skein_io::file_layer
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits { pub files: u32, pub read: u32, pub entries: u32, pub whole: u32, pub timeout: Duration }
#[must_use] pub fn valid(limits: &Limits) -> bool;     // today's "file limits need room" check, named
impl FileIo { #[must_use] pub fn with_limits(limits: &Limits) -> FileIo; }  // asserts `valid`
pub mod step {
    pub fn down(io: &mut FileIo, env: &Env<Limits>, request: Request, events: &mut Queue<Event>, subs: &mut Queue<Submit>);
    pub fn down_until(io: &mut FileIo, env: &Env<Limits>, deadline: Time, request: Request, events: &mut Queue<Event>, subs: &mut Queue<Submit>);
    pub fn up(io: &mut FileIo, env: &Env<Limits>, complete: Complete, events: &mut Queue<Event>, subs: &mut Queue<Submit>);
    pub fn expire(io: &mut FileIo, env: &Env<Limits>, subs: &mut Queue<Submit>);
}
```

`FileIo::new`, `with_whole_limit` and `file_layer::{down, down_until, up, expire}` keep their signatures and forward: each builds `Env { now, wall: Wall::EPOCH, limits: io.limits }`, so `FileIo` keeps one `limits` field for them. `up` has no `now` and passes `Time::ZERO`, which it never reads. `cancel` doesn't change.

Acceptance:

- `rg -n 'io\.(max_read|max_file|max_entries|timeout)\b' skein/crates/skein-io/src/file_layer.rs skein/crates/skein-io/src/file_layer` finds nothing.
- `rg -c '^pub fn (down|down_until|up|expire)\b' skein/crates/skein-io/src/file_layer.rs` is 4, each a forward. `rg -c 'file_layer::(down|down_until|up|expire)\b' skein/crates/skein-io/src/tests` is at most 4: one test per thin form. The rest of `skein-io`'s own tests are on `step`.
- The branch changes nothing outside `skein/` (`hearth/crates/hearth-service-builder/src/lib.rs` above all), and `heavy cargo check -p hearth-service-builder -p smith-local-service -p smith-agent-service -p jig-protocol-store -p temper-box --tests` compiles.
- A test pins `valid`: a limit with no room is refused, and `read: 4096, whole: 1` is accepted, as `skein-shell` runs it, so `whole >= read` isn't required.
- `heavy cargo nextest run -p skein-io --lib`, `-p skein-io-world --test suite`, `-p skein-kv-tests` pass, with what they assert unchanged.
- By hand: read `entries` where `read` belongs in `step::up`'s path (a test goes red); make `valid` always true (its test goes red). Restore each.
- The net production lines are at most +70 (estimate: +60). The item is deleted, its `3, 5, 7:` clause and this plan's `5: file_layer::Limits` clause leave `io.status.md`, and that file gains the entry below.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `skein/docs/development/debt/D018-io-pipe-and-file-states.md`
- `skein/docs/design/io.md`, 5 (this plan's paragraph on `Limits`)
- `skein/docs/foundation/programming-model.md`, 6.3
- `skein/crates/skein-io/src/limits.rs` and `stage.rs`: the sibling's form

## Owns and may change

- **Owns:** the files in `files`.
- **`file_layer::step` is a new name, so nothing else needs to change.** Limits are read at 7 sites (`down`, `read_whole`, `up_inner`, `directory::page`, `store::start`, `read_check`).
- **`smith-agent-service/src/service.rs`'s hand check of the same room** stays: a reader's.
- **`io.status.md`, the new **Departs** entry, in place of the closed clause:**
  > 5: `FileIo`'s readers call the thin forms, not `file_layer::step`, because the move is wide (216 sites, 149 outside `skein-io`'s tests), some readers keep no clock, and `hearth-service-builder/src/lib.rs` is being migrated apart. `file_layer/store.rs`'s `Store` and `file_layer/private.rs`'s `Opening` keep phases in flags and `Option`s, against `skein/docs/foundation/programming-model.md`, 5.4. (The code's side is `skein/docs/development/debt/D049-file-layer-thin-forms.md`.)
- **The new item,** `D049-file-layer-thin-forms.md`, in the form of its neighbours (`python3 -I -B tools/debt/debt.py skein` checks it): **Kind** complexity; **Where** `skein-io/src/file_layer.rs`, `file_layer/store.rs`, `file_layer/private.rs`, every tree's readers; **Value** low; **Size** M; **Status** open; **Paid when** no reader calls `file_layer::{down, down_until, up, expire}` or the two old constructors, they're gone, and `Store` and `Opening` follow 5.4; **Measure** `rg --type rust 'file_layer::(down|down_until|up|expire)\b' skein smith jig temper hearth | wc -l`, with what it reads at your commit. Raise the highest ID in `skein/docs/development/debt/README.md` to D049.

## Steps

1. `Limits`, `valid`, `with_limits`, with their test; `new` and `with_whole_limit` over them.
2. The internals read `env.limits` and `env.now`: `step`'s four functions; the old four forward.
3. `skein-io`'s tests onto `step`, keeping one per thin form.
4. Break the rules by hand, as the acceptance says.
5. Delete the item, write the new one, edit the status. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-io --all-targets -- -D warnings`; the tests above; `python3 -I -B tools/debt/debt.py skein`. The citations check. Commit and push.

## Report

The common note's items; D049's **Measure** figure; what you broke and what failed.
