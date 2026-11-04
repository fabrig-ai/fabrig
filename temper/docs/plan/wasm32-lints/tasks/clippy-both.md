+++
id = "clippy-both"
part = "clippy-both"
kind = "feature"
needs = []
files = [
  ".config/gate/wasm32.txt",
  "jig/crates/jig-domain-core/src/public/index.rs",
  "jig/crates/jig-domain-hub-link/src/down.rs",
  "jig/crates/jig-domain-hub-link/src/lib.rs",
  "jig/crates/jig-protocol-store/src/codec/settings.rs",
  "smith/crates/smith-local-service/src/credentials.rs",
  "smith/crates/smith-agent-shell/src/config/native.rs",
  "smith/crates/smith-agent-shell/src/config/document.rs",
  "smith/crates/smith-agent-shell/src/config/endpoints.rs",
  "temper/crates/temper-engine-domain/src/admission.rs",
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/decision.rs",
  "temper/crates/temper-engine-domain/src/store.rs",
  "temper/crates/temper-engine-domain/src/translate.rs",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/views.rs",
  "temper/crates/temper-engine-smith/src/lib.rs",
  "temper/crates/temper-service/src/setup/lifecycle.rs",
  "temper/crates/temper-service/src/stock.rs",
  "temper/crates/temper-service/src/worker_git.rs",
  "smith/docs/design/shell.status.md",
  "smith/docs/design/STATUS.md",
  "temper/docs/design/playground.status.md",
  "temper/docs/design/STATUS.md",
]
effort = "high"
minutes = 80
+++

# temper-playground and smith-agent-shell pass the wasm32 clippy and join the gate's list

## Goal

The designs build both packages for wasm32, but the gate's list leaves them out because their
wasm32 clippy fails, warnings denied. Make both pass, by lint attributes and `cfg`s alone, with no
change of behaviour on any target; add both to `.config/gate/wasm32.txt`; close the status entries.

Acceptance:

- `heavy cargo clippy --target wasm32-unknown-unknown -p smith-agent-shell -- -D warnings` passes.
- `heavy cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings` passes.
- `.config/gate/wasm32.txt` holds its base's lines unchanged and in order, then `smith-agent-shell`
  and `temper-playground` appended after the last line, one per line, the file ending with one
  newline.
- `heavy cargo clippy --target wasm32-unknown-unknown -p <each package line of the list, in order>
  -- -D warnings` passes, run once over the final list.
- `heavy cargo clippy -p <every package you changed> --all-targets -- -D warnings` passes: native
  clippy stays green.
- `heavy cargo nextest run -p smith-agent-shell` passes, its count reported.
- The diff adds no `#[allow]`, and every expectation it scopes keeps its `reason` word for word.
- The status files read as `## Status edits` says.
- `python3 -I -B tools/citations/citations.py --base origin/main` prints no finding this branch
  adds.

## Inputs

- `temper/docs/plan/wasm32-lints/inputs/request.md`: the request.
- `smith/docs/design/shell.md`, 2.5: what of `smith-agent-shell` is portable (the configuration's
  decoding and checks from its bytes) and what is native under `cfg(unix)` (trust roots).
- `temper/docs/design/playground.md`, 9, "The wasm32 builds"; `skein/docs/design/testing.md`, 10:
  what the gate's list checks. Read only.
- `.config/gate/README.md`, `The gate's lists`: who adds a line to the list.
- `AGENTS.md`, 2: a lint exception is scoped, never `#[allow]`.
- `DESIGN.md`, 2: a status file's form. It holds only the gaps as they stand after your change.

**Departures from the implementer's text,** each because the request asks it or nothing else
reaches it:

- The branch crosses trees (jig, smith, temper, `.config`): the request asks for one task.
- You edit the four status files listed under `## Owns and may change`, as `## Status edits` says,
  and no design document (`<name>.md`), draft or `docs/help/`.
- Tests: `smith-agent-shell`'s alone, no direct dependents and no `cargo check -p <dependent>
  --tests`. On a native build the forms below change no compiled code: `cfg_attr(not(target_arch =
  "wasm32"), expect(..))` becomes the `expect` it was, `cfg_attr(not(unix), ..)` becomes nothing,
  and `#[cfg(unix)]` keeps its item. The trunk's end check runs the wider set. A form D edit runs
  the tests of the package it changes as well.
- No test is added, so the implementer text's "must bite" has nothing to bite: the wasm32 clippy
  is the check.

## What is measured and what is not

The request and the status files name what the wasm32 clippy reported, and it stopped there. cargo
does not build a dependent of a crate that failed, so the request's "in jig's and smith's crates"
covers only what was reported. These 17 packages have never been linted on wasm32:

`jig-domain-hub-link`, `jig-protocol-accounts`, `jig-protocol-charter`, `jig-protocol-config`,
`jig-protocol-engine-clients`, `jig-protocol-engine-clients-http`, `jig-protocol-store`,
`temper-box`, `temper-engine-domain`, `temper-engine-domain-forge`, `temper-engine-smith`,
`temper-fake`, `temper-forge-setup`, `temper-playground`, `temper-protocol-records`,
`temper-protocol-store`, `temper-service`.

They depend on `jig-domain-core`, `smith-local-service` or `smith-agent-shell` (`cargo metadata
--filter-platform wasm32-unknown-unknown`, at `98385b96c1`). Expect more of the same as each layer
compiles. These unconditional `#[expect]`s name lints that depend on the target's sizes, so they
are the likely ones; none has been measured, and any may stay as it is:

| Package | Place | Lint |
|---|---|---|
| `jig-domain-hub-link` | `src/down.rs:9`, `src/lib.rs:344` | `result_large_err`, `large_enum_variant` |
| `jig-protocol-store` | `src/codec/settings.rs:2` | `trivially_copy_pass_by_ref` |
| `temper-engine-domain` | `src/boundary.rs:288`, `:855`, `src/store.rs:555` | `large_enum_variant` |
| `temper-engine-domain` | `src/admission.rs:146`, `src/decision.rs:315`, `:401`, `:430`, `:536`, `src/translate.rs:475` | `result_large_err` |
| `temper-engine-domain-forge` | `src/views.rs:15`, `src/boundary.rs:1024`, `:1661` | `large_enum_variant` |
| `temper-engine-smith` | `src/lib.rs:55` | `large_enum_variant` |
| `temper-service` | `src/stock.rs:22`, `src/setup/lifecycle.rs:117` | `large_enum_variant` |
| `temper-service` | `src/worker_git.rs:89` | `result_large_err` |

**Change only what the wasm32 clippy reports.** An `#[expect]` it does not report stays plain: its
lint is met on both targets, and scoping it would lose that check. `temper-playground/src/worker.rs`
is wasm32-only and has never been linted at all.

## Owns and may change

**Owned:** `.config/gate/wasm32.txt`; the attributes named in `## The named items`; the four status
files of `## Status edits`.

**May change, reader and neighbour files:** any non-test `src/**/*.rs` of a package the two clippy
runs reach, in these forms alone. Each edit is the attribute or `cfg` and nothing beside it, and
fits one of them:

- **A. An expectation met natively, unmet on wasm32:** `#[cfg_attr(not(target_arch = "wasm32"),
  expect(<lint>, reason = "<its reason>"))]`, as
  `jig/crates/jig-domain-core/src/routing/effect/procedure.rs:21`. A `#![expect(..)]` becomes
  `#![cfg_attr(..)]`, as `jig/crates/jig-domain-core/src/runs/delegation.rs:3`. One attribute
  naming several lints, only some unmet, splits in two.
- **B. An item only `cfg(unix)` code reads:** the item takes that `cfg`. Not in jig or skein, and
  not where portable code constructs or decodes it (form C).
- **C. Something portable code keeps but only `cfg(unix)` code reads, or a lint that fires on
  wasm32 alone:** the same exception, scoped to that target: `#[cfg_attr(not(unix), expect(dead_code,
  reason = "..."))]`, or `#[cfg_attr(target_arch = "wasm32", expect(<lint>, reason = "..."))]`.
- **D. In smith and temper only, where A to C fit nothing:** a local reshape that changes no
  behaviour on any target and no public signature. Report each one.

**jig is shared with hearth:** in `jig/` change lint attributes alone, forms A and C, no behaviour.
skein's crates were measured and are not expected to appear; if one does, it takes the same rule.

**Pre-approved options:**

- If a parameter's lint attribute is not honoured, put it on the function instead.
- A report in a package or file the table omits needs no asking: the forms hold for every package
  the runs reach.
- If a package needs what no form allows (a behaviour change, a public signature, a compile error
  on wasm32, jig beyond its attributes): that is a question. Push what you have first; leave the
  list and the status files untouched for the package that stays red.

## The named items

Each was read at `98385b96c1`.

- `jig/crates/jig-domain-core/src/public/index.rs:115`, `:130`: form A, on `Index::publish` and
  `Index::follow`.
- `smith/crates/smith-local-service/src/credentials.rs:27`: form A, on `Source`. Its
  `#[expect(missing_debug_implementations, ..)]` stays.
- `smith/crates/smith-agent-shell/src/config/native.rs`:
  - `:10`, `use skein_tls as tls;`: form B, `#[cfg(unix)]`; everything naming `tls` there is.
  - `:74`, `const CERTIFICATES`: form B, `#[cfg(unix)]`. `TRUST_BYTES` stays: it is exported on
    every target.
  - `:31`, the parameter `errors`: form C.
    `#[cfg_attr(not(unix), expect(unused_variables, reason = "only the cfg(unix) trust loading
    writes diagnostics"))]`. It keeps its place on every target: `config.rs:251` calls `endpoints`
    from `Decoded::resolve`, public everywhere, and a call's argument can take no `cfg`.
- `smith/crates/smith-agent-shell/src/config/document.rs:183`, `TrustDocument`: form C, on the
  struct, `reason = "only the cfg(unix) trust loading reads these; decoding keeps one document on
  every target"`. Its fields stay: with `deny_unknown_fields`, fencing them would make wasm32
  refuse keys it decodes today (`shell.md`, 2.5: decoding is portable).
- `smith/crates/smith-agent-shell/src/config/endpoints.rs:39`, `EndpointTransport::Tls`: form C, on
  the variant, `reason = "only the cfg(unix) trust loading reads these; the checks that build them
  are portable"`. Its fields stay: `endpoint_transport` builds them, and `tls::Name::new` is one
  of the portable checks. The `#[cfg(not(unix))]` arm at `native.rs:48` stays as it is.

## Status edits

Each file's date line takes today's date (`date -u +%F`). Reflow a paragraph you edit; every other
word stands.

- `temper/docs/design/playground.status.md`, Not yet, 9: remove from `The wasm32 builds in the
  gate:` through `` `temper-playground` among them. `` The entry ends at `whether what lets the
  web client's walk pass lifts that is unknown.` The Departs entries stand.
- `smith/docs/design/shell.status.md`, Not yet: remove the whole `2.5:` entry.
- `smith/docs/design/STATUS.md`: `` the `--account` flag (`shell.md`, `testing.md`) and
  `smith-agent-shell` in the gate's wasm32 clippy, which it fails (`shell.md`), and the
  benchmarks' calibrations `` becomes `` the `--account` flag (`shell.md`, `testing.md`), and the
  benchmarks' calibrations ``.
- `temper/docs/design/STATUS.md`: `` whose story A walks in the browser tier but not yet by hand,
  whose download is past its bar and which the gate's wasm32 clippy leaves out (`playground.md`) ``
  becomes `` whose story A walks in the browser tier but not yet by hand and whose download is past
  its bar (`playground.md`) ``.

Status files are not plan files: `tests/boundary/tests/outward.rs` reads them, so what you write
there never names the tool that runs the gate. `smith/docs/design/**` is in
`.config/gate/read-by-tests.txt`, but the one test that compiles a document of that tree,
`smith/crates/smith-local-domain/src/parent_tests.rs`, includes `shell.md` alone, so no test needs
running for these edits.

## Steps

1. Make `## The named items`' edits.
2. Run `heavy cargo clippy --target wasm32-unknown-unknown --keep-going -p temper-playground -p
   smith-agent-shell -- -D warnings`. Fix what it reports by the forms above, then run it again:
   each round reaches the next layer of dependents, so expect several. Read each failure in its
   log; don't rerun unchanged.
3. Append the two lines to `.config/gate/wasm32.txt`.
4. Run the two clippy commands of the acceptance alone, then the whole list once.
5. Run the native clippy and `smith-agent-shell`'s tests.
6. Make `## Status edits`' edits; run the citations check.
7. `cargo fmt`; for each tree the branch touches, `python3 -I -B <tree>/scripts/old_forms.py`.
8. Commit in small commits that say why, one per tree's lints, then the list with the status files
   last. Push.

## Report

For each attribute or `cfg` changed: its file, line, lint and form, marking those the request did
not name, with the number of rounds step 2 took. Any form D reshape and why no other form fit. The
whole list's wall time and host: the list grows, and the user may want the figure. Any package left
red goes under `## Left` or in a question, as `## Owns and may change` says.
