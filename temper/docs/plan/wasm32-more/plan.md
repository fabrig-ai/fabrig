# wasm32-more: plan

Draft: the design's other wasm32 crates measured for fab's gate list (`inputs/request.md`), the
recommended option of plan wasm32-list's question `wasm32-more`. fab's gate step
`clippy-wasm32` reads `.config/gate/wasm32.txt`, six packages at `0db5c4f348`.

## Parts and tasks

- **gate-more** (slice 1): `tasks/gate-more.md`, no needs, released in the first and only push.
  Clippies 46 packages alone for wasm32 at its base, appends the green ones to the list, runs
  the whole list once, records each red one as Not yet in its tree's status with its first
  error, closes the entries the green ones settle, and writes the answer into
  `temper/docs/plan/wasm32-list/plan.toml`.

## Decisions

- **One task,** as the request asks: the list is one file with one writer, and the per-tree
  measurements share one warm `target/`; split across slots, each would build skein's and
  smith's shared crates again.
- **The names are resolved in the spec,** from `cargo metadata` and the design sections the
  request cites, so the implementer measures rather than interprets: skein's 18 portable crates
  (`testing.md`, 10 lists them; `skein-playground` is listed already), jig's six client crates
  and `jig-client-test-shell-web` (`protocol/browser.md`, 15) and `jig-protocol-transcript`
  (`protocol/README.md`, 11; `browser.md`, 18), temper's four (`client/browser.md`, 4.3), and
  `smith-agent-shell` with the 15 smith crates `cargo tree --target wasm32-unknown-unknown -e
  normal` gives it (`shell.md`, 2.5). The implementer re-checks each against `cargo metadata`
  and reports, never guesses, a missing one.
- **`jig-client-shell-native` is left out:** "every client crate" (`browser.md`, 15) is read
  as the browser's; the native shell is over skein's HTTP client and io
  (`protocol/README.md`, 5) and builds for the host alone. Recording it red would ask jig for
  a build its design does not want.
- **Already linted, still measured:** the six listed packages' wasm32 tree already holds
  `skein-lib`, `-codec`, `-io`, `-sim`, `-shell`, `-http`, `-sim-world`, jig's client crates,
  `jig-protocol-transcript` and `smith-transcript` (clippy lints a listed package's workspace
  dependencies). The request asks for each crate alone, and listing them lets the gate's step
  select them directly; their status has no gap today. A red alone among them (features differ
  alone) is recorded with "(alone)".
- **The status edits are the request's red records plus the closing of entries the green ones
  settle,** since a status holds the gaps as they stand (`DESIGN.md`, 2): skein's
  `testing.status.md` 6, 10, `world.status.md` 8 and `STATUS.md`; jig's
  `protocol/browser.status.md` 15 (and `protocol/README.status.md` 11 only if the transcript
  crate is red) and `STATUS.md`; temper's `client/browser.status.md` 4.3 and `STATUS.md`;
  smith's `shell.status.md` 2.5 and `STATUS.md`. Each has a rule for all-green and for some red,
  and a red line has one form: `` `<package>`: `<lint>` at `<file>:<line>` ``.
- **No design section is written:** the cited sections already name the crates and the gate's
  check; only the status follows the code.
- **The combined run's failure:** a red in a crate only the added packages bring in takes those
  packages out and reruns once; a red in the base list's own tree is the trunk's, reported.

## Overlaps with the drafts beside this one

- **playground-browser** (integration): its trunk adds `temper-client-shell-web` to
  `.config/gate/wasm32.txt` between `jig-client-shell-web` and `skein-playground` (task
  shell-turn-split, with the fix of `turn`), and rewrites `temper/docs/design/client/
  browser.status.md` 4.3 to three crates; its task browser-walk (not merged) edits that status
  file too. This plan appends after `skein-playground`, which merges cleanly with the insertion
  (wasm32-temper-2 checked the same pair with `git merge-file`), and skips measuring
  `temper-client-shell-web` when its base's list holds it. The 4.3 entry is unavoidable: the
  request asks for it, and the spec applies its rule to whichever text the base has.
- **public-reading-build** (integration): task public-status (not merged) edits
  `skein/docs/design/STATUS.md`, `jig/docs/design/STATUS.md`, `temper/docs/design/STATUS.md`
  and `temper/docs/design/client/browser.status.md`, whose wasm32 sentences this plan edits.
  Unavoidable for the same reason; the edits are one sentence or clause each.
- No listed file of configuring-p3 or wasm32-rustls is touched; the queued drafts
  (v0.1.0-ladder, sign-in-wait, reference-walks) and fabrig-scale list no files yet.

## Critical path

gate-more alone: about 90 implementer-minutes, most of them 46 wasm32 clippy builds through
`heavy`, the first few cold, the rest on the warm `target/`, and one combined run.
