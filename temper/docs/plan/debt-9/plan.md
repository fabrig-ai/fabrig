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

## Still to come

- **transport** (D016): the design note, one `Link`, the four components on it.

## Questions for the user

Two, in `plan.toml`. Each task stops with a `Question:` until its question has an answer, for the request forbids
it to go on: this isn't a task waiting on fab, it's a task that may end early.

- **`github-media-type-assertions`:** the request approves one assertion moving and says no other does. Removing
  the switch reaches three.
- **`wrong-kind-meaning`:** the request says stop and ask if the design is silent. It is: jig's `service.md`, 11.1
  leaves the meaning to the owner, and temper's design names none.

## What the cut decided

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

## Overlaps

`smith-block-walks` edits the lines that name `Block::ToolCall` in `smith-protocol-program/src/backend/`, which
`smith-backend-effects` owns: a reader's edit on other lines, run side by side so neither slot waits.
