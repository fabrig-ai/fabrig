+++
id = "peer-ports"
part = "peer-ports"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/src/fake_backends.rs",
  "temper/crates/temper/src/fake_backends/tests.rs",
]
effort = "medium"
minutes = 30
+++

# A fake peer is never reserved on a port the peer check refuses

## Goal

`fake_backends::tests::reuse_atomically_replaces_bundle_and_preserves_every_other_edit` failed once
with "peer listen must be nonzero loopback with a distinct, unreserved port". The cut's survey
found the cause by reading; the flake was not reproduced. `prepare` builds the ports a peer must
avoid from `deployment.checked.setup.entries`, which `setup.rs` always leaves empty, so the owned
and fake listeners are never excluded. `checked_peers` does count them. In a preparation that
imports a source, nothing holds those two ports, so `reserve` can be handed one by the kernel and
the check then refuses a preparation that was its own doing. The two sets become one.

Acceptance:

- `prepare` and `checked_peers` take their ports from one function, `ordinary_ports`.
- `rg -n 'setup\.entries' temper/crates/temper/src/fake_backends.rs` reads nothing.
- The new test passes, and failed with the old set (below).
- `temper`'s `fake_backends` tests pass with `-p temper --lib -E 'test(fake_backends::)'`, then
  its focused tests.

## Inputs

- `temper/docs/plan/bugs-temper/notes/common.md`.
- `temper/docs/design/protocol/setup.md`, 7.5: each peer is loopback with a nonzero port, distinct
  from the other peer ports and the ordinary box listeners.
- `temper/crates/temper/src/fake_backends.rs`: `reserve`, `checked_peers`, `prepare`.

## Owns and may change

- Owns: `fake_backends.rs` and `fake_backends/tests.rs`.
- `setup.rs` stays as it is: `Checked.setup` is also read by `temper/src/lib.rs`, and is not this
  bug.
- Pre-approved: if `configuration_steps` refuses `initial` in `prepare` for something `rewrite`
  would have repaired, keep `ordinary_ports` for `checked_peers` alone, keep `prepare`'s list, and
  replace its loop over `entries` with the `owned` and `fake` `listen` ports of
  `crate::box_command::machine(&initial, &effective)?`. Say so in the report.

## Steps

1. In `fake_backends.rs`, move `checked_peers`' opening into a function, unchanged in meaning:

   ```rust
   /// The ports no peer may take: the callback's and the ordinary box's listeners and readiness
   /// (temper/docs/design/protocol/setup.md, 7.5).
   fn ordinary_ports(dirs: &Directories, bytes: &[u8]) -> Result<Vec<u16>, String> {
       let (ordinary, _) = crate::box_command::configuration_steps(dirs, bytes, Box::new([]))?;
       let mut ports = vec![1455];
       // the steps loop, as it stands in checked_peers
       Ok(ports)
   }
   ```

   `checked_peers` opens with `let mut ports = ordinary_ports(dirs, bytes)?;`.

2. Test first. In `fake_backends/tests.rs`:

   ```rust
   #[test]
   fn reservations_avoid_every_port_the_peer_check_refuses() {
       let scratch = Scratch::new("ordinary-ports");
       let (_source, first) = prepared_source(&scratch);
       let document = read_json(&first.directories.document());
       let machine = &document["application"]["machine"];
       let ports = ordinary_ports(&first.directories, &first.bytes).expect("ports");
       for listen in [&machine["owned"]["listen"], &machine["fake"]["listen"]] {
           let socket = listen["socket"].as_str().expect("socket");
           let port = socket.parse::<SocketAddr>().expect("address").port();
           assert!(ports.contains(&port), "{socket}");
       }
   }
   ```

   The test reads the document itself, so it does not share the function's derivation. To see it
   fail without kernel luck: for one local run, give `ordinary_ports` the body of `prepare`'s old
   list (from `let deployment` to the demo push), run the test, quote the failure in the report,
   and restore the body. It always fails, because `entries` is empty.

3. In `prepare`, replace everything from `let deployment = crate::setup::check(...)` through the
   demo push with `let mut excluded = ordinary_ports(&effective, &initial)?;`. A document
   `setup::check` refuses is still refused, with the same words, since `configuration_steps` opens
   with that check.

4. Run the `fake_backends` tests, then `cargo fmt`, the old-form check, clippy on `temper` and its
   focused tests, as the implementer's text says. Commit and push.

## Report

As the implementer's text says, with the failure line from step 2. Say plainly that the flake
itself was not reproduced, if it was not.
