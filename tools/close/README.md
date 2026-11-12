# fabrig's close command

`.temper/close close <request.json>` settles a goal's in-flight status entries
(`DESIGN.md`, section 2) once every change of the goal has ended, before its
closer lands the goal (temper/docs/design/protocol/tooling.md, 4.2;
temper/docs/design/domain/forge/branches.md, 2). It runs from a job directory with
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
  item, blank line or heading. **Its tag** is the text `[in flight: TEXT]`
  that ends it, after a space, with nothing after it on its last line but
  a run of sentence punctuation straight after the bracket (any of `.`,
  `,`, `;`, `:`, `!`, `?`) and then spaces: `... [in flight: change 161].`
  ends with a tag; `... [in flight: change 161] .`,
  `... [in flight: change 161]).` and `... [in flight: change 161] more`
  do not. The tag names change N of the request when TEXT is
  `change N`, N a decimal integer and a task of the request; any other
  tag, another goal's or one of another form (`[in flight: some-name]`),
  names no change of it. An entry with no tag is left as it is.
- **A bare `- **Not yet:**` label** groups nested `  - ` items. Each
  item, with its continuation lines, is settled separately; an untagged
  sibling stays. The label goes only when all its items have been deleted.
- **What it does to a tagged entry:**
  - its change landed (`"landed": true`): the entry is deleted, its lines
    whole, nothing else of the file moved;
  - its change ended without landing (`"landed": false`): the tag is
    dropped, with the one space before it, and the punctuation and spaces
    after it stay (`gap [in flight: change 12].` becomes `gap.`), so the gap
    stays recorded;
  - its tag names no change of the request: the entry is left as it is,
    byte for byte, and the result lists the tag as kept. Whoever owns a
    tag settles it: the command never removes or rewrites another goal's.
- **A file it changed** is staged with `git add`, its date line as the
  goal's design dated it. It never commits, never writes another branch,
  and changes no other file.
- **`$TEMPER_JOB/result.json`, exit 0:** `{"settled": [PATH]}`, the files it
  staged, sorted, and, when it kept a tag, `"kept": [TAG]` after them:
  each tag it kept as `PATH:LINE: [in flight: TEXT]`, LINE the 1-based
  number of the line holding the tag, sorted by path, then line:
  `{"settled":[],"kept":["jig/docs/design/domain/engine/briefs.status.md:42: [in flight: change 232]"]}`.
  It is compact JSON (`,` and `:` with no spaces, ASCII escapes) ending
  in a newline, at most 512 bytes. A result past 512 bytes is cut as the
  merge command's is (`tools/merge/README.md`): it lists the longest
  prefix that fits of the paths followed by the tags, in that order,
  with `"more": N`, the count of paths left out, and `"kept_more": M`, the
  count of tags left out, each present only when not 0; `"kept"` stays,
  empty if none fits, whenever a tag was kept. Nothing to settle is exit 0
  with `{"settled": []}`, its `"kept"` when it kept a tag, and nothing
  staged.
- **Any other exit,** 1, is its failure, the reason on stderr; it stages
  nothing then.
- **Deterministic:** the same tree and request stage the same tree, and a
  second run over its own result stages nothing.

Run its tests with `python3 -I -B tools/close/test_close.py`: the focused
cases and the fuzzy one, which settles random status files against random
change ends, each seed printed so that it replays (`--seed N`).
