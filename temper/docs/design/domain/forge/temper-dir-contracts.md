# `.temper/`'s goal: contracts between its changes

Provisional, 2026-11-28. The shapes the changes of goal W2
(`../../draft/ways.execution.md`) share and no other area reads, as
`../../draft/design-economy.md`, 4.3 keeps them: they bind as any
delivered design does, and the goal's last change deletes this document
once the code holds them. The lasting design is
`../../protocol/temper-dir.md` and this directory's `temper-dir.md`.

## 1. The judgement's crate

`temper-protocol-temper-dir`, in `temper/crates/`: step code under the
strict subset, `no_std`, pure. It holds what
`../../protocol/temper-dir.md` designs: which entries are read, the
reading of them at one commit, and the judgement. Its dependencies:
skein-lib, `skein-toml` (`skein/docs/design/data/toml.md`), skein-forge
for its vocabulary (`Commit`, `Read`, `Answer`, `Entry`, `EntryKind`
with `Link`), the crate holding jig's `is_name`, `temper-charter-set`
for a charter's front matter, and the hooks' `config.toml` reader
(`../../protocol/hooks.md`, 2.3).

### 1.1 Reading one commit

- **`Reading`:** one read of a repository's `.temper/` at one commit,
  made by its caller through the forge's client. `Reading::new(head:
  Commit, answer_bytes: u32)` starts it; `next(&self) ->
  Option<skein_forge::api::Read>` is the next read to send, always a
  `Read::File { head, path, max_bytes }` at that commit, `max_bytes` the
  least of the file's bound (`../../protocol/temper-dir.md`, 3) and
  `answer_bytes`; `take(&mut self, answer: &Answer) -> Result<(),
  Unread>` takes its answer.
- **What `take` keeps:** a `Directory` or `File` answer whose head and
  path are the call's; `Missing` for `.temper` itself as a tree with no
  `.temper/`, and for any other path as that entry gone. Any other
  answer, or one for another head or path, is `Unread`, and the caller
  ends the read as failed (`temper-dir.md`, 2).
- **Order:** `.temper`'s listing first; then each directory of
  `../../protocol/temper-dir.md`, 2's table it holds, in byte order;
  then each file that table reads, in byte order of its path. `next`
  is `None` once nothing is left, or once the files or bytes pass the
  judgement's bounds, which the judgement then names.
- **One owner at a time:** a caller holds one `Reading` per read it
  makes, in its own working set, and drops it on a restart.

### 1.2 The judgement

- **`Reading::judge(&self, home: Home) -> Verdict`,** once `next` is
  `None`. `Home` is `Here`, `Elsewhere { name }` (the home's
  `OWNER/NAME`) or `None`, the project having no home.
- **`Verdict`:** `method: bool`, whether `.temper/` holds `ways.toml`,
  `ways/` or `charters/` (`../../protocol/temper-dir.md`, 2); `faults`,
  at most 16 `Fault`s in the judgement's order; `more: u32`, the count
  past them. No fault is a good tree.
- **`Fault`:** `path`, the repository-relative path it names (`.temper/`
  for the tree as a whole); `line`, from 1, or 0 for the file or entry as
  a whole; `why`, its words after `path:line: `, as
  `../../protocol/temper-dir.md` gives them. `Fault::write` writes
  ``PATH:LINE: WHY``, or ``PATH: WHY`` at line 0.
- **`Reading::method_files(&self)`:** the paths and bytes of `ways.toml`,
  each way's file and each charter's file it read, in byte order of
  path: what the revision keeps (`temper-dir.md`, 3).

### 1.3 What a change touches

- **`touched(paths: &[Box<[u8]>], whole: bool) -> Touch`,** over a
  comparison's paths (skein's `Answer::Compare`'s `files`, `whole` its
  count matched):
  - `Method`: a path is `.temper/ways.toml`, or under `.temper/ways/` or
    `.temper/charters/`;
  - `Temper`: else a path is under `.temper/`;
  - `None`: else, `whole` true;
  - `Unknown`: else, `whole` false.

## 2. The merge's evidence

`ChangeEvidence` (in `temper-engine-domain-forge-vocabulary`'s
`boundary.rs`) gains `temper_dir: TemperDirEvidence`, beside `touched`,
read by the queue's and the turn's merge, and by the sign-off
(`temper-dir.md`, 5):

- `Uncovered`: the check doesn't cover the change at this head and base
  (`temper-dir.md`, 4);
- `Pending`: it covers it, and no pass names this head and base;
- `Passed { tree: Commit }`: a pass names this head and base, `tree` the
  commit whose `.temper/` it judged.

A step with no comparison read is `Pending` for a change the check could
cover, never `Uncovered`.
