+++
id = "skein-startup-store"
part = "smith-items"
kind = "interface"
needs = []
files = ["skein/crates/skein-shell/src/startup/store.rs", "skein/crates/skein-shell/src/startup.rs", "skein/crates/skein-shell/src/lib.rs", "skein/tests/ring/tests/suite/startup.rs", "skein/docs/design/shell.status.md"]
effort = "high"
minutes = 50
+++

# skein: `store_file`, startup's durable write

## Goal

`smith/docs/development/debt/D009-startup-writers.md` needs one skein-shell call that writes a file at
startup, beside `read_private_file`. smith drives open, store and close by hand in two hosts; this task
gives skein-shell that call, built from smith's `Writing` host, which is the more general of the two.
`smith-startup-writers` moves smith onto it.

Acceptance:

- `skein_shell::store_file` and the types below are exported.
- On a real ring, in `skein/tests/ring/tests/suite/startup.rs`: it keeps a file; `Expect::Absent` on an
  existing file answers `Stored::Conflict` and leaves the file as it was; a private directory with a group
  bit answers `Unstored::Unsafe(Unsafe::Mode(..))` and writes nothing; each of the three leaves the
  descriptor count where it found it.
- `shell.status.md` no longer lists 6.7.
- **Size cap:** at most 190 lines added on the branch, tests included. It adds a host and removes nothing,
  and `smith-startup-writers` removes the copies. Measure it, and stop and ask if it's passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/shell.md`, 6.7 (on the trunk), 6.1, 12.
- `skein/docs/design/io.md`, 5.2 and 5.3.
- `smith/crates/smith-local-shell/src/credentials/private_file.rs`: `Writing`, which you port. You don't
  change it.

## Owns and may change

- Owns: the new `skein-shell/src/startup/store.rs`; `startup.rs`'s `mod store;` and re-export; `lib.rs`'s
  export lines; the new tests in `skein/tests/ring/tests/suite/startup.rs`.
- `skein/docs/design/shell.status.md`: remove `6.7: `store_file`.` from **Not yet**, and update its date
  line. This departs from the implementer's text.
- Not yours: any smith file, `skein-io`.
- `store_file` is `cfg(unix)`, like the rest of `startup`.
- If the file layer needs a change for this, stop and ask.

## Steps

1. `startup/store.rs`, module doc citing `shell.md, 6.7`:

   ```rust
   /// The most operations a startup store has in flight: the ring a caller opens for it alone.
   pub const STORE_OPERATIONS: u32 = 8;

   /// The directory a startup store writes into (shell.md, 6.7).
   #[derive(Clone, Copy, Debug)]
   pub enum StoreDirectory<'a> {
       /// The store's root itself.
       Root,
       /// The directory of this name under the root, opened through io's private-file checks as `user`.
       Private { name: &'a [u8], user: u32 },
   }

   /// One durable whole-file write at startup.
   pub struct StartupStore<'a> {
       /// An open directory from `open_root`; the store closes it on every path.
       pub root: kernel::Fd,
       pub directory: StoreDirectory<'a>,
       pub name: &'a [u8],
       pub bytes: Box<[u8]>,
       pub expected: file::Expect,
       pub timeout: Duration,
       pub seed: u64,
   }

   /// How a startup store ended, with everything it opened closed.
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum Stored {
       /// The file is durable.
       Kept,
       /// The expectation failed; nothing was written.
       Conflict,
   }

   /// Why a startup store failed.
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum Unstored {
       /// io's private-file checks refused the directory.
       Unsafe(file::Unsafe),
       /// The store or a close after it failed; the first such error.
       Kernel(kernel::Error),
       /// The bytes exceed a `u32`.
       TooLarge,
   }

   pub fn store_file(kernel: &mut Kernel, clock: &Clock, store: StartupStore<'_>) -> Result<Stored, Unstored>;
   ```

   Don't derive `Debug` over `bytes`: `StartupStore` has none. Copy the derives `kernel::Error` and
   `file::Unsafe` allow: if one lacks `Copy`, drop `Copy` here.
2. The host is `Writing`, renamed `Storing`, private to the module:
   `FileIo::with_whole_limit(2, 4096, 1, maximum, timeout)` with `maximum` the byte length, at least 1;
   `.with_effective_user(user)` for `Private`; `seed_randomness(seed)`; `adopt_root(root)`. `Private` starts
   with `OpenPrivate { root, path: name }`, and `Opened` makes the store's root the opened file, as `Writing`
   does; `Root` starts with the store. `Stored` and `Conflict` are terminal; so are `Failed`, `Refused` and
   `Cancelled`, which record the first failure (`Cancelled` as `kernel::Error::Cancelled`). A terminal closes
   every root, innermost first. A failure wins over `Kept`, a failed close included, as `Writing`'s `failure`
   does: this is both smith sites' behaviour.
3. `Host`: as `Writing`'s, with `operations()` answering `STORE_OPERATIONS`, and the file timeout in
   `next_policy_deadline` (`shell.md`, 12: "the stall deadlines owners state for files"). `drive` takes the
   kernel it is given, so a caller's larger ring is fine; check that in `drive.rs` and say so.
4. Tests as the acceptance says, using that file's `DESCRIPTORS`, `Scratch` and `Clock`. Open the kernel with
   `Kernel::open(Config { operations: STORE_OPERATIONS })`.
5. `cargo fmt`; `heavy cargo clippy -p skein-shell -p skein-ring-tests --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-shell --lib`; `heavy cargo nextest run -p skein-ring-tests`; the citations
   check. Commit and push.

## Report

The common note's items. Breaks: answer `Kept` on `Conflict`; leave the root unclosed on a refusal; each with
the failure read. Lines added against the cap. Whether `Writing`'s `closed` flag survived as it was, and any
difference from `Writing`.
