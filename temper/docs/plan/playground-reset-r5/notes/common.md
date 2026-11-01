# Standing rules for every task of playground-reset-r5

Cited by every spec of `temper/docs/plan/playground-reset-r5/tasks/`. The
draft is `temper/docs/design/draft/playground-reset.md`; the run's binding
shapes are `temper/docs/plan/playground-reset-r5/notes/contracts.md`, and
where a spec and the contracts disagree, the contracts win and the report
says so.

## Machines

- The builders are hetzner6 (hostname `debian-16gb-hel1-6`) and hetzner9.
  hetzner6's hostname contains "hel1", but it is a builder, not hel8: build
  there through `heavy` as on any builder.

## What is whose

- **The draft is not adopted.** No task edits `docs/design/` (the design
  documents, their status files, the draft) or `docs/help/`; what the code
  taught goes in the report, under `## Design`. The one exception is a
  task whose spec says `kind = "docs"` and names the files.
- **One writer per crate per wave.** Each spec lists what it owns and what
  else it may touch, and how. A file outside both lists is a question,
  except the always-allowed edits of the implementer's text (arms, fields
  and literals the compiler requires, a scoped `#[expect]`).
- **Store layouts are in place until the first release** (temper's
  `protocol/store.md`, section 4; jig's `protocol/store.md`): a new family
  or variant bumps the owner's codec version as the store design says and
  needs no migration of old stores.
- **Old names go whole.** A renamed or removed item (`Demo` → `Playground`,
  `StartDemo` → `StartPlayground`, the document's forge members) leaves no
  alias, shim or `#[deprecated]` behind; its readers move in the same task
  when the spec lists them, else they are the next task's and the report
  names them under `## Left`.

## Tests

- **Fakes and worlds:** use the tree's existing worlds and kits (temper's
  `temper/tests/*` worlds, jig's `jig/tests/*` worlds and
  `jig/testing/*` kits, skein's fake forge and fake LLM). A new test goes
  into the world the spec names; no new test crate unless the spec says so.
- **Fixtures:** a test that needs a forge makes it the way the contracts
  say forges are made (by its record, through the connector's acts or the
  test kits' builder for it), never by a document member once
  `forge-doc-out` has merged.
- **A change of meaning runs its readers' tests:** where a spec says an
  item changes meaning, run the focused tests of each package the spec
  names as its readers, not only the owner's.
- Every build and test goes through `heavy`, one command at a time,
  `heavy cargo nextest run -p PKG --test BIN -E 'test(=NAME)'` for the
  narrow one; check the count; never the whole workspace.
- Old-form counts (`scripts/old-forms.txt` per tree) only fall: run the
  tree's `python3 -I -B scripts/old_forms.py` from the tree's root before
  pushing when you removed or added code in it.

## Report

As the implementer's text says, plus `## Left` (what stays outside your
scope, file and why) and `## Design` (what the draft or the contracts got
wrong, or left open, and what you chose).
