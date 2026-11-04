+++
id = "smith-startup-writers"
part = "smith-items"
kind = "migration"
needs = ["skein-startup-store"]
files = ["smith/crates/smith-local-shell/src/credentials/private_file.rs", "smith/crates/smith-local-shell/src/compose/files.rs", "smith/crates/smith-local-shell/src/compose/spawned.rs", "smith/crates/smith-local-shell/src/compose/cleanup.rs", "smith/crates/smith-local-shell/src/compose.rs", "smith/docs/development/debt/D009-startup-writers.md", "smith/docs/development/debt/README.md"]
effort = "high"
minutes = 50
+++

# smith: pay D009, startup files written through skein

## Goal

Pay `smith/docs/development/debt/D009-startup-writers.md`: smith writes its private startup files through
one skein-shell call, as it reads them through `read_private_file`, and `ResultFile` writes results only.
Both writers move to `skein_shell::store_file`, and the ephemeral directory's cleanup moves out of
`ResultFile` into its own small host.

Acceptance:

- `struct Writing` and `impl Host for Writing` are gone; `credentials/private_file.rs` holds only what is
  smith's: making and checking the directory, and mapping the answers.
- `ResultFile::write_startup`, `empty_directory`, `remove_directory` and the states `StoreRoot` and
  `Directory` are gone. `ResultFile`'s `directory` is a `Token`, not an `Option`.
- `compose/cleanup.rs` holds `EphemeralCleanup`, and `Files::close_startup` drives it.
- Every smith test passes unchanged but for the two cleanup tests, which move and name the new type; their
  assertions don't change. The strings both sites answer don't change.
- **Size cap:** the branch's net lines are at most -38. Measure it, and stop and ask if it's passed.
- The item's file and its README row are gone.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `skein/docs/design/shell.md`, 6.7.
- `smith/docs/design/shell.md`, 4.2 to 4.3, 5.4 and 7.2 (they say what these files are; nothing changes).
- `skein/crates/skein-shell/src/startup/store.rs`.

## Owns and may change

- Owns: `credentials/private_file.rs`, `compose/files.rs`, `compose/spawned.rs`, the new `compose/cleanup.rs`,
  `compose.rs`'s `mod` and `use` lines.
- May change: `use` lines in `credentials/login/key.rs` and `credentials/host_id.rs` only if the move forces it.
  `private_file::store`'s signature stays, so it shouldn't.
- Not yours: anything in skein. If `store_file` lacks something, stop and ask.
- If `RESULT_FILE_OPERATIONS` still counts for `ResultFile` alone, it stays.

## Steps

1. `private_file::store` keeps its signature and its strings. After the directory checks, in today's order:

   ```rust
   let root = skein_shell::open_root(grandparent).map_err(|_| "cannot open private directory")?;
   let mut kernel = skein_shell::Kernel::open(skein_shell::Config { operations: skein_shell::STORE_OPERATIONS })
       .map_err(|_| "private file kernel unavailable")?;
   let store = skein_shell::StartupStore {
       root,
       directory: skein_shell::StoreDirectory::Private { name: directory_name, user: skein_shell::effective_user() },
       name: file_name,
       bytes,
       expected,
       timeout: Duration::from_secs(30),
       seed: u64::from_be_bytes(..),
   };
   match skein_shell::store_file(&mut kernel, &clock, store) {
       Ok(skein_shell::Stored::Kept) => Ok(true),
       Ok(skein_shell::Stored::Conflict) => Ok(false),
       Err(skein_shell::Unstored::TooLarge) => Err("private file exceeds bound".into()),
       Err(skein_shell::Unstored::Unsafe(_) | skein_shell::Unstored::Kernel(_)) => {
           Err("private file operation failed".into())
       }
   }
   ```

2. `Startup::launch` in `spawned.rs` calls `store_file` where it called `write_startup`, with `check_path`
   (`pub(super)` in `files.rs`), `root(parent)` where `parent` falls back to `.` as `ResultFile::open` does,
   `StoreDirectory::Root`, `Expect::Any`, `self.timeout` and `self.seed`. The failure keeps its text:
   `Refused::Local(Failure::Host { detail: format!("generated agent configuration: {why:?}") })`, with `why` the
   `kernel::Error` of `Unstored::Kernel(why)`. `Unstored::Unsafe` and `TooLarge` cannot arise here (no private
   check; the document's bound is 1 MiB): give them the same text over the whole `Unstored`. `Stored::Conflict`
   cannot arise under `Expect::Any`: `unreachable!`, with that reason.
3. `compose/cleanup.rs`, module doc saying it closes an unfinished `--ephemeral` directory's store, removes the
   directory, and closes its parent (`smith/docs/design/shell.md`, 4.2):

   ```rust
   pub(super) struct EphemeralCleanup { .. }

   impl EphemeralCleanup {
       pub(super) fn new(path: &Path, store: Option<kernel::Fd>, parent: Option<kernel::Fd>, timeout: Duration) -> Self;
       pub(super) fn failure(&mut self) -> Option<kernel::Error>;
   }

   impl Host for EphemeralCleanup { .. }
   ```

   `new` is `empty_directory`'s body; the states it needs are `empty_directory`'s, `failed`'s and `iterate`'s
   for `StoreRoot`, `Directory`, `Closing` and `Closed`. The first failure is kept. Move
   `startup_cleanup_closes_a_captured_store_even_when_its_parent_cannot_open` and
   `startup_private_cleanup_settles_after_a_long_path_removal_fails` to its tests, with what they need of
   `complete`. Their `Failure::Close(kernel::Error::Other(20))` reads `kernel::Error::Other(20)`.
4. `Files::close_startup` builds and drives it; its `eprintln!` stays, word for word.
5. Prune `ResultFile`: the removed states, `failed`'s arms for them, the `Closed` branch in `iterate`, and the
   `Option` on `directory`, which only cleanup needed. `Failure` stays, as `local_host.rs` reads it.
6. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p smith-local-shell
   --all-targets -- -D warnings`; `heavy cargo nextest run -p smith-local-shell`; `heavy cargo nextest run -p
   smith --features end-to-end --test end_to_end`, which drives both paths from a real process.
7. Delete the item and its row; the citations check. Commit and push.

## Report

The common note's items. No test is added; the two that move need no break. Net lines against the cap.

## Answers

2026-11-04. Q: the branch measures net -15 against a cap of -50: accept -15, or hold -50 and revise the cleanup?

Neither. The cap becomes **-38** (12 lines, 24%, the limit of what the user's answer of 2026-11-04 lets an answer
call raise), and the branch compacts to meet it; net -15 isn't accepted. The cut's estimate left out `cleanup.rs`'s
own Host plumbing: 112 lines net leave `files.rs`, but the host takes 220, and its exhaustive event vocabulary, Host
methods, queues and test scaffolding stay under the strict subset (no `_` arm), so -50 can't be met. A trial of the
compaction below, formatted by `rustfmt`, brought `cleanup.rs` to 199 lines.

Keep commit `9b6e34d831` and add a commit on the branch; nothing else in it changes. Behaviour, strings and test
assertions stay as they are.

1. `cleanup.rs`: one method holds what follows the store's close, and replaces `remove_directory` and the branch
   written twice, in `new` and in `iterate`:

   ```rust
   fn store_closed(&mut self) {
       self.state = State::Closed;
       if let Some(directory) = self.directory {
           let root = self.io.descriptor(directory).expect("private parent descriptor");
           let name = self.name.clone();
           self.state = State::Directory;
           self.request = Some(file::Request::Remove { owner: Token::new(1), root, name, directory: true });
       }
   }
   ```

   `iterate`'s arm is `file::Event::Closed { .. } if self.state == State::StoreRoot => self.store_closed(),`.
2. `new`: `let (directory, failure) = match parent { Ok(parent) => (Some(..), None), Err(error) => (None, Some(error)) };`
   and no `let mut failure`. The literal sets `request: store.map(|file| file::Request::Close { owner: Token::new(1), file })`
   and `state: State::StoreRoot`, and `if store.is_none() { cleanup.store_closed(); }` follows it; the `match store` goes.
3. `spawned.rs`: `use skein_shell::{Clock, Kernel, Stored, Unstored};`, so the second arm fits one line:
   `Unstored::Unsafe(_) | Unstored::TooLarge => format!("generated agent configuration: {why:?}"),`. The `why` there is
   the closure's, the whole `Unstored`, as step 2 says.
4. Those give -38 by the trial. If the measure falls a line short, `cleanup.rs`'s tests can name `std::path::PathBuf`
   in place, without their `use`.

Rerun step 6's checks and the citations check, and report the net lines against -38. The spec's other acceptance
stands. Past -38, stop and ask: this answer has used the room an answer call has.
