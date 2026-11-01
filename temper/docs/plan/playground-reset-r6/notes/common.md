# Standing rules for every task of playground-reset-r6

Every spec of this plan names this note. Read it after fabrig's `AGENTS.md` and your tree's
`AGENTS.md` (`temper/AGENTS.md`, `jig/AGENTS.md`).

## What the run builds

temper's draft `temper/docs/design/draft/playground-reset.md`, sections 3 to 7: forges as
records in temper's store, a removable forge, jig's deletable project, and the playground and
its reset on them. The draft is the design for this run; it is not yet folded into
`docs/design/`. Where a spec or `notes/contracts.md` settles a shape the draft leaves open,
the spec and the note win. The draft's open questions (its section 9) take their defaults.

## Binding shapes

`temper/docs/plan/playground-reset-r6/notes/contracts.md` holds every shape that crosses a
task, a crate or a tree: names, fields, signatures, record layouts, wire forms, pipe lines.
Build to it exactly; a shape it names that you think wrong is a question, never a quiet
change.

## Machines

The builders are fab's. hetzner6's hostname (`debian-16gb-hel1-6`) contains "hel1": it is a
builder, not hel8. Build and test only through `heavy`, one command at a time.

## Not yours, in every task

- `docs/design/`, the drafts, `docs/help/`, and this plan's directory: what the code taught
  you goes in your report.
- Another task's owned files, beyond the edits your spec's `Owns and may change` allows.
- Process-tier tests (binary `process` of package `temper`, `qa_*` binaries): never run them
  unless your spec says so; they are not in the gate. A spec that adds one says how to run it
  (`heavy cargo nextest run -p temper --profile process-temper --test process -E 'test(=NAME)'`).

## Allowed in every task without asking

- The arms, fields and literals the compiler requires because of a type your task changed,
  in any file of your tree, each edited only for that (the implementer's text).
- **Across trees, the same, for jig's types temper matches:** a jig task that adds a variant
  to an enum or a field to a struct that temper's code matches or builds exhaustively also
  edits those temper lines, each only to route the new variant to a refusal, a no-op or the
  nearest existing arm, or to fill the field with its neutral value; the report lists each
  file. This departs from "a branch stays in its tree" so that the trunk keeps building.
- Test fixtures and golden files that your changed code regenerates
  (`SKEIN_CODEGEN_WRITE=1` for schema-generated code, as `temper/tests/codecs` and
  `jig/tests/store-codecs` print), in the crates you own.

## Tests

- Narrowest first (`AGENTS.md`, 6); the spec names the binaries. Read a failure's log
  before anything reruns.
- A change of meaning runs its readers' tests: the spec lists the packages.
- Worlds are ordinary Rust under the tree's `tests/clippy.toml`: one thread, one loop, a seed
  replays.

## Old forms

`temper/scripts/old-forms.txt` and `jig/scripts/old-forms.txt` hold counts that only fall.
Run `python3 -I -B scripts/old_forms.py` from the tree's root before you push; never raise a
count. If your change removes forms, lower them with `--write`.

## Report

As the implementer's text says. Also list under `## Left` every reader you found that still
names a removed or renamed item in a file you do not own.
