# Render

Provisional, 2026-10-09, revised 2026-10-18. How hearth turns a revision
of an IaC repository and a target into a spec: what render reads and how
it reads it, its pipeline, the spec's parts and their digests, package
resolution, the rendered diff of a change, its errors, its crates and
how it is tested. The format it reads is hearth/docs/design/protocol/format/README.md. Milestone M1 builds
it, with the client's `render` and `diff` (hearth/docs/design/client/README.md, section 3). Open
questions are in section 7.

## 1. In one page

- **Render is a pure function of a revision and a target,** exports
  included. It reads nothing but the revision: no secrets, no machine, no
  clock, no network. Any hearth renders any target, in full, in
  milliseconds, so nothing about a spec is remembered but its digests.
- **The revision is read through git,** a tool behind a protocol machine
  (`hearth-protocol-git`): one listing, then the blobs render needs, every
  size bounded before a byte is read.
- **Eight steps:** decode, compose and merge, validate, evaluate exports,
  evaluate conditions, render files, detect collisions, assemble. The
  protocol layer decodes JSON and templates into typed values and
  instruction sequences; the domain never parses.
- **A spec is parts with digests.** Output parts (packages, software,
  config) enter layer keys; slot parts (data, secrets, boot) never do;
  checks, exports and live are the others; meta names the target. Each
  digest is SHA-256 over hearth/docs/design/domain/store/README.md's canonical encoding.
- **Resolution is the one slow step,** and it is remembered.
  `mmdebstrap --simulate --mode=chrootless` resolves a packages part to a
  pinned closure (each package's name, version, architecture, archive
  path, SHA-256 and size), unprivileged, once per key; the closure is a
  record in the store.
- **The rendered diff is what a change's reviewers read:** per target and
  per part, typed, then printed, with the targets it touches and the
  layers each must rebuild. Who posts it is open.
- **Errors are values,** each with its kind, its file, line and column in
  the revision, and its target and task. Render collects every
  independent error, not just the first.
- **Tested from day 0:** golden repositories of hearth's own, the merge
  laws and determinism as properties, every decoder fuzzed, the template
  engine held to MiniJinja, and fakes of git and mmdebstrap kept honest
  by conformance. os's goldens inspire cases; they are not an oracle.

## 2. Inputs

### 2.1 What render reads

- **A repository:** a git directory, of two kinds:
  - **a local one,** which `hearth render` and `hearth diff` name with
    `--repo`, or find around the current directory (hearth/docs/design/client/README.md,
    3.5): the client renders it in its own process, with
    its own user's access, and never hands the path to a daemon
    (hearth/docs/design/client/README.md, section 3);
  - **the builder's own clone,** `@hearth.store/repo/`, which the
    daemon's builder fetches and renders in jobs its steward gives it,
    and the only repository the daemon renders (hearth/docs/design/protocol/standalone.md;
    hearth/docs/design/domain/steward.md, 3.3).
- **A revision:** a commit id, or a ref name that render resolves to a
  commit id once, first. Everything after names the commit.
- **Targets:** one, several, or every target of the revision.
- **Nothing else.** No overlay, no workspace, no environment variable, no
  file outside the revision. Resolution alone reads more: the store's
  records and the archive (`spec.md`, section 2).

### 2.2 Git as a tool

```
git --git-dir=<repository> rev-parse --verify --quiet <revision>^{commit}
git --git-dir=<repository> ls-tree -r -l -z --full-tree <commit>
git --git-dir=<repository> cat-file --batch
git --git-dir=/dev/null diff --no-index --no-color --no-ext-diff --no-textconv --diff-algorithm=myers
    --unified=3 -- <store>/cache/<before> <store>/cache/<after>                  a changed file's hunks (6.1)
```

- **git is `/usr/bin/git`,** Debian's, spawned by its path: skein's
  `Spawn` searches no `PATH`. The builder's tools are named by fixed
  paths, as mmdebstrap's is (`spec.md`, 2.2); in M1 nothing configures them
  (hearth/docs/design/client/README.md, 3.5).
- **The environment is given whole:** `LC_ALL=C`, `GIT_CONFIG_NOSYSTEM=1`,
  `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_NO_REPLACE_OBJECTS=1`,
  `GIT_TERMINAL_PROMPT=0`, and `HOME=/dev/null`. No user's or
  system's configuration changes what git prints, and a replace ref
  cannot change a blob behind a commit id. Every file git looks for
  under `HOME` (`.gitconfig`, `.config/git/attributes`) lies beneath a
  file that is not a directory, so git finds none and says nothing, and
  there is no directory to make or keep empty.
- **The revision is checked before it reaches git,** as a guard, not a
  parse: 1 to 255 bytes of `[A-Za-z0-9._/-]`, never starting with `-`.
  git resolves what passes as it always does: a commit id of 40 or 64
  hex digits, an abbreviated or uppercase one, a ref name. `rev-parse`
  exiting 1 means the revision names no commit, whatever it printed
  (`a..b` prints two lines).
- **`ls-tree` gives the listing:** one record per entry, `<mode> <type>
  <oid> <size>\t<path>\0`, the size right-aligned in at least seven
  columns and `-` for a submodule, whose type is `commit`. Modes `100644`
  and `100755` are files (the
  second executable), `120000` a symbolic link whose blob is its target,
  `160000` a submodule, which is an error under a path render reads.
- **`cat-file --batch` gives blobs,** one process per `Read` (5.2):
  render writes `<oid>\n` for each of the `Read`'s blobs, closing git's
  input after the last, and reads, for each in turn, `<oid> <type>
  <size>\n`, the bytes, and `\n`. The header's size must equal the
  listing's and fit the blob's kind (2.3) before render demands the
  body. `<oid> missing\n` answers an object the repository lacks: a
  `tool` error of what needs that blob, and the batch reads on.
- **`git diff --no-index` gives hunks,** for the diff alone (4.1): it
  runs outside any repository, on two cache files. Wherever its working
  directory lies, in a work tree too, `--git-dir=/dev/null` names no
  repository, so git discovers none, and no work tree's configuration or
  attributes (`diff.interHunkContext`, `core.autocrlf`, a `-diff`) change
  what it prints. It exits 1 when the files differ, as they do when
  their digests do. Its output is unified: the headers before the first
  `@@`, which are skipped, then hunks; a file git takes for binary
  prints one `Binary files … differ` line and no hunk. Anything else
  fails that file's hunks: an exit of 0, an exit of 1 with neither a
  hunk nor that line (git's answer when it cannot read a file), an exit
  past 1 or a signal. The change stands without its hunks, and its
  target has a `tool` error, so the diff exits 75 (hearth/docs/design/client/README.md,
  3.3).
- **A failed git command is a `tool` error** with its exit and the tail
  of its standard error: the last 20 lines, each cut at 160 bytes with
  no marker, as mmdebstrap's (`spec.md`, 2.5). The oldest line leaves first, a line
  keeps its first bytes, and a last line with no line feed counts as the
  newest. Output git should not give is one too. git runs with no deadline in M1: it reads a local repository,
  which answers or fails, and the person may interrupt the client; a
  daemon's render is bounded by its job's deadline (hearth/docs/design/domain/steward.md, 5.2).
- **`hearth-protocol-git` builds the vectors and parses the outputs** into
  typed records; the domain sees a listing, decoded documents and
  templates, the digests of static files and a diff's hunks as lines,
  never git's text (README, `spec.md`, section 1.2).

### 2.3 Bounds

Render's limits are configured (`skein/docs/foundation/programming-model.md`, section 7); these
are the defaults. M1's client, the one builder of M1, runs on the
defaults alone (hearth/docs/design/client/README.md, 3.5).

| Limit | Default |
|---|---|
| entries in a revision's listing; the bytes of all its paths | 16,384; 4 MiB |
| a path | 4,096 bytes |
| a JSON document; its depth; keys in an object | 256 KiB; 32; 4,096 |
| a template; an expression, wherever it stands (a `task.json` string, a template's tag or `{{ }}`, an interpolation) | 256 KiB; 4 KiB |
| a static file, or a `file` setting's target: streamed to the cache, never held | 64 MiB |
| JSON documents, declarations and templates decoded from one revision, counted as they are held | 16 MiB |
| targets in a revision; tasks; settings in a task | 1,024; 1,024; 512 |
| nesting of statements in a template; of an expression | 32; 64 |
| entries one `task.json`'s declarations expand to for one target, an `if` that is false making none | 65,536 |
| loop iterations in one evaluation | 1,048,576 |
| values one evaluation makes; the exports of one environment | 16 MiB; 16 MiB |
| one target's composed settings; its validated settings; its expanded entries | 16 MiB each |
| the specs one command holds, with the sizes of the files they name and their resolutions' closures, a diff's two sides together | 16 MiB |
| a rendered file, held until it is in the cache; bytes rendered for one target | 16 MiB; 256 MiB |
| errors kept from one command, a diff's two sides together | 256, and 16 MiB of the bytes they carry; then a count |

- **A level of nesting** is, in an expression, each `(`, a list's or an
  index's `[`, a filter's `(`, a `not`, and an `if` or an `else` still
  open: each place hearth/docs/design/protocol/format/templates.md, 4's grammar nests. In a template it is
  an open `if` or `for`. The level past the limit is refused where it
  opens.
- **One evaluation** is one expression's, or one template's run for one
  target. The values it makes count a string's bytes, and 32 bytes for
  each element of a list or entry of a map; a list it makes nests at
  most 64 deep. Composed and validated settings, expanded entries,
  exports, specs and closures count alike, a record as a map of its
  fields.
- **What is decoded counts as it is held,** not by its file's bytes: a
  document's or a declaration's values as values count, a key's bytes as
  a string's, and a template the bytes of the text and names it holds,
  32 bytes for each instruction and operation, and 4 bytes for each of
  its lines, whose first bytes it keeps (5.3): every line, a comment's
  among them. The document, declaration or template whose count passes
  the 16 MiB fails the render at its path.
- **What a limit fails** is what holds the excess. The listing's limits,
  the 16 MiB of decoded documents and templates, and the 1,025th target
  or task fail the render: no target renders, and the error is the
  revision's, at the path at fault. A document, a template, an
  expression or a static file past its limit, or a task past its
  settings, fails what needs that file (3.1). A limit met while
  composing, validating, expanding, evaluating or rendering a target, or
  holding its spec, fails that target (in a diff, on that side). One met
  while reading the revision is `decode`; any other is `evaluation`
  (`errors.md`, 2).

### 2.4 The builder's clone and its fetch

- **The clone is bare, at `@hearth.store/repo/`,** a name of no digest
  shape that hearth/docs/design/domain/store/README.md's scan knows (hearth/docs/design/domain/store/README.md, section 2). It is made once,
  `git init --bare --quiet <store>/repo`, and read with 2.2's commands.
- **The fetch** is the one git command that reaches the network, as
  `apt-helper` is build's (hearth/docs/design/domain/build/README.md, 8.2). The builder runs it in a
  `fetch` job from its steward (hearth/docs/design/domain/steward.md, 3.3). Its argument vector,
  with the repository's URL and the branch the machine follows, which
  the job carries from the steward's desired revision (hearth/docs/design/protocol/standalone.md, 1):

  ```
  git --git-dir=<store>/repo fetch --quiet --no-tags --no-recurse-submodules --prune
      <url> +refs/heads/<branch>:refs/hearth/<branch>
  ```

  A desired commit must then be reachable from `refs/hearth/<branch>`;
  render resolves it there, or a commit imported from a bundle from its
  own `refs/hearth-dev/<commit>` (hearth/docs/design/protocol/standalone.md, 1; hearth/docs/design/domain/development.md, 6.2).
- **Its environment is 2.2's,** given whole, plus the credential's one
  variable. The URL never carries a credential.
- **The read credential is the builder's own,** never a secret of the
  secret store: a forge token or a deploy key kept in the store under
  `credentials/`, 0700, user `hearth`, imported at bootstrap
  (hearth/docs/design/domain/machine/bootstrap.md), and given to git by a fixed variable:
  - over HTTPS, `GIT_ASKPASS=/usr/local/lib/hearth/askpass`, a program the
    hearth task ships that prints `hearth` for a username and the token
    in `<store>/credentials/token` for a password;
  - over SSH, `GIT_SSH_COMMAND=ssh -F /dev/null -i <store>/credentials/deploy_key
    -o IdentitiesOnly=yes -o StrictHostKeyChecking=yes
    -o UserKnownHostsFile=<store>/credentials/known_hosts`.
- **Bounded:** the deadline the steward sets on the `fetch` job
  (hearth/docs/design/domain/steward.md, 5.2), and the store's free space; git's progress is not
  parsed, only its exit and the tail of its errors.

## 3. The pipeline

```
revision ──git──▶ 1 decode ──▶ 2 compose ──▶ 3 validate ──▶ 4 exports ──▶ 5 conditions
                                                                              │
                  spec ◀── 8 assemble ◀── 7 collisions ◀── 6 files ◀──────────┘
```

Step 1 and discovery run once per revision; the rest once per target,
and step 4 reads other targets of the same environment. A target's
settings are composed when it renders, and another target's task or
placement when step 4 reads it, and none is kept past its use (5.4).

### 3.1 Decode

- **The listing classifies every path** (hearth/docs/design/protocol/format/README.md, section 2): a
  configuration document (`fleet.json`, `envs/*.json`,
  `hosts/<base>/*.json`), a declaration (`tasks/*/task.json`), a template
  (`*.j2`), a static file under `tasks/*/tree/`, or a file outside the
  four roots, read only if a `file` setting names it. A path under the
  four roots that hearth/docs/design/protocol/format/README.md, section 2 does not allow is a `repository`
  error: in `envs/`, anything but an `<env>.json`; directly in
  `hosts/<base>/`, a `.json` that is neither of its documents (a file
  deeper there, or not `.json`, is another file); a file directly in
  `hosts/` or `tasks/`; a link where a document or a declaration must
  be; and any submodule. An absent `fleet.json` or `<base>.json` is an
  empty layer.
- **The domain asks for blobs by kind,** and the protocol layer decodes
  each before it goes up:
  - JSON through skein's tokenizer into `hearth-protocol-config`, which
    refuses duplicate keys, non-integer numbers and U+0000, drops
    `_note`, and gives typed values, each with its line and column, from
    the position the tokenizer gives each token (README, 5.1);
  - a declaration into its fixed shape (hearth/docs/design/protocol/format/tasks.md, 1), its `if` and
    `for_each` fields and its interpolated strings passed through
    `hearth-protocol-template`;
  - a template through `hearth-protocol-template` into a flat sequence of
    instructions (5.3);
  - a static file, or a file a `file` setting names, is never decoded
    and never held: the protocol layer streams its blob from `cat-file`
    into the store's cache by SHA-256 (hearth/docs/design/domain/store/README.md, section 2), hashing as
    the bytes pass with lib's SHA-256 (asked of skein: README, 5.1), and
    the domain receives its object id, size and digest.
- **A decode error fails what depends on the file,** not the revision:
  the targets that have a broken task, or that compose a broken document;
  only the limits of the whole revision fail it all (2.3). A repository
  error fails the targets of its task or base alike; one of neither (a
  stray file in `envs/`, or directly in `hosts/` or `tasks/`) fails no
  target, and is the revision's error all the same.
- **Errors of the revision print whatever targets are asked.** Step 1
  and discovery read every document and declaration, so their errors
  print with any command; an error of a target prints only when that
  target is asked, and a task's templates are read only when an asked
  target has the task (3.3). While a `hosts/` document is refused, a
  named target that discovery cannot find is left out under that error,
  which gives exit 2, not 64's unknown target (hearth/docs/design/client/README.md, 3.3).

### 3.2 Compose and merge

- **Discovery** makes the targets from the listing and the bases'
  `members` (hearth/docs/design/protocol/format/README.md, section 3): base, member, environment, name. Two
  targets of one name are an error.
- **Each target's layers** are merged in order (hearth/docs/design/protocol/format/README.md, 3.2) by
  RFC 7386 (hearth/docs/design/protocol/format/README.md, 4.1): task defaults, fleet, base, member,
  environment, base in the environment, its member.
- **Presence** comes from the base, member and base-env layers alone
  (hearth/docs/design/protocol/format/README.md, 3.3), and the `hearth` task is added to every target but
  a container (hearth/docs/design/protocol/format/tasks.md, 12); the composed settings are restricted to
  the present tasks.
- **Every value keeps its provenance:** the layer, file, line and column
  that set it, so a later error names the right file. An object's is the
  layer that made it (RFC 7386's `result = target`), not a later one that
  patched its members, and each member keeps its own.

### 3.3 Validate

- **Once per revision, each declaration is checked against itself:** its
  schema is well formed and its defaults have their types. **For each
  task an asked target has,** every
  `settings.<name>`, `tasks.<task>.<setting>` and `exports.<task>` that its
  templates and expressions name is declared (hearth/docs/design/protocol/format/README.md, 5.4); its
  templates are used and its scripts declared.
- **Per target, the composed settings are checked against the schemas:**
  undeclared settings and fields, types, enums, ranges, required
  settings, `target` and `file` references (hearth/docs/design/protocol/format/README.md, 5.5). Defaults are
  filled at every depth (hearth/docs/design/protocol/format/README.md, 5.3). `debian` and `placement` are
  checked against their fixed schemas.
- **Owners of what persists have fixed ids.** Once the declarations are
  concrete (3.5), every owner and group of a data path, a secret file or
  an identity file must be a declared user or group with a fixed `uid`
  or `gid`, package users included; a download's must be `root` or such
  a package user or group (hearth/docs/design/protocol/format/tasks.md, 6, and hearth/docs/design/protocol/format/README.md, 8.1). Anything else is
  refused: the packages layer is rebuilt from scratch, and an id a
  postinst took could move under data that keeps the old one.

### 3.4 Exports

- **For each task whose exports a task of this target reads,** render
  composes every target of the environment that has that task, validates
  that task's settings there, and evaluates its exports
  (hearth/docs/design/protocol/format/README.md, section 6). Each target's exports are computed once per
  revision and shared by all their readers.
- **Placements are read the same way:** `target.containers` holds the
  environment's targets placed on this one (hearth/docs/design/protocol/format/README.md, 3.5). A
  container's `on` must name a target of its environment that is not a
  container; a build environment has none (hearth/docs/design/protocol/format/README.md, 3.4). The
  containers placed on one host must have id ranges that overlap
  neither each other nor the builder's [M5] (hearth/docs/design/protocol/format/README.md, 3.5).
- **No cycle is possible:** exports depend only on their own task's
  settings and on `target` (hearth/docs/design/protocol/format/README.md, 6.1).
- **An error in another target's exports fails this render too,** naming
  both targets: a spec never reads a guess. A target's exports come from
  its exporting task's validated settings and `target` alone
  (hearth/docs/design/protocol/format/README.md, 6.1), so what else fails on it (another task, its own
  `requires`, its files) leaves them as they are, and its readers
  render. When they cannot be computed (a document the exporter composes
  is broken, the task's settings fail validation, an export's expression
  fails, or a refused `hosts/` document hides which targets have the
  task, 3.1), each reader fails with an `exported` error (`errors.md`, 2). A
  container's placement is read alike: each entry of `target.containers`
  holds the fields of the container's `target.placement`, and a container
  whose placement cannot be composed fails its host.
- **A container reads of its host its placement alone:** that `on` names
  a target of its environment that is not a container (hearth/docs/design/protocol/format/README.md, 3.4).
  Its context holds nothing else of the host (hearth/docs/design/protocol/format/templates.md, 7), and its
  data part none of the host's policies (`spec.md`, 1.3). So what else fails on the
  host (a task that does not exist, `hearth` among them when
  `tasks/hearth/` is absent; a declaration or a file of its tasks; its
  settings, its guards, its entries) leaves its containers as they are,
  and they render. A host whose placement cannot be composed fails each
  container placed on it:
  - while a document the host composes is refused, by its decode, the
    listing or a rule of composition (`README.md`, 3.1, `errors.md`, 2), the container is left
    out under that error, which prints whatever targets are asked, once
    and without a target (`errors.md`, 1);
  - a host whose documents are not refused but compose no placement
    gives each container asked a `composition` error at its `on` (`errors.md`, 2),
    beside the host's own when the host is asked too.

### 3.5 Conditions

- **`requires` first** (hearth/docs/design/protocol/format/tasks.md, 9): a false guard is an error with
  its message.
- **Then every declaration entry:** `for_each` expanded, `if` evaluated
  for each element, keys and string fields interpolated, typed fields
  evaluated (hearth/docs/design/protocol/format/tasks.md, 2). The result is each task's concrete
  declarations: packages, preseeds, modules, downloads, file entries,
  users, groups, presets, tmpfiles lines, data, secrets, identity, boot
  parameters, scripts.

### 3.6 Files

- **Every file of the tree is produced and digested:** a static file of
  `tree/` and a `copy` by the digest step 1 gave it; a template of
  `tree/` or `templates/` by evaluating it in the task's context
  (hearth/docs/design/protocol/format/templates.md, 7); a link by its target. `customize.sh` and
  `software.sh` fragments likewise.
- **A rendered file is written to the store's cache by digest** as soon
  as it is complete, and dropped from memory: a target holds one rendered
  file at a time.
- **Each output is bounded** (2.3), and an undefined value, a type error
  or an exceeded limit is an error at the template's line.
- **Render adds what tasks may not write themselves:** `/etc/subuid` and
  `/etc/subgid` from the users' subordinate ranges, a `tmpfiles` line of
  type `d` for each data entry's mode and owners (hearth/docs/design/protocol/format/tasks.md, 6 and
  7), and the initramfs configuration (`spec.md`, 1.2). `/etc/subuid` and
  `/etc/subgid` are one text, subuid(5)'s `<user>:<start>:<count>` line
  for each user with a range, by name: two files `0644 root:root` in
  the config part's tree, and neither when no user has a range.

### 3.7 Collisions

Render compares the concrete declarations of all the target's tasks:

| What | Rule |
|---|---|
| a path in the file tree | one task only, even with identical content |
| a data name, within the target and across classes; a data path | one task only |
| a secret's or identity's path | one task only, and never a path of the tree |
| a path in the tree under a data path | never: the mount would hide it |
| a download's path | one task only; never equal to, beneath or over another's, a path of the tree, or a secret's or identity's path; never under a data path |
| users, groups (a sysusers user's own group among them: hearth/docs/design/protocol/format/tasks.md, 6), presets, tmpfiles lines, preseeds, boot parameters | several tasks only identically |
| a path a `live` entry covers | several entries, of one task or several: their consumers are a union (hearth/docs/design/protocol/format/tasks.md, 14) |
| a uid or gid; a subordinate range | one user or group; ranges never overlap |
| packages, initramfs modules | a union |

- **Identically** is field for field, once defaults are filled: a
  default one task writes out and another leaves out is the same value.
  Positions and the tasks themselves are not compared.
- **Paths collide when one is the other or lies beneath it:** a tree
  file beneath a tar download's path, two data paths nested, a download
  under a data path, which the mount would hide. Two claims of one task
  collide as two tasks' do.

A collision is an error that names both tasks and both lines. os
overwrote silently, in alphabetical order of task.

### 3.8 Assemble

- **Each part's record** (`spec.md`, section 1) is built from the concrete
  declarations and the produced files, sorted as hearth/docs/design/protocol/format/README.md, 4.3 says,
  encoded canonically and digested.
- **Render also gives the resolution key** (`spec.md`, 2.1), and the layer keys when
  the store already holds that key's resolution (`spec.md`, 1.5). When it does not,
  the key is needed, and the layer keys are unknown until it is resolved
  (`spec.md`, 2.7).
- **The live part maps each covered path to its consumers,** from every
  task's `live` entries (hearth/docs/design/protocol/format/tasks.md, 14): the eligibility input that
  only a spec holds. Render decides nothing about a change: the builder
  pairs two specs' live parts with two outputs' tree files when it plans
  a trial, in a `plan live` job (hearth/docs/design/domain/slots/trial.md, 7; hearth/docs/design/domain/steward.md, 3.3).
- **The output is the spec.** Every file it names is in the store's
  cache by digest, static files from step 1 and rendered ones from
  step 6, where a build finds them. Specs are never stored: a builder
  renders again when it needs one, as a new builder does for a `build`
  job whose render an earlier one answered (hearth/docs/design/domain/steward.md, 3.3).

### 3.9 Checks and probes [M5]

- **A task's checks are evaluated as its other declarations** (step 5):
  their fields may read settings, and their rendered form, every value
  resolved, enters the checks part (hearth/docs/design/domain/operating.md, 2.2).
- **A probe with `from` is exported.** Render puts it into its task's
  exports, rendered, with the target it probes; in step 4 each target of
  the environment that has the task `from` names reads those probes and
  renders them into its own checks part (hearth/docs/design/domain/operating.md, 2.6). A change to
  one probe changes the probed target's and its probers' parts, and no
  other.
- **Refused at render:** a `blessing` check with `from`; a `from` that
  no target of the environment has; a command check whose `user` is
  `root`, or is neither `nobody`, a sysusers user nor a package user a
  task of the target declares (hearth/docs/design/protocol/format/tasks.md, 6; hearth/docs/design/domain/operating.md, 2.2;
  hearth/docs/design/domain/activator/operations.md).

### 3.10 Costs

- **A revision is decoded once** and shared by all its targets. Decoded
  documents, templates and static files' digests are kept by git object
  id, so the next revision, a diff's second side among them (5.4),
  decodes only the blobs that changed. Object ids name contents, so this
  memory changes no result.
- **A target's render** composes its layers, validates, evaluates and
  hashes its files: for the fleet's targets, milliseconds each.
  Rendering every target of two revisions, as a diff does (section 4), is
  cheap enough to do on every change.

### 3.11 What changes from os

- **Always in full.** os's `manifest` wrote a spec directory and reused
  its previous output as the resolution cache; hearth keeps only digests
  and records.
- **No overlay, no workspace, no `--resolved`:** every input is in the
  revision, and tests use fake mmdebstrap (6.6).
- **Collisions, undeclared settings and undefined values are errors**
  where os overwrote, ignored or printed nothing.
- **Every part has a digest;** os had a hash for packages alone, and
  deploy hashed the etc tree without modes or links.

## 4. The rendered diff

### 4.1 Typed

```
diff
  from, to             commit ids
  targets              map target → change
change
  added { spec digest } | removed | same
  | changed { parts: list of part change, rebuild }
  | failed { side: from | to | both, errors }
part change
  packages             request: added, removed; sources, variant, preseeds, initramfs, apt: before, after;
                       seeds: users and groups added, removed, changed, by name;
                       resolution: same | new key | new key, resolved: added, removed, versions changed
  software             downloads and artifacts added, removed, changed, by path; scripts changed
  config               tree entries added, removed, changed { path, before, after };
                       groups, users, members, presets, tmpfiles: added, removed, changed, by key;
                       scripts changed
  data                 entries added, removed, changed, by name
  secrets              names added, removed, changed; never a value
  boot                 before, after
  live                 paths added, removed, changed
  exports              task.name: before, after
  checks [M5]
  meta                 base, member, environment, placement: each before, after, when it changed
rebuild
  from packages { resolution: known | needed } | from software | from config | slot only | nothing
```

- **A changed file carries its two digests and sizes, and its hunks.**
  The domain asks for them (5.2) when both sides are under 1 MiB: git
  computes them from the two contents in the store's cache (2.2), with
  three lines of context, and `hearth-protocol-git` hands up at most 200
  lines a file, with the count of those left out, or that the file is
  binary. The lines count from the first `@@`, each `@@` line among them,
  as 4.3 prints them, and each is one of a hunk's header, context,
  removed, added, or `\ No newline at end of file`, kept and counted
  like the others. The domain holds one target's hunks at a time, as it
  hands its change up. Step code computes no text diff. The hunks are
  part of the typed diff, so its JSON carries them (hearth/docs/design/client/README.md, 3.2), and a client that
  cannot read the daemon's store [M3] receives them with the rest.
- **Secrets show names and places,** never values, since render has none.
- **`failed` is a target whose spec did not render** on a side that has
  it: `from` or `to`, or `both` when each side has it and neither
  rendered. Its errors are those sides' (`errors.md`, 1, `side`), and print after
  the changes. A failed target counts among the diff's targets, neither
  as changed nor as unchanged. A resolution that fails leaves the spec
  rendered, so its change stands (`spec.md`, 2.7).
- **Every part's change says what moved its digest:** a live-only change
  rebuilds `nothing` (`spec.md`, 1.4), as does a change to meta, and no changed
  target shows an empty list of part changes.

### 4.2 Targets touched and costs

- **A change touches the targets whose spec digest differs,** and those
  it adds or removes. A change to a task touches every target that has
  it; a change to an export, every reader in that environment; a change
  to `envs/lab.json`, no production target.
- **Rebuild** is the lowest layer whose key changed: from packages (and
  whether the new resolution is known), from software, from config. A
  change to data, secrets or boot alone is a new slot of the same output.
  A change to exports alone builds nothing on the exporting target.
- **Time,** when the builder has built the target before, is its last
  recorded cost for each layer to rebuild. Whether activation needs a
  reboot is known only after the build, from the kernel's and the
  initramfs's digests (hearth/docs/design/domain/slots/trial.md), and whether it can be live,
  from the two outputs' tree files and the live parts (hearth/docs/design/domain/slots/trial.md, 7).

### 4.3 Printed

A commit changes west-prod's public endpoint, in `hosts/west/west-prod.json`:

```
$ hearth diff main HEAD
carbon-prod                                         rebuild from config
  config   ~ /etc/systemd/network/wg0.netdev        0644 root:root
               @@ -9,5 +9,5 @@ ListenPort=51820
                [WireGuardPeer]
                PublicKey=AAAA…=
                AllowedIPs=10.0.0.1/32
               -Endpoint=203.0.113.157:51820
               +Endpoint=203.0.113.158:51820
                PersistentKeepalive=25
west-prod                                           nothing to build
  exports  ~ vpn.endpoint   "203.0.113.157" → "203.0.113.158"

2 of 11 targets change; 9 are unchanged.
```

- The client prints it as text, or as JSON for agents
  (hearth/docs/design/client/README.md, section 3). The typed diff is the same either way.
- The coupling is visible: carbon-prod changes because it reads west-prod's
  exports, and carbon-lab does not, because exports stay in their
  environment.
- **Hunks print as git gives them,** three lines of context and each
  `@@` line with git's own context after it (hearth/docs/design/protocol/format/README.md, 6.6's template
  rendered for carbon-prod). A failed target prints `failed (from)`,
  `failed (to)` or `failed (both)` where the rebuild goes, and its errors
  print after the changes.

### 4.4 Who posts it

- **Open** (README, section 6): the lab, CI or temper's gate could post it
  at a change's head, as a status is; and how a change that touches many
  targets is shown, folded by identical part changes or not, follows from
  who reads it.

## 5. Layers and crates

### 5.1 The crates

```
hearth-domain-builder            the builder's root: routes between its children
├── hearth-domain-render         configuration to spec; resolution's decisions; the diff        M1
└── hearth-domain-store          M1: resolutions and cache files' records; M2: the rest         M1, M2

hearth-protocol-builder          the builder's protocol layer: routes its machines, translates to render's types
├── hearth-protocol-git          git's argument vectors; ls-tree, cat-file and diff records       M1
├── hearth-protocol-tree         M1: the frame, 3.1's decoder, records of kinds 1, 4 and 5        M1, M2
├── hearth-protocol-config       strict JSON to typed documents and declarations, with positions  M1
├── hearth-protocol-template     template and expression text to instruction sequences            M1
└── hearth-protocol-mmdebstrap   simulate's argument vector; Inst lines; Packages stanzas; tails  M1
```

- **`hearth-domain-render`** holds, as modules: the revision's decoded
  documents; composition and RFC 7386; validation; evaluation; exports;
  collisions; parts and digests; resolution's keys, records and closures;
  the diff. It depends on lib and nothing else.
- **The machines depend on lib only** (`skein/docs/foundation/programming-model.md`, 4), but
  for `hearth-protocol-config`, which composes skein's JSON tokenizer and
  depends on `skein-json` too, as skein's `skein-llm` does (
  `skein/docs/design/llm.md`). `hearth-protocol-config` cannot call the template machine,
  so it hands up a declaration's expression strings as text with their
  positions, and `hearth-protocol-builder` passes each through
  `hearth-protocol-template` before translating the whole declaration
  into render's types.
- **The client runs the same domain in-process** when no daemon
  answers, or for a local repository (README, `spec.md`, 1.1; hearth/docs/design/client/README.md, 3.5); in
  the daemon, the builder's root takes the steward's jobs through
  `hearth-protocol-jobs` (hearth/docs/design/domain/steward.md, section 3).
- **The client prints with `hearth-client-protocol-records`'s
  printer** (hearth/docs/design/client/README.md, 3.2), a machine that writes the spec,
  the diff and the errors as text and as JSON lines: M1 brings forward that crate's records of render
  and diff and their printer, and M3 adds the rest. Which crate of the
  client routes the builder's answers to it is the plan's, within
  hearth/docs/design/README.md, 4.2's roles.

### 5.2 What crosses

```
render → protocol                                      protocol → render
List { repository, revision }                          Listed { commit, entries: path, mode, oid, size }
Read { commit, blobs: (oid, as) }                      Read { oid, a document | a declaration | a template | a link's target }
  as: document | declaration | template | link | cached  Cached { oid, size, sha256 }: streamed into the cache
Keep { sha256, bytes }: a rendered file                Kept { sha256 }
Resolve { key, sources, request, variant, apt }        Resolved { key, closure, indices, versions, cost }
Hunks { before: sha256, after: sha256 }                Hunks { lines: (kind, text), left out, binary }: kind is header, context, removed, added or no newline (6.1)
every request                                          Failed { request, class, at, tail }
```

- **Resolution records** go through the builder's root to the store and
  back (hearth/docs/design/domain/store/records.md): render asks for a key and records a new
  closure; it never touches a file. In M1 the store's domain is
  `hearth-domain-store`'s first part: resolutions and cache files' records
  alone, over `hearth-protocol-tree`'s frame and its kinds 1, 4 and 5
  (hearth/docs/design/domain/store/README.md, 2.4, and hearth/docs/design/domain/store/records.md, 1); M2 grows both.
- **Every request gets one terminal event** (`skein/docs/foundation/programming-model.md`,
  5.2), and a resolution's deadline is render's to set; git's commands
  have none (2.2).

### 5.3 Evaluation without recursion

- **Step code has no recursion and no unbounded loop**
  (`skein/docs/foundation/programming-model.md`, section 3), so nothing is evaluated as a tree:
  - `hearth-protocol-template` gives an expression as a flat sequence of
    operations in postfix order, with forward jumps for `and`, `or` and
    `… if … else …`; the domain evaluates it on a value stack whose depth
    the expression limit bounds;
  - a template is a flat sequence of instructions (emit text, emit an
    expression, branch, enter and advance a loop, assign), evaluated by a
    `for` over a range bounded by the iteration limit, with a
    `lib::Stack` of loop frames bounded by the nesting limit. Beside its
    instructions, it keeps the offset of each line's first byte, which
    places a fault at its own line and column (`errors.md`, 1) without reading the
    blob again (3.10), and which its count includes (2.3);
  - merging walks two values with an explicit stack bounded by JSON's
    depth.

### 5.4 Limits and what a render holds

- **Render's limits are 2.3's.** What it holds at once at those limits
  is the revision's listing (16,384 entries, 4 MiB of paths), with
  an object id, size and digest per static file; its decoded documents,
  declarations and templates (16 MiB, counted as they are held); the
  tasks each target has, by index; the exports of one environment
  (16 MiB); one target's composed settings, validated settings and
  expanded entries (16 MiB each); one evaluation's values (16 MiB); one
  rendered file (16 MiB); the command's specs, with their files' sizes
  and their closures (16 MiB); and the errors it keeps (256, and 16 MiB
  of the bytes they carry).
- **It holds no other target's settings** (section 3). Of a resolution
  record, a render holds the closure's digest, which its layer keys take
  (`spec.md`, 1.5), and a diff the closures it compares (`spec.md`, 2.7, `README.md`, 4.1). A diff renders
  its sides in turn, and keeps of the first only its specs, closures and
  errors, and, until the second's listing, its decoded blobs (below).
- **A diff's second side decodes only the blobs that changed** (3.10).
  When its listing arrives, it keeps of the first side's decoded
  documents, declarations, templates and links' targets, and of its
  static files' digests, those of the blobs that listing names, by object
  id and as what they were read; the rest go. It reads only the others,
  a blob the first refused among them. It gives what a render of its
  revision alone gives, within the same 16 MiB: each document,
  declaration and template counts in its own 16 MiB (2.3) in that
  render's order, kept or read; when what it holds would pass the 16 MiB,
  the kept blobs not yet counted go first, and one it then needs is read
  again. So a diff's peak holds one revision's decoded data, as a
  render's does.
- **The heap is measured, under ceilings**
  (`skein/docs/foundation/programming-model.md`, 6.3). The counting
  allocator holds each fixed scenario's peak under a ceiling named beside
  its test at every iteration (`skein/docs/foundation/testing-strategy.md`,
  section 6). No formula sums a layer or the builder, and nothing checks a
  sum against a machine.
- **A formula sizes an edge:** a decoder's retained bound, a decoded
  record, one template run's workspace. Each is named for what it bounds,
  gives `None` on overflow, and has a test of its value. The builder's
  `valid(limits)`, computed from them, is startup's check.
- **The defaults fit a modest machine, by measurement.** The large render,
  a revision of 4,096 entries, 512 of them documents, declarations and
  templates, with 64 targets, runs through the builder at the shipped
  limits, and its peak stays under a ceiling of its own
  (hearth/docs/design/testing/worlds.md, 4). This is the claim about a
  small VM, and it has no formula.
- **The protocol's machines take 2.3's limits,** the listing's, a
  document's and a template's among them, and the store's bounds
  (hearth/docs/design/domain/store/records.md, 1); the client's printer
  (hearth/docs/design/client/README.md, 3.2) prints a record of at most
  16 MiB, its line endings included, and refuses a longer one, which
  fails the command. Static files and rendered files live in the store's
  cache, never in the domain, so a small VM's builder renders what a
  large one does.
- **Between the builder's layers nothing is capped.** The queues between
  its own children grow as they fill, and only the edges refuse
  (`skein/docs/foundation/programming-model.md`, 6.3). `Limits::queue`
  sizes the two queues the builder shares with the kernel, `completions`
  and `submissions`, and nothing else.
- **A watermark holds render behind a client that has stopped reading**
  (`skein/docs/foundation/programming-model.md`, 7.1). `Limits::watermark`
  is one number, 30 as shipped, which keeps pacing close to what capped
  queues gave. The backlog is what `told`, `file_told`, `answers` and
  `asked` hold. At the watermark the domain's next resume waits, so render
  makes no more records: however many targets a call has, about a
  watermark of them wait behind a stuck client. What finishes admitted work
  is always taken: an event enters the protocol layer, an answer goes down
  once the client's printer takes it, and a request reaches io, at any
  backlog. `calls` stays outside the backlog: its events wait behind the
  domain's own next resume, so counting them would hold the watermark shut.
  The watermark paces and bounds no memory. A backlog may pass it by what
  one admitted event makes, and the heap stays measured.
- **A pass is paced.** A pass takes from each queue what the queue held
  when the pass reached it, and resumes a layer at most
  `Limits::watermark` times, so one pass never runs a whole render: what
  is left stays ready for the next pass. Io's stage paces itself
  (`skein/crates/skein-io/src/stage.rs`).

### 5.5 What keeps its order

No order here rests on a queue's room
(`skein/docs/foundation/programming-model.md`, 7.2). A machine's state
or a named predicate keeps each one, and a test holds it.

- **One read at a time.** A git machine, mmdebstrap's pipe reader and
  list reader, and the JSON decoder have at most one demand out, and make
  the next once the last is answered.
- **A blob's piece is taken before the next is read.** `cat-file`'s
  machine stops after each piece until it is told `Next`. A body waits in
  git's pipe, never in the builder, so a static file of 64 MiB is held a
  chunk at a time, however slowly the cache writes.
- **An outcome answers a request.** A stream's end or fault that comes
  before a request, or while a piece is still being taken, overtakes
  nothing: the machine holds it and gives it to the next request.
- **A pipe's reader gives one thing a delivery.** An `Inst`, a `Fetched`
  or the pipe's end, and the next demand. Resolution takes one of each
  from a delivery, and one demand from a list reader's; a second would
  be lost. The pipe's end comes last, and once.
- **A list's end follows its last stanza.** A scan gives at most one
  stanza. A list with no blank line after its last stanza gives that
  stanza and then its end, and the end comes once.
- **A decoder gives one thing a delivery.** The JSON decoder gives the
  next demand or a document's outcome, and a read takes one of each from a
  delivery and drops the decoder at its outcome, so that anything beside
  it would be lost. The outcome comes once, with no read out; `Closed`
  comes last, and once.
- **Resumed work goes before the next event.** While the protocol layer
  has work to resume, no io or file event enters it (`work_pending`).
- **One write at a time.** The client's printer has at most one demand
  out, and makes the next once the last is answered. A `Room` gives one
  piece, within the room it answers, and then the next demand or nothing.
- **A record is answered once.** The printer gives one event a record:
  `Printed` once its last piece has gone down, `Refused` with no bytes,
  or `Failed`. `Client` takes it before the next record.
- **One record at a time.** The client's printer takes the next record
  once the last has gone out, on standard output and standard error alike
  (`takes`), so answers print in the order render gave them.
- **An answer follows its resources' close.** Render hears from a git
  child once the process and its pipes are closed, so its next request
  finds none open.
- **A build's report comes whole.** Its layers print first, then the step
  it failed on, if it failed, and its reply last: the builder's root holds
  render's reply until the report is out (`holds`, `reporting`).
- **The domain takes a store step's answer before the store's next
  step,** so nothing waits in the queue above a step of the store. A step
  has answered when it added to what the protocol layer gave above,
  whatever that queue already held (`store_answered`), and only then does
  the next index open.
- **Render waits at the watermark, and what it asked for does not.** The
  domain's next resume waits while the backlog is at the watermark
  (`admits`, 5.4). Nothing else waits on that number.

## 6. Its world

### 6.1 Worlds

- **Render's domain world** runs `hearth-domain-render` under a scripted
  parent that plays the protocol layer: it delivers typed documents,
  declarations and templates as the machines would, in an order and at a
  pace drawn from the seed, and plays the store's records. It checks
  every world's invariants (`skein/docs/foundation/testing-strategy.md`, section 6).
- **Machine worlds** run each protocol machine against recordings of the
  real tool, cut at random: `ls-tree` and `cat-file` streams, `Inst`
  lines, `Packages` indices, failure tails (`skein/docs/foundation/testing-strategy.md`, 2.4).
- **The builder's system world** runs render with the fakes of git and
  mmdebstrap on the fake machine (hearth/docs/design/testing/fake-machine.md and hearth/docs/design/testing/tool-fakes.md).

### 6.2 Golden repositories

- **hearth's own:** `tests/render/<name>/`, each a repository's files and
  the expected printed spec of each target (`expected/<target>`), or the
  expected errors. The fake git serves a directory; one test in the real
  tier builds a real repository from it and renders through real git.
- **The repositories:** `fleet` (a miniature of the fleet: x1-prod,
  east-prod, west-prod, carbon-prod, carbon-lab, chat-prod), `merge`
  (presence, defaults, removal, maps, lists), `templates` (each
  statement, filter and whitespace rule), `exports` (the VPN, the mail
  pair, two environments), `cluster` (stoa's members), `errors` (one case
  per kind of `errors.md`, 2 that a repository causes; `tool` and `resolution` are
  builder-world cases, with git or mmdebstrap's fake failing, 6.6),
  `resolution` (a target whose resolution the store holds, from a
  recorded closure, printed with its `keys` line).
- **The printed spec** is the client's stable text form:

  ```
  target       x1-prod   base x1   environment prod
  placement    metal efi 7th-gen
  spec         9a0c5e1f2b3d
  packages     part 4be1c0d2a7f1   resolution 0c7d9e41c0d2 needed
    sources    forky amd64 main contrib non-free-firmware 20261001T030405Z
    request    btrfs-progs ca-certificates dbus efibootmgr firmware-intel-sound …
    variant    apt
    preseed    locales/default_environment_locale select en_US.UTF-8
    initramfs  btrfs nvme
  config       part e7c2a0d95b13
    /etc/hostname            file 0644 root:root 3f1c2ab07e44
    /etc/localtime           link root:root /usr/share/zoneinfo/Etc/UTC
    tmpfiles   /var/log/journal d 2755 root systemd-journal
  data         part 51aa0c7d9e41
    journal    /var/log/journal
  ```

  Digests and keys are cut to 12 hex digits, as every table of the
  client's is (hearth/docs/design/client/README.md, 3.2); its JSON gives them whole. A link
  prints its target, and an empty file the digest of no bytes
  (`e3b0c44298fc`), as any file prints its own. With the
  resolution in the store, its line reads `resolution 0c7d9e41c0d2
  known`, and a last line gives the layer keys: `keys  packages …
  software … config …`.

- **`HEARTH_BLESS=1` rewrites the expected files and still fails,** so a
  blessed change is always reviewed (os's practice).

### 6.3 Properties

Over generated repositories, valid by construction and then mutated:

- **Determinism:** one revision renders to the same digests in every
  world, whatever the seed's order of blob arrival and chunking: render's
  world over seeds, the protocol world's pipes cut at every byte, the
  simulated client, and real git, each compared by the whole digests
  the JSON form prints.
- **Representation:** reordering a JSON object's members, reformatting
  whitespace or editing a `_note` changes no digest.
- **RFC 7386's laws:** `merge(t, {}) = t`; `merge(merge(t, p), p) =
  merge(t, p)`; a non-object patch replaces; a key the patch lacks keeps
  its value; no result holds `null`.
- **Locality:** changing a setting in one target's own document that no
  export reads changes that target's spec alone; one that an export
  reads changes the exporting target and the targets of its environment
  that read it, and no other; changing a production file never changes
  a lab spec.
- **Order:** the order in which render takes tasks and targets changes
  nothing; a collision names the same pair, the task first in name order
  at `at` and the other at `also` (`errors.md`, 1), whichever task arrives first.
- **Digests:** two different records of a part give different digests
  over generated pairs; the encoding's own laws are hearth/docs/design/domain/store/README.md's
  (`../store/README.md`, section 9).

### 6.4 Fuzzed decoders

- **Every decoder is fuzzed** on its own (`skein/docs/foundation/testing-strategy.md`, 2.4):
  configuration documents and declarations; templates and expressions;
  `ls-tree` and `cat-file` streams; `Inst` lines and `Packages` stanzas;
  and what render hands the store and its tools, store names and
  argument vectors, swept in the builder's world.
  Corpora start from the golden repositories and the recordings.

### 6.5 MiniJinja as an oracle

- **Test code is ordinary Rust,** so it may run MiniJinja 2 with
  `trim_blocks`, `lstrip_blocks` and `keep_trailing_newline` on, strict
  undefined, and hearth's own filters registered.
- **Generated templates of the subset** render byte for byte the same in
  both. Where hearth differs on purpose (undefined, `first` on an empty
  list), the generator avoids it and the difference is listed in the
  test. hearth keeps hearth/docs/design/protocol/format/templates.md 4's precedence, which binds `~` below
  `+ -` and `is` below the comparisons, where MiniJinja binds `~` above
  them and `is` with the filters: both are listed, and the generator
  parenthesizes a `~` beside `+ -` and an operand of `is` that is not a
  lookup chain.

### 6.6 Resolution's fakes and conformance

- **`hearth-fake-mmdebstrap`** resolves against the fake machine's
  package universe and writes `Inst` lines and `Packages` indices of the
  real shape (hearth/docs/design/testing/tool-fakes.md).
- **Conformance** runs real `mmdebstrap --simulate --mode=chrootless`
  against the test tier's local mirror, and the fake against the same
  universe, and compares closures (hearth/docs/design/testing/README.md, section 3). Its runs are
  recorded, and the recordings replace os's hand-written logs.

### 6.7 os's goldens

- **Inspiration, not an oracle.** os's fixtures (conditions, fan-out,
  accumulation, secrets, components, forky's sources, the full host and
  container) are rewritten in hearth's format where the case still
  exists. Their expected output is hearth's, reviewed; it is never
  compared with os's, which strips final newlines and records no modes.

### 6.8 Budgets

- **Rendering every target of the miniature fleet, and a diff of two
  revisions,** are domain-world scenarios. Their budgets are counts, the
  requests a run makes, asserted against 3.10's costs; their time is an aim,
  1 s (0.5 s before the user's doubling of 10-22, provisional) a scenario,
  reported and never failed on (hearth/docs/design/testing/README.md, 4.3). The suites' budgets
  are hearth/docs/design/testing/README.md's (`../../testing/README.md`, section 4).

## 7. Open questions

- **A decompressor for the indices:** after a chrootless simulate run,
  `--skip=cleanup/apt/lists` leaves `InRelease` and the decompressed
  `Packages` files in the scratch directory, as the recordings show
  (6.6), and M1 refuses a compressed one (`spec.md`, 2.3). One is needed only if an
  apt keeps them compressed (`Acquire::GzipIndexes`); it depends on
  such an apt.
- **The `Inst` line with several origins** (`(2.41-12 Debian:13.1/stable,
  Debian-Security:13/stable-security [amd64])`), to verify against
  recordings.
- **Chrootless or unshare for resolution:** os and the bash fleet
  resolved chrootless and built unshare; a reason for preferring unshare
  was found once, in experiments, and not recorded. A conformance
  scenario compares the closures of both modes (hearth/docs/design/testing/README.md, 3.2), and
  hearth/docs/design/domain/build/packages.md, 7 refuses any build whose installed set differs from its
  closure. If the modes ever disagree, resolution moves to unshare, and
  render then needs the builder's subordinate range. A lead, from
  mmdebstrap 1.5.7's code: chrootless installs the essential set with
  `dpkg --force-depends` from outside and the rest with an outside
  `apt-get install`, while unshare and root skip that last apt step for
  `--variant=custom`, so the two may diverge for containers' variant.
- **apt's version and solver:** recorded beside the closure from M2
  (`spec.md`, 2.4), not keyed.
  Whether a new apt should re-resolve depends on how often closures
  differ in practice, which the costs and conformance will show.
- **A foreign architecture** with chrootless simulate, to verify; it
  depends on the first non-amd64 target.
- **Who posts the rendered diff,** and how a change that touches many
  targets is shown (README, section 6).
- **The defaults of 2.3 and the printer's bounds on hunks,** to set from
  the first measurements of the fleet.
- **Classifying mmdebstrap's failures:** `spec.md`, 2.5's table was checked against
  runs of mmdebstrap 1.5.7 and apt 3.0.3 for a package apt does not
  know, one with no installation candidate, dependencies apt cannot
  satisfy, a snapshot timestamp before the archive and an unreachable
  proxy. Recordings of a hash mismatch, a full disk and a 5xx answer
  will confirm or correct it.
- **Keeping decoded documents across revisions** within one builder's
  life, a pass and its linger (hearth/docs/design/domain/steward.md, 4.1): what a pass's later
  renders, a diff's two revisions or the bless's render, would save,
  against the builder's memory, bounded by a limit to set.
- **Fetching a single commit** rather than a branch, which needs the
  forge to allow fetching an object by id; it depends on the forges the
  fleet uses.
- **The bounds outside the builder:** `hearth_domain_tree::limits::walk_bound`
  and `hearth_protocol_accounts::decoder_bound` size edges of the tree
  helper and the activator, which the builder's limits don't hold, so its
  `valid` (5.4) reads neither. Whether each of those processes checks them
  at its own startup is to settle with their limits.
- **Where a resolution's closure is made:** 5.2's `Resolved` carries the
  closure, while `spec.md`, 2.3 has the domain match `Inst` triples to `Packages`
  stanzas; one of the two is to change.
- **The domain's next event and its own resumed work:** 5.5 orders the
  protocol layer's alone, and the domain's step takes event after event
  without asking whether it has work to resume, and `calls` stands outside
  the watermark's backlog (5.4). Whether it must ask, and whether a stream's
  events need a watermark of their own, depends on the first build step that
  streams its events.
- **What the service orders with no document asking** (5.5): `file_told` drains
  before `told`, by the order of the loops, and a search that swapped them failed
  no test; `asked` leaves in one order, a request that can't be taken holding those
  behind it (`io_accepts_request`). Neither is stated as an order. Whether either
  is one. Default: as they stand, until a test shows one.
- **The store adapter's and a keep's lifecycles** (8): `store.rs` and `keep.rs`
  in `hearth-protocol-builder` keep theirs in flags and `Option`s
  (`skein/docs/foundation/programming-model.md`, 5.4). Default: as they stand,
  until a debt item names them.
- **A stop with its output gone** (5.5): if standard error breaks and closes
  during a lock wait and the store then answers `Unopened`, `progress.stop()`
  queues a `Down::Finish`, and `resume` expects an output there is none of. This
  is from reading the code; no test has run it. Whether `stop` asks first.
  Default: as it stands, until a test shows it.
- **What the printer holds that the builder never reaches** (5.5): `Client`
  closes its output through io, so only
  `close_withdraws_the_demand_and_answers_closed` sends `printer::Request::Close`;
  and `Client::up` takes `Up::Failed` first, so the printer's `Below::Failed` runs
  only under its own tests, and `Client::resume`'s `Event::Failed` arm under none.
  Whether they stay. Default: as they stand, until a debt item names them.
- **What `takes` asks beside its stream** (5.5): `requests.is_empty()` and
  `events.is_empty()` repeat, by reading, what `Client`'s `Stream` says: it leaves
  `Printing` only as `resume` takes a record's one event, once `requests` is
  empty, and `fail` empties both. Whether they go. Default: as they stand, until
  a debt item names them.
