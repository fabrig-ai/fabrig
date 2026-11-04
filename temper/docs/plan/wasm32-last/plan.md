# wasm32-last: plan

Draft: the last re-measure of the gate's wasm32 list (`inputs/wasm32-last.md`), after
wasm32-skein and playground-browser landed. fab's gate step `clippy-wasm32` reads
`.config/gate/wasm32.txt`, 44 packages at `6730de2602`.

## Parts and tasks

- **gate-last** (slice 1): `tasks/gate-last.md`, no needs, released in the first and only push.
  Clippies nine packages alone for wasm32 at its base, appends the green ones to the list, runs
  the whole list once, closes the entries the green ones settle and refreshes each red one's
  first error in its status.

## Decisions

- **One task,** as the request asks and as wasm32-more was: the list is one file with one writer,
  and the measurements share one warm `target/`.
- **The nine are resolved in the spec,** from the status files' Not yet lines that cite wasm32
  (`grep -i wasm` over every status file finds three, and three `STATUS.md` summaries): skein's
  `skein-tls`, `skein-fake-forge-github`, `skein-fake-peers`; smith's `smith-protocol-events`,
  `-llm`, `-program`, `smith-agent-service`, `smith-agent-shell`; and `temper-playground`. All nine
  are packages at `6730de2602`. The request calls them left red by wasm32-more; the statuses have
  since moved: seven of the nine are recorded as passing and only left out, so the likely outcome
  is seven added, not nine re-measured reds.
- **Likely red, and the spec holds either way:** no build ran for this cut (the call has no
  builder), but every place the entries name stands in the source at `6730de2602`: `Source`'s and
  `Stored`'s `#[expect(clippy::large_enum_variant)]` are unconditional, where `skein-kv`'s own was
  scoped to 64-bit targets (`7362bb368a`), and `skein_tls as tls` is still imported outside
  `cfg(unix)`. So `smith-agent-shell` and `temper-playground` are expected red, each entry
  refreshed; `temper-playground` is green only if `smith-agent-shell` is. Those are their trees'
  repairs, not this plan's: the request forbids code changes, and no task here makes them.
- **One `--keep-going` run, the one addition to the request's commands:** `temper-playground`'s
  entry names places in three crates, and a plain run stops scheduling crates at the first failure,
  so a place it does not report may still fail. Refreshing the entry from a plain run alone would
  either drop a live red or keep an unmeasured one. The run is for that package alone, and the
  entry then says what stays unlinted above the failed crates, where it now says "alone" more
  widely than a run can show.
- **The status forms are new, not wasm32-more's:** its form said "fab's gate list", and
  `tests/boundary/tests/outward.rs` now fails that outside a plan's directory. The status entries
  already say "the gate's wasm32 list", which the spec keeps.
- **No design section is written, no survey or designer started:** `skein/docs/design/testing.md`,
  10, `smith/docs/design/shell.md`, 2.5 and `temper/docs/design/playground.md`, 9 already ask for
  these checks; only the status follows the code. The readers were checked directly: the list is
  read by `.config/gate/README.md` and the gate alone, the status files by the citations check
  alone, no citation names the labels this task may remove, and `smith/docs/design/**`'s one
  compiled document is `shell.md`, which is not touched.
- **The combined run's failure:** a red in a crate only the added packages bring in takes those
  packages out and reruns once. A red in the base list's own tree is told apart by one run of the
  base's lines: red there is the trunk's and reported; green there means an addition changes that
  crate's build by features, and since the request says the run must pass, that stops as a
  question rather than landing a red list. wasm32-more kept the list in both cases.

## Overlaps with the drafts beside this one

- **No file overlap with any listed branch.** The list and six status files are in no branch's
  files; `debt-7` edits skein's `shell.status.md` and `kv-chunk-target` `kv.status.md`, and
  `live-login-guard` temper's `domain/testing.status.md`, different files from smith's
  `shell.status.md` and skein's `testing.status.md`.
- **debt-7's code, unavoidable and one way:** `debt-7/smith-primitives` edits `smith/crates/**/*.rs`
  and `debt-7/smith-unused-deps` `smith-agent-shell`'s manifest, so its landing may move the lines a
  red entry names; a later status keeps them true. And the list reaches further once this lands:
  `debt-7/github-fake-surface` edits `skein-fake-forge-github`, and its smith branches the four
  added smith crates, so a debt-7 merge that goes wasm32-red fails its own gate step after this
  lands, where it would have passed before. That is the gate doing its work, and it is debt-7's to
  repair; this plan measures its own base and needs no order between them.
- The queued drafts (v0.1.0-ladder, reference-walks, configuring-walks) list no files yet.

## Critical path

gate-last alone: about 50 implementer-minutes, most of them nine wasm32 clippy builds through
`heavy`, the first few cold, one `--keep-going` run and one combined run on the warm `target/`.
