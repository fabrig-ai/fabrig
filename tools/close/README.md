# fabrig's close command

`<repository name>/.temper/hooks/close close <request.json>` settles a goal's
in-flight status entries (`DESIGN.md`, section 2) once every change of the goal
has ended, then checks the goal (temper/docs/design/protocol/tooling.md, 4.2;
temper/docs/design/domain/forge/branches.md, 2). It runs from a job directory
with the goal's branch checked out in `<repository name>/`, with no model or
credential. Settlement uses only Python's standard library and git, and no
network.

- **The request:** `{"version": 1, "goal": G, "changes": [{"task": N,
  "landed": BOOL}]}`, as temper writes it. Anything else, a task named twice
  among them, or a number that is no positive integer, is the command's
  failure.
- **What it reads:** every tracked file whose name ends `.status.md`,
  wherever it is in the repository (a tree's `docs/`, a tool's directory
  under `tools/`, or any other), in the repository's index, at the
  checked-out tree.
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
- **A tag may wrap:** each of its spaces, the one before its `[`, the
  one after `[in`, the one after `flight:` and each within TEXT, may
  instead be a line break between two of its entry's lines: a line's
  end, with any spaces or tabs before it, and the next line's
  indentation. Its `]` and what follows it are on the entry's last
  line. TEXT reads each break as one space, so `... [in flight:` with
  `  change 15531].` on the line after names change 15531, as do
  `... [in` with `  flight: change 15531].` and `... [in flight: change`
  with `  15531].`; a wrapped tag is settled as a one-line one is.
- **A bare `- **Not yet:**` label** groups nested `  - ` items. Each
  item, with its continuation lines, is settled separately; an untagged
  sibling stays. The label goes only when all its items have been deleted.
- **What it does to a tagged entry:**
  - its change landed (`"landed": true`): the entry is deleted, its lines
    whole, nothing else of the file moved;
  - its change ended without landing (`"landed": false`): the tag is
    dropped, with the one space before it, and the punctuation and spaces
    after it stay (`gap [in flight: change 12].` becomes `gap.`), so the gap
    stays recorded. A wrapped tag goes with its breaks: the line holding
    the space before it keeps what precedes that space, then what follows
    the `]` with the last line's ending, its other lines gone (`gap [in
    flight:` with `  change 12].` after it becomes `gap.`, and so does
    `gap` with `  [in flight: change 12].` after it);
  - its tag names no change of the request: the entry is left as it is,
    byte for byte, and the result lists the tag as kept. Whoever owns a
    tag settles it: the command never removes or rewrites another goal's.
- **A file it leaves with no entry says so** (`DESIGN.md`, section 2).
  Where settling a file leaves nothing but blank lines after its opening,
  its heading and the dated paragraph after it as
  `tools/citations/README.md` reads them, the file becomes that opening,
  byte for byte, then one blank line, then `No gap.` and a line ending:
  `# x: status`, a blank line and `2026-11-20. Where ...` with only
  `- **Not yet:** 3: built [in flight: change 9].` after them become
  `# x: status`, a blank line, `2026-11-20. Where ...`, a blank line and
  `No gap.`. The lines it writes end as the opening's last line does,
  `\r\n` or `\n`, and that line gains a `\n` when it has no ending. Any
  other text left after the opening stays as settling left it: a
  `No gap.` line already there is kept and never doubled, and a file left
  with an entry, or with prose, gains none. A file with no such opening
  gains none, and neither does a file in which it settled no tag, empty
  or not.
- **A file it changed** is staged with `git add`, its date line as the
  goal's design dated it. It never commits, never writes another branch,
  and changes no other file.
- **`$TEMPER_JOB/result.json`, exit 0:** `{"settled": [PATH]}`, the files it
  staged, sorted, and, when it kept a tag, `"kept": [TAG]` after them:
  each tag it kept as `PATH:LINE: [in flight: TEXT]`, LINE the 1-based
  number of the line holding the tag's `[`, TEXT with each break read
  as one space, sorted by path, then line:
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

## Goal validation

The close hook runs `tools/close/close.py` to settle status, then runs
`tools/pre-pr/pre_pr.py --close` from the repository root, even when settlement
stages nothing. The older `.temper/close` and `.temper/checked-close` both
forward to the hook. The check compares the goal's tree with the newest
published main commit that the goal contains (`tools/pre-pr/design.md`, 1)
and returns its exit status to the worker. A missing verifier, unavailable
comparison, failed check or expired job prevents successful close. The worker still
owns committing and pushing the settled tree. Where the worker asks for a
typed refusal, `TEMPER_CLOSE_REFUSAL=3` in the environment, a refusal
replaces settlement's result with its own and exits 3, and an error of
the check's own exits 2 (`tools/pre-pr/design.md`, 4.1); without it,
either exits 1. A settlement that fails stops before the check, exit 1.
