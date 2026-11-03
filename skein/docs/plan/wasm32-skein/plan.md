# wasm32-skein: plan

Draft: skein's crates pass wasm32 clippy (`inputs/request.md`). `temper-playground` cannot join
fab's gate list `.config/gate/wasm32.txt`: with jig's expectations scoped (plan wasm32-jig) and
rustls's `std` native only (plan wasm32-rustls), its wasm32 clippy next fails in skein, at
`#[expect(clippy::large_enum_variant)]` on `Request` in
`skein/crates/skein-llm-connection/src/boundary.rs:49`, unmet on wasm32, where sizes differ.

## Parts and tasks

- **wasm32-expects** (slice 1): `tasks/wasm32-expects.md`, no needs, released in the first and
  only push. Runs `temper-playground`'s wasm32 clippy without `-D warnings` to list every
  warning at once, scopes each skein expectation unmet on wasm32 to the host
  (`cfg_attr(not(target_arch = "wasm32"), expect(...))`, same lint and reason), runs the
  request's command with warnings denied, then `skein-tls`, `skein-fake-forge-github` and
  `skein-fake-peers` on their own, then the native clippy of the changed packages. Reports what
  fails outside skein, and which of the three pass.

## Decisions

- **One task, no design,** as the request says. The design already asks for the check:
  `skein/docs/design/testing.md`, 10 builds these crates for wasm32 and checks them "with
  clippy's lints as errors on this target". Its status entry (`testing.status.md`, 6, 10) names
  the gate list, which this plan does not change, so no status file changes.
- **The request's form, `not(target_arch = "wasm32")`,** for every new site. skein already has
  four sites scoped by `target_pointer_width = "64"` (in `skein-fake-forge-domain`,
  `skein-fake-forge-github` and `skein-fake-peers`); they hold on both targets and stay as they
  are, so the diff touches only what fails.
- **Attributes, and fences only where a fence leaves code dead.** Boxing a variant or changing a
  signature would change skein's public types and their readers in every tree. The request also
  allows making the code right for both; the one such case pre-approved is an item or `use`
  line only `cfg(unix)` code reaches, which takes its users' fence (testing.md, 10: "a fence
  that leaves code dead warns"). The host's build stays exactly as it was, so no test runs
  (`notes/common.md` states the departure); the native clippy is the host's check.
- **The candidates are where the gate does not look yet.** Of the 29 skein crates
  `cargo tree --target wasm32-unknown-unknown -p temper-playground` lists, seven (lib, codec,
  io, http, shell, sim, sim-world) are in the graph of the gate list's packages and pass its
  wasm32 step. Of the other 22, a grep finds ten size-dependent expectations not yet scoped:
  `large_enum_variant` in skein-llm-connection, skein-forge-setup (three), skein-oauth-accounts
  (two) and skein-fake-peers (two), and `result_large_err` in skein-llm and
  skein-fake-forge-protocol. The run decides which are unmet.
- **The three crates the request names** are all in `temper-playground`'s wasm32 graph, so the
  main run checks them under its features; step 5 checks them under their own, in one
  `--keep-going` run, since `skein-fake-peers` depends on the other two. Their skein warnings
  are handled like the rest; the report says which pass.
- **The survey run goes first without `-D warnings`:** clippy then checks every crate and shows
  every warning in one pass, where a denied run stops at the first failing crate's dependents.
  `--keep-going` on the denied runs for the same reason. skein depends on no other tree, so no
  failure elsewhere hides a skein crate.
- **Other trees are reported, not fixed:** jig, smith and temper code failing wasm32 clippy
  goes in the report's `## Left`, as the request says. Plan playground-browser owns
  `temper-client-shell-web`, `temper-playground` and the list.
- **`old_forms.py` runs from `skein/`:** run from the repository root it looks for
  `scripts/old-forms.txt` there and asks for `--write`.

## Overlaps with the drafts beside this one

- No draft beside this one lists a file of this task. Under `skein/crates/`, the others list
  `skein-ui-web` (public-reading-build, playground-browser) and
  `skein/crates/skein-tls/Cargo.toml` (playground-browser's merged playground-worker); this task
  changes neither, and no `Cargo.toml`. Their trunks may change the sizes of skein types only
  through skein code, and none of them lists any.
- **wasm32-more** and **public-reading-build** edit `skein/docs/design/testing.status.md`,
  `world.status.md` and `STATUS.md`; this plan writes no status.
- The queued drafts (v0.1.0-ladder, sign-in-wait, reference-walks) and open-default list no
  files yet.

## Critical path

wasm32-expects: about 55 implementer-minutes, most of them three wasm32 clippy builds (two of
`temper-playground`'s tree, one of the three crates, mostly cached) and one native clippy.
