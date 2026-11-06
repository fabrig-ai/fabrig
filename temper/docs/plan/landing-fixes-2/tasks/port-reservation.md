+++
id = "port-reservation"
part = "port-reservation"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/src/init.rs",
  "temper/crates/temper/src/init/tests.rs",
  "temper/crates/temper/src/fake_backends.rs",
  "temper/crates/temper/src/fake_backends/tests.rs",
]
effort = "high"
minutes = 45
+++

# A reservation that loses its port takes another

## Goal

The request's item 4. `fake_backends::tests::repeating_the_same_bundle_preserves_document_bytes_and_inode` was seen twice failing on a port reservation, and passes alone. The test takes no port itself. It reaches `fake_backends::prepare`, and from reading, two places there take a kernel-chosen port and fail for good if they lose it:

- `fake_backends.rs`, `reserve`: `TcpListener::bind((LOCALHOST, 0))` is a socket, `SO_REUSEADDR`, a bind and a listen. Its `?` returns on the first error, so its 32 tries only cover a port in `excluded`. The test's failure would read `fresh sandbox: "fake peer reservation: Address already in use (os error 98)"`.
- `init.rs`, `Io::port` and `Io::listened`, reached with `[0; 5]` from `initialized_source` and `prepare`. Under `SO_REUSEADDR`, another process can bind the same port explicitly and listen first, between this socket's bind and its listen. `listened` retries only when the port was a preferred one, so a kernel-chosen port that loses fails with `errno 98`. `init/tests.rs`, `a_port_another_init_listens_on_between_bind_and_listen_gets_a_kernel_port`, already sets up this race for a preferred port.

The evidence's log isn't in this repository, so which of the two was seen is unknown. Fix both. This is a cause from reading: if the test's reach holds another port path, fix that too and say so.

Acceptance:

- A kernel-chosen reservation in `Io` whose listen answers `Error::AddressInUse` closes that socket and asks the kernel again, at most 32 tries in all, then refuses as it does now. Any other error refuses at once, as now. A preferred port keeps its behaviour.
- `reserve` treats `std::io::ErrorKind::AddrInUse` as a try spent and takes another, within its existing 32. Any other error refuses at once with its existing words. Running out reads as its existing "could not choose" words.
- Neither path ever reports a port it doesn't hold.
- The named test passes, and so does every test of `fake_backends::` and `init::` in `temper`'s lib.
- No test here depends on winning a timing: each race is staged or injected in one thread.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.5 (a sandbox takes the kernel's ports, held until transferred) and 8.2 (`temper init`'s ports). Nothing here changes what they say.
- `temper/docs/plan/landing-fixes-2/notes/common.md`.

## Owns and may change

Owns the files in the header.

Reader and neighbour files: none need changing. `reserve` is read by `fake_backends.rs`'s `prepare` and by `fake_backends/tests.rs`, `retained_peer_edits_are_checked_without_replacement`. `Io::listened` is read by `Io::port` and by `init/tests.rs`.

Other drafts: `ui-repo-link/box-sign-in` changes `temper/crates/temper/src/init.rs`, so keep your hunk inside `Io`.

Options, each pre-approved:

- `reserve` needs a seam to inject a lost port, since std's bind can't be split. Default: `fn reserve_from(excluded: &[u16], bind: impl FnMut() -> std::io::Result<TcpListener>) -> Result<TcpListener, String>`, private, with `reserve` passing the real bind. A simpler seam of the same reach is yours, and say so.
- `Io::listened`'s `preferred: bool` may go if the retry makes it needless. `init/tests.rs`'s call is a call site you may edit.
- Other tests that release a port and bind it again by number can race the same way (`fake_backends/tests.rs`, `peer_reservations_transfer_once_and_close_with_preparation`, its `held`; `temper/crates/temper/tests/support/mod.rs`, its free socket). They have no failure on record, so they stay as they are. Name the ones you see under `## Left`.

## Steps

1. **Reproduce the init half.** In `init/tests.rs`, beside the test above, add `a_kernel_chosen_port_another_takes_before_listen_gets_another`. Stage the same two binds and have the other listen first, then call `listened` on the late socket with the kernel-chosen path. Today it answers `errno 98`. Afterwards it answers a different port, held, as the existing test asserts.
2. **Reproduce the fake half.** Add the seam with no change in behaviour, then in `fake_backends/tests.rs` add `a_reservation_that_loses_its_port_takes_another` (the bind answers `AddrInUse` once, then a real listener on port 0) and `a_reservation_refuses_at_once_on_any_other_error` (it answers `PermissionDenied`, and the bind runs once). The first fails today.
3. **Add a bound test each:** the bind or listen always loses, and the result is a refusal after 32 tries, never a hang. For `Io`, if a real socket can't stage 32 losses, a loop bound read from one named constant and a test of `reserve_from` is enough. Say which you chose.
4. **Fix** both. Share nothing between `Io` and `reserve` unless it's one constant.
5. Break each retry once by hand, see its test fail for that reason, and restore.
6. **Run the named test by itself:** `heavy cargo nextest run -p temper --lib -E 'test(=fake_backends::tests::repeating_the_same_bundle_preserves_document_bytes_and_inode)'`. Read what it reaches (`validate`, `crate::setup::check`, `replace_bundle`) and confirm that nothing there binds.
7. `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p temper --all-targets -- -D warnings`, then `heavy cargo nextest run -p temper --lib -E 'test(fake_backends::) | test(init::)'`, and `temper`'s `box_listeners` tests, which read the reservations. Commit and push.

## Report

The implementer's report, plus: whether anything in the test's reach other than these two takes a port, and the racy tests you saw under `## Left`.
