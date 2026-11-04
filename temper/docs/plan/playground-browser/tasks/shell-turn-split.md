+++
id = "shell-turn-split"
part = "shell-clippy"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-web/src/lib.rs",
  ".config/gate/wasm32.txt",
  "temper/docs/design/client/browser.status.md",
]
effort = "medium"
minutes = 45
+++

# `temper-client-shell-web` passes wasm32 clippy and joins the gate's list

## Goal

The gate runs clippy for `wasm32-unknown-unknown` over `temper-client-shell-web`
(`temper/docs/design/client/browser.md`, 4.3), so a change that stops temper's browser shell
building fails the gate. Today `heavy cargo clippy --target wasm32-unknown-unknown -p
temper-client-shell-web -- -D warnings` fails on `clippy::too_many_lines` at `src/lib.rs`
(`Shell::turn`, 102 lines of 100). Split `turn` into smaller methods, with no lint exception
(`AGENTS.md`, 2), behaviour unchanged, and add the package to fab's gate list.

Acceptance:

- `heavy cargo clippy --target wasm32-unknown-unknown -p temper-client-shell-web -- -D
  warnings` passes, and so does the same with `--features record`.
- No `#[expect(clippy::too_many_lines, ...)]` or `#[allow]` anywhere in `lib.rs`.
- `.config/gate/wasm32.txt` holds `temper-client-shell-web` on its own line, right after
  `jig-client-shell-web`; every other line stays as the trunk has it (plan `wasm32-temper` may add
  `temper-playground` after `skein-playground` at the same time: keep it if the trunk holds it).
- Natively nothing changes: `heavy cargo nextest run -p temper-client-shell-web` passes with the
  trunk's count, and `heavy cargo clippy -p temper-client-shell-web --all-targets -- -D warnings`
  passes.
- `temper/docs/design/client/browser.status.md`, **Not yet**, 4.3, edited in place as below.

## Inputs

- `temper/docs/plan/playground-browser/notes/common.md` (the standing rules).
- `temper/docs/design/client/browser.md`, 2.1 (the shell) and 4.3 (the client's crates for
  wasm32).
- `temper/docs/design/playground.md`, 6: a later task of this plan (`page-feature`) makes the
  shell's platform a type its feature `playground` chooses, "and nothing else of its loop
  changes". Split `turn` so that the platform is touched in one small method (below), which that
  task then changes alone.

## Owns and may change

- `temper/crates/temper-client-shell-web/src/lib.rs`: the wasm32 `Shell`'s methods only. `start`,
  `Report`, the `record` helpers and `host.rs` stay as they are.
- `.config/gate/wasm32.txt`: the one line above.
- `temper/docs/design/client/browser.status.md`: its date line, and in **Not yet** 4.3 exactly
  these edits, in place, never reflowed: "none of `temper-client-domain`, `temper-client-view`,
  `temper-client-protocol-wire` and `temper-client-shell-web` is in" becomes "none of
  `temper-client-domain`, `temper-client-view` and `temper-client-protocol-wire` is in", and the
  clause "; `temper-client-shell-web` fails it at `clippy::too_many_lines` (`src/lib.rs`,
  `turn`)" is removed, the sentence ending after "`.config/gate/wasm32.txt`)". The other entries
  (2.1, 4.6) stay word for word.
- Options, pre-approved:
  - If wasm32 clippy then reports another lint in `temper-client-shell-web`'s own code, fix it by
    reshaping the item; if a lint fires only in a dependency of another tree (jig, skein), leave
    it, do not add the line to the gate list, and report the first error with its file and line.
  - If `--features record` fails on a lint in the `record` helpers that the plain build does not
    reach, fix it the same way; it is in this file.

## Steps

1. Split `Shell::turn` (`lib.rs`, ~131-232). A sketch, inside which the choices are yours:

   ```rust
   impl Shell {
       /// One inbox turn: each report given to the host, its duties carried, then the page drawn.
       fn turn(&mut self, inbox: &Inbox<Report>);
       /// The host's report for one of the shell's, or none for those the shell keeps
       /// (`record`'s marks and saves); the timer told it fired.
       fn host_report(&mut self, report: Report) -> Option<host::Report>;
       /// Carries one report's duties: requests, navigation, the store's writes, the stream read
       /// on; the only method that touches `self.platform`. Returns whether a new page shows.
       fn carry(&mut self, inbox: &Inbox<Report>, duties: host::Duties, read_on: bool) -> bool;
   }
   /// Re-applies the tab policy to the page's forge sign-in links (the two loops after `apply`).
   fn forge_links(document: &web_sys::Document);
   ```

   Keep every statement's order and every `expect` message as they are: this is an extraction.
2. `cargo fmt`; the two wasm32 clippy commands; the native clippy and nextest above.
3. Edit `.config/gate/wasm32.txt` and the status entry; run the citations check
   (`notes/common.md`).
4. Commit ("Split the browser shell's turn so it passes wasm32 clippy"), then the list and status
   in a second commit; push.

## Report

As the implementer's text says, and also: each clippy command's outcome and the host it ran on,
and the line counts of the methods `turn` became.
