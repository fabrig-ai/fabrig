+++
id = "serve-file"
part = "serve-local"
kind = "feature"
needs = ["local-link"]
files = [
  "temper/crates/temper-service/src/clients.rs",
  "temper/crates/temper-service/src/system.rs",
  "temper/crates/temper-service/src/tests/clients.rs",
  "temper/crates/temper/src/host.rs",
  "temper/crates/temper/src/lib.rs",
  "temper/crates/temper/tests/suite/box_listeners.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "high"
minutes = 90
+++

# Serve keeps the local link in the engine's link file, and prints the printed link alone

## Goal

The request's change in temper's engine (`temper/docs/plan/tui-native-secret/inputs/request.md`, *The change*, its second and fifth bullets, and *Tests*, the file's contents). Today `temper serve` writes the printed link it prints into the engine's link file, and under `temper box` the box's first secret too, so a TUI's sign-in from the file spends the person's printed link. After `local-link`, jig's component also gives `Request::LocalLink { secret }`; serve keeps that link in the file, never prints it, and never writes the printed link there.

Acceptance, by `temper/docs/design/client/README.md`, 6.12 (*The engine's link, kept at rest*) and section 7's first bullet, `temper/docs/design/protocol/setup.md`, 13.4, and `notes/contracts.md`, section 3:

1. **`temper-service`:** `Clients` keeps the local link's secret (`kit::Request::LocalLink { secret }`, replacing `local-link`'s dropping arm) until its listener is up, as `print_link` keeps the printed one, then pushes its URL, `printed::link_public(listener, self.public.as_ref(), &secret)`, on a queue of its own, `local_links`, and never on `links`; the place that moves `engine.clients.links` to `service.links` once the clients have started (`system.rs`, about line 870) moves `local_links` alike, and `Service::local_links()` (contracts, section 3) hands them to the binary. `Link` still goes on `links` alone.
2. **The binary:** `Host::iterate` prints each of `links()` as today and no longer keeps it; it keeps each of `local_links()` with `keep_link` and prints none of it. `serve` no longer computes the box's `first_link` nor keeps it at start (`lib.rs`, about lines 711 to 764): the box's first secret is the printed link's and is never written. The link file's path, mode, directory, removal and `temper serve: link file PATH: WHY` line stay as they are.
3. **Tests,** a failing test first for each:
   - `temper/crates/temper-service/src/tests/clients.rs`, through `with_given_secret()`: one new test, `the_local_link_is_kept_apart_and_signs_in_in_either_order`, rows `local first` and `given first`: after a `turn`, `local_links()` holds one link, at the given link's address and route, differing from `given_link()`; `links()` stays empty; the first named signs in the owner, and the other still signs in after it; spending the local link pushes the next local link on `local_links()` and nothing on `links()`; spending the given link pushes nothing on `local_links()`. The existing `a_given_secret_prints_no_first_link_and_signs_in` and `a_spent_given_secret_prints_a_fresh_link` keep passing, and also assert `local_links()` never holds the given or a fresh printed link.
   - `temper/crates/temper/tests/suite/box_listeners.rs` (focused: the temper crate's `suite` binary):
     - `a_fresh_box_prints_its_block_and_nothing_else`: the file, once serve listens (wait for it as the story waits for `serve_listening`), holds one link and a line feed: a link at the same address as stdout's (`box_link::address`), which is not stdout's link; its `secret=` value appears in none of stdout, stderr and `box.log`. Its mode and its parent's stay asserted.
     - `the_first_link_opened_at_once_waits_and_signs_in`: the file read before the printed link is spent is the file after the fresh printed link is printed (its wait on the file holding `fresh` goes), and differs from both printed links; stdout still holds exactly the first and the fresh printed links.
   - Not run: no process, browser or live story (`notes/common.md`, *Runs*). If a process-tier story (`temper/crates/temper/tests/process.rs` and what it includes) asserts the file holds the printed link, it is in `walks-lower/walks-go`'s files: report it under `## Left` with its name, unchanged.
4. **Status:** delete the **Not yet** bullet naming `serve-file` in `temper/docs/design/client/README.status.md` and the one in `temper/docs/design/protocol/setup.status.md`; in the latter's prose, "Serve keeps the engine's private link file at startup and each new Link" becomes "Serve keeps the engine's private link file at each `LocalLink`, never a printed link". No other design line.
5. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p temper-service`; `cargo nextest run -p temper --lib` and `-p temper --test suite -E 'test(/box_listeners::/)'`; then `-p temper --test suite` whole once, since its box stories share the binary. A red in `temper-service` that `local-link` left (its report's `## Left`, or seen on your base) is yours: fix it in `temper-service`'s tests and name it.

## Inputs

- `temper/docs/plan/tui-native-secret/notes/common.md` and `notes/contracts.md`, sections 1 and 3.
- `temper/docs/design/client/README.md`, 6.12, *The engine's link, kept at rest* (*Written* and *The risk*), and section 7's first bullet; `temper/docs/design/protocol/setup.md`, 13.4, the `temper box` story.
- jig's `jig/docs/design/protocol/client.md`, section 8, *The local link*.
- The code today: `temper-service/src/clients.rs`, `Clients`' `secret` and `links` (about lines 74 and 270 to 310), `print_link` (about line 303), its calls (about lines 410 and 524); `system.rs`, `links` (about lines 34, 299, 338, 571 and 870 to 880); `temper/crates/temper/src/host.rs`, `keep_link` and the links' loop (about lines 41 and 120 to 133); `lib.rs`, `serve` (about lines 694 to 769); `temper/crates/temper/tests/fake_backends/box_link.rs`, `link_path`, `address`, `Boxed::link` and `stop`, read and not changed.

## Owns and may change

Owns `Clients`' local link and `local_links`; `Service::local_links`; the host's two loops; `serve`'s start; the new test and the changed asserts above; the two status edits. Not `temper/crates/temper/src/link_file.rs` (its path, keep, forget and read stay), not `temper/crates/temper/tests/fake_backends/**` or `tests/support/**` (`walks-lower/walks-go`'s), not `box_command.rs` (the box still draws and prints the first printed link), not the TUI's crates.

Options, each decided:

- **`setup::check` in `serve`** was there for the first link alone: if nothing else reads its `named`, remove the call with the block; if something does, keep it and say so.
- **Waiting for the file** in the box stories: poll the file as the story already polls (`boxed.wait`), until it holds a line; no new helper in `box_link.rs`. A helper the two stories share goes at the bottom of `box_listeners.rs`.
- **A public origin** makes the local link's URL as it makes the printed link's (`link_public`), as the file did; don't change it to the listener's own address.

## Steps

1. The failing tests of acceptance 3, then the service, then the binary; the narrowest test after each.
2. Status edits; the runs of acceptance 5; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service -p temper --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base origin/main`.
3. Commit (why: the file holds the engine's local link, so a TUI never spends the person's printed link), push.

## Report

As the implementer's text says, with each test added and the rule broken by hand for it, any process-tier story left under `## Left`, and any `temper-service` red inherited from `local-link` and how it was fixed.
