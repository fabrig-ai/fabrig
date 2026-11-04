# debt-9: three independent parts of debt

The request has three parts: skein D016, the user's two follow-ups, and four smith items. `Failed: cut short`
stands in the report until every part is here.

## Released

- **github-media-type** (the request's part 2): `tasks/github-media-type.md`, medium, ~35 minutes, needs nothing.
- **temper-wrong-kind** (part 2): `tasks/temper-wrong-kind.md`, medium, ~40 minutes, needs nothing.
- **smith-one-program** (D008): `tasks/smith-one-program.md`, medium, ~50 minutes, needs nothing.
- **smith-backend-effects** (D006): `tasks/smith-backend-effects.md`, high, ~100 minutes, needs nothing.
- **smith-block-walks** (D005): `tasks/smith-block-walks.md`, high, ~180 minutes, needs nothing.
- **smith-names** (D013): `tasks/smith-names.md`, medium, ~90 minutes, needs D005 and D006.

- **transport** (D016), `skein/docs/design/link.md`, written with this batch: `tasks/link-crate.md`, high, ~110 minutes,
  needs nothing; then, side by side, one writer per crate, `link-oauth` (~70), `link-llm` (~80), `link-forge-setup` (~75)
  and `link-forge-connection` (~120), all high; then `link-close`, docs, ~20.

## Questions for the user

Three, in `plan.toml`. On the first two the task stops with a `Question:` until there's an answer, for the request
forbids it to go on: that isn't a task waiting on fab, it's a task that may end early. The third stops nothing.

- **`github-media-type-assertions`:** the request approves one assertion moving and says no other does. Removing
  the switch reaches three.
- **`wrong-kind-meaning`:** the request says stop and ask if the design is silent. It is: jig's `service.md`, 11.1
  leaves the meaning to the owner, and temper's design names none.

- **`link-role`:** `skein-link` takes the protocol layer's row, so `draft/component-kit.md`, 6.2 stays open. That
  draft calls a new role the user's, so the user should see this reading. Nothing waits on it.

## What the cut decided

- **D016's own words are wrong in one place.** It says oauth-accounts and llm-connection send `close_notify`. By
  reading, only oauth-accounts delivers it: llm-connection's `route` acts on TLS's `Closed` before it moves the `Send`
  beside it, and io drops a stream request once closing. `link-llm` reproduces that first, and this is why `pump`
  moves ciphertext first, always.
- **The close needed a bound that the item doesn't mention.** The forge pair's close is bounded today because it goes
  straight to io. A TLS close waits for `Room` first, and nothing bounds that, so a stalled peer would hold a slot for
  good, against `forge-connection.md`, 8.1 and `forge-setup.md`, 6.5. So `Link` holds a deadline of io's
  `close_timeout`. oauth-accounts and llm-connection have the same gap today, and gain the same bound.
- **`forge-setup.md`, 6.5's bound on a run's slot doubles,** to twice `close_timeout`, as the link's wait comes before
  io's. It only shows against a stalled peer.
- **`Link` is a new crate:** io may not name TLS, a machine may not name io, and a composition names no io.
- **`pump` makes one move and `close` at most one io request,** because forge-setup reserves room for one.
- **`Upload` is beside the link, not in it,** as a body may go through HTTP's framing. llm-connection has none: its
  upload is `skein-llm`'s.

- **The fake GitHub's design moved with the first push:** `fake-forge.md`, 9's open question closes and 10.6 says
  one spelling. `fake-forge.status.md` holds a **Not yet** that `github-media-type` closes.
- **D006 keeps `Queue<Below>` and its public step signatures,** so `smith-agent-service` doesn't change. The price
  is that a step's io and file requests now interleave in causal order. By reading, `down` splits them at once, so
  only a stalled file's head can tell; the spec makes any moved test a stop.
- **D006's `server_up` keeps capacity 2** and moves to its owner, with the bound reading the capacity. `while let`
  is barred in step code, and `0..len()` would drop an `Up` pushed during the pass.
- **D008 keeps `ProgramName`:** the local shell's credentials name it, and the item asks only that the maps go.
- **D005's measure empties on the shape change alone,** so the spec's acceptance also reads the field patterns left.
- **temper's fix, if the answer is to stop, compares one token,** so a real `Corrupt` on an accounts write stays as
  it is: that's a wider gap than the request names.

- **D013 leaves three uses standing, and names them:** the rendered prompt's "## Checkout" text is what the LLM
  reads, and the effective limits' `adapter.tool_bytes` and `adapter.history_bytes` are reported names, so each would
  be a visible change the request doesn't allow. `limits.llm.adapter` becomes `translation`, the word its doc and
  `fixture.rs` already use.
- **D013 runs last in smith** because it renames across crates D005 and D006 own.

## Critical path

`link-crate` → `link-forge-connection` → `link-close` (~250 implementer-minutes), beside `smith-block-walks` →
`smith-names` (~270).

## Overlaps

`link-crate` adds lines to the root `Cargo.toml` and `Cargo.lock`, and `hearth-workspace`'s `join-worlds` does too;
fab's merges meet there, in different lines. `smith-block-walks` edits the lines that name `Block::ToolCall` in `smith-protocol-program/src/backend/`, which
`smith-backend-effects` owns: a reader's edit on other lines, run side by side so neither slot waits.
