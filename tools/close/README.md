# fabrig's close command

`.temper/close close <request.json>` settles a goal's in-flight status entries
(`DESIGN.md`, section 2) once every change of the goal has ended, before its
closer lands the goal (temper/docs/design/protocol/tooling.md, 4.2;
temper/docs/design/domain/forge.md, 11.2). It runs from a job directory with
the goal's branch checked out in `<repository name>/`, with no model, no
credential and no network, and needs only Python's standard library and git.

- **The request:** `{"version": 1, "goal": G, "changes": [{"task": N,
  "landed": BOOL}]}`, as temper writes it. Anything else, a task named twice
  among them, or a number that is no positive integer, is the command's
  failure.
- **What it reads:** every tracked file whose name ends `.status.md` under a
  `docs/` directory, in the repository's index, at the checked-out tree.
- **An entry** is a list item of a status file opening `- **Not yet:**`,
  with its continuation lines, those indented beneath it, to the next list
  item, blank line or heading. **Its tag** is the text `[in flight: change N]`
  that ends it, N a positive decimal integer, with nothing but spaces after
  it on its last line. An entry ending `[in flight: ANY]` of another form
  has a tag naming no change of the request, which fails. An entry with no
  tag is left as it is.
- **What it does to a tagged entry:**
  - its change landed (`"landed": true`): the entry is deleted, its lines
    whole, nothing else of the file moved;
  - its change ended without landing (`"landed": false`): the tag is
    dropped, with the one space before it, so the gap stays recorded;
  - its change is not in the request: the command fails, naming the file,
    the line and the tag, and writes nothing.
- **A file it changed** is staged with `git add`, its date line as the
  goal's design dated it. It never commits, never writes another branch,
  and changes no other file.
- **`$TEMPER_JOB/result.json`, exit 0:** `{"settled": [PATH]}`, the files it
  staged, sorted, cut to 512 bytes with `"more": N` as the merge command's
  result is (`tools/merge/README.md`). Nothing to settle is exit 0 with
  `{"settled": []}` and nothing staged.
- **Any other exit,** 1, is its failure, the reason on stderr; it stages
  nothing then.
- **Deterministic:** the same tree and request stage the same tree, and a
  second run over its own result stages nothing.

Run its tests with `python3 -I -B tools/close/test_close.py`: the focused
cases and the fuzzy one, which settles random status files against random
change ends, each seed printed so that it replays (`--seed N`).
