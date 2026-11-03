# wasm32-jig: plan

Draft: jig's crates pass wasm32 clippy (`inputs/request.md`). `temper-playground` cannot join
fab's gate list `.config/gate/wasm32.txt` (plan wasm32-temper measured it red): its wasm32
clippy fails in its jig dependencies, where lint expectations met on the host go unmet on
wasm32, whose sizes differ.

## Parts and tasks

- **wasm32-expects** (slice 1): `tasks/wasm32-expects.md`, no needs, released in the first and
  only push. Runs `temper-playground`'s wasm32 clippy without `-D warnings` to list every
  warning at once, scopes each jig expectation unmet on wasm32 to the host
  (`cfg_attr(not(target_arch = "wasm32"), expect(...))`, same lint and reason), runs the
  request's command with warnings denied, then the native clippy of the changed packages.
  Reports what fails outside jig.

## Decisions

- **One task, no design,** as the request says. The design already asks for the build
  (`temper/docs/design/playground.md`, 9); no jig status entry names these lints, and jig's
  entries on the gate list (`protocol/browser.status.md`, `protocol/README.status.md`) name the
  list, which this plan does not change.
- **Attributes only.** The request prefers the `cfg_attr` form; boxing a variant or changing a
  signature would change jig's public types and their readers across trees. So the host's build
  stays exactly as it was, and no test runs (`notes/common.md` states the departure); the native
  clippy is the host's check.
- **A lint that fires on wasm32 only** (pointer-width-dependent, such as
  `trivially_copy_pass_by_ref`) is pre-approved the mirror form,
  `cfg_attr(target_arch = "wasm32", expect(...))`; anything else in jig is a question.
- **The survey run goes first without `-D warnings`:** clippy then checks every crate and shows
  every warning in one pass, where a denied run stops at the first failing crate's dependents.
  `--keep-going` on the denied run for the same reason.
- **Other trees are reported, not fixed:** skein, smith and temper code failing wasm32 clippy
  goes in the report's `## Left` (the request: name temper's; plan playground-browser owns
  `temper-client-shell-web`, `temper-playground` and the list).
- The candidate sites (23 size-dependent expectations in jig-domain-core, jig-domain-people,
  jig-domain-tasks, jig-domain-hub-link and jig-protocol-store) come from a grep over the 24 jig
  crates `cargo tree --target wasm32-unknown-unknown -p temper-playground` lists; the run
  decides which are unmet.

## Overlaps with the drafts beside this one

- **configuring-p2-build** (integration, not on main): its merged tasks touch three files with
  candidate sites. `restore-surface` adds `Ask::Restore` right after `pub enum Ask {` in
  `jig/crates/jig-domain-people/src/boundary.rs`, one line below the attribute this task
  rewrites; its trunk also adds an arm far from `Work` in `jig-domain-core/src/vocabulary.rs` and
  constants below the inner attribute of `jig-protocol-store/src/codec/settings.rs`. Unavoidable:
  the attributes are where the request points. The hunks do not share lines. Its new variants
  can change sizes, so a site may flip once both land; the gate's wasm32 step does not clippy
  `temper-playground` until it joins the list, whose task measures it again.
- **playground-browser** (integration): owns `temper-playground`, `temper-client-shell-web` and
  `.config/gate/wasm32.txt`; this plan touches none of them.
- The queued drafts (v0.1.0-ladder, sign-in-wait, reference-walks) list no files yet.

## Critical path

wasm32-expects: about 45 implementer-minutes, most of them three wasm32 clippy builds of
`temper-playground`'s tree and one native clippy.
