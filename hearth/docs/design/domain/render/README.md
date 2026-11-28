# Render

Provisional, 2026-10-09, revised 2026-11-14. How hearth turns a revision of an IaC repository and a target into a spec, and two revisions into a rendered diff: what render reads, its pipeline, the diff, its crates and its tests. The format it reads is hearth/docs/design/protocol/format/README.md; the spec and package resolution are `spec.md`, the errors `errors.md`. Milestone M1 builds it, with the client's `render` and `diff` (hearth/docs/design/client/README.md, section 3).

See [`diff.md`](diff.md) for how the rendered diff is made and printed, and
[`crates.md`](crates.md) for the layers and their crates.

## 1. In one page

- **Render is a pure function of a revision and a target,** exports included. It reads nothing but the revision: no secrets, no machine, no clock, no network. Any hearth renders any target, in full, in milliseconds, so nothing about a spec is remembered but its digests.
- **The revision is read through git,** a tool behind a protocol machine: one listing, then the blobs render needs, every size bounded before a byte is read (2.2).
- **Eight steps make four results:** the revision's inputs, a target's settings, its admitted entries and its spec (section 3). The protocol layer decodes JSON and templates into typed values and instruction sequences; the domain never parses.
- **A spec is parts with digests** (`spec.md`, section 1). Resolution, the one slow step, is remembered as a record of the store (`spec.md`, section 2).
- **The rendered diff is what a change's reviewers read:** per target and per part, typed, then printed, with the layers each target must rebuild (`diff.md`, section 1).
- **Errors are values,** each with its kind, its place in the revision, its target and task; render collects every independent error (`errors.md`).
- **Tested from day 0:** golden repositories, the merge laws and determinism as properties, every decoder fuzzed, the template engine held to MiniJinja, and fakes of git and mmdebstrap kept honest by conformance (section 6).

## 2. Inputs

### 2.1 What render reads

- **A repository,** a git directory: a local one, which `hearth render` and `hearth diff` name with `--repo` or find around the current directory and render in the client's own process with its user's access, never handing the path to a daemon (hearth/docs/design/client/README.md, 3.5); or the builder's own clone, `@hearth.store/repo/`, the only repository the daemon renders (2.4; hearth/docs/design/domain/steward.md, 3.3).
- **A revision:** a commit id, or a ref name resolved to one once, first; everything after names the commit.
- **Targets:** one, several, or every target of the revision.
- **Nothing else:** no overlay, workspace, environment variable or file outside the revision. Resolution alone reads more: the store's records and the archive (`spec.md`, section 2).

### 2.2 Git as a tool

```
git --git-dir=<repository> rev-parse --verify --quiet <revision>^{commit}
git --git-dir=<repository> ls-tree -r -l -z --full-tree <commit>
git --git-dir=<repository> cat-file --batch
git --git-dir=/dev/null diff --no-index --no-color --no-ext-diff --no-textconv --diff-algorithm=myers
    --unified=3 -- <store>/cache/<before> <store>/cache/<after>                  a changed file's hunks (4.1)
```

- **git is `/usr/bin/git`,** spawned by its path: skein's `Spawn` searches no `PATH`, and in M1 nothing configures the builder's tools (hearth/docs/design/client/README.md, 3.5).
- **The environment is given whole:** `LC_ALL=C`, `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_NO_REPLACE_OBJECTS=1`, `GIT_TERMINAL_PROMPT=0` and `HOME=/dev/null`. No user's or system's configuration changes what git prints, a replace ref can't change a blob behind a commit id, and every file git looks for under `HOME` lies beneath a file that is not a directory, so git finds none.
- **The revision is guarded before it reaches git,** not parsed: 1 to 255 bytes of `[A-Za-z0-9._/-]`, never starting with `-`. git resolves what passes as it always does; `rev-parse` exiting 1 means the revision names no commit, whatever it printed.
- **`ls-tree` gives the listing:** one record per entry, `<mode> <type> <oid> <size>\t<path>\0`, the size right-aligned in at least seven columns and `-` for a submodule, whose type is `commit`. Modes `100644` and `100755` are files (the second executable), `120000` a symbolic link whose blob is its target, `160000` a submodule, an error under a path render reads.
- **`cat-file --batch` gives blobs,** one process per `Read` (`crates.md`, 1.2): render writes `<oid>\n` for each blob, closing git's input after the last, and reads for each `<oid> <type> <size>\n`, the bytes and `\n`. The header's size must equal the listing's and fit the blob's kind (2.3) before render demands the body. `<oid> missing\n` is a `tool` error of what needs that blob, and the batch reads on.
- **`git diff --no-index` gives hunks,** for the diff alone, outside any repository: `--git-dir=/dev/null` names none, so no work tree's configuration or attributes change what it prints. It exits 1 when the files differ; its output is headers, skipped, then hunks, or one `Binary files … differ` line and no hunk. Anything else fails that file's hunks (an exit of 0, an exit of 1 with neither a hunk nor that line, an exit past 1, a signal): the change stands without them, and its target has a `tool` error, so the diff exits 75 (hearth/docs/design/client/README.md, 3.3).
- **A failed git command is a `tool` error** with its exit and the tail of its standard error: the last 20 lines, each cut at 160 bytes with no marker, the oldest line leaving first, a line keeping its first bytes, and a last line with no line feed counting as the newest. Output git should not give is one too. git runs with no deadline: it reads a local repository, the person may interrupt the client, and a daemon's render is bounded by its job's deadline (hearth/docs/design/domain/steward.md, 5.2).
- **`hearth-protocol-git` builds the vectors and parses the outputs:** the domain sees a listing, decoded documents and templates, static files' digests and a diff's hunks as lines, never git's text.

### 2.3 Bounds

Render's limits are configured (`skein/docs/foundation/programming-model.md`, section 7); these are the defaults, and M1's client runs on them alone (hearth/docs/design/client/README.md, 3.5).

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

- **A level of nesting** is, in an expression, each place hearth/docs/design/protocol/format/templates.md, 4's grammar nests: a `(`, a list's or an index's `[`, a filter's `(`, a `not`, an `if` or `else` still open; in a template, an open `if` or `for`. The level past the limit is refused where it opens.
- **One evaluation** is one expression's, or one template's run for one target. Its values count a string's bytes and 32 bytes for each element of a list or entry of a map; a list it makes nests at most 64 deep. Composed and validated settings, expanded entries, exports, specs and closures count alike, a record as a map of its fields.
- **What is decoded counts as it is held:** a document's or declaration's values as values, a key as a string, and a template the bytes of its text and names, 32 bytes for each instruction and operation, and 4 for each of its lines (`crates.md`, 1.3), a comment's among them. The document, declaration or template whose count passes the 16 MiB fails the render at its path.
- **A limit fails what holds the excess.** The listing's limits, the decoded 16 MiB and the 1,025th target or task fail the render, as the revision's error at the path at fault. A document, template, expression or static file past its limit, or a task past its settings, fails what needs that file (3.1). A limit met while composing, validating, expanding, evaluating or rendering a target, or holding its spec, fails that target, on its side in a diff. One met reading the revision is `decode`, any other `evaluation` (`errors.md`, 2).

### 2.4 The builder's clone and its fetch

- **The clone is bare, at `@hearth.store/repo/`,** a name of no digest shape that the store's scan knows (hearth/docs/design/domain/store/README.md, section 2), made once with `git init --bare --quiet <store>/repo` and read with 2.2's commands.
- **The fetch** is the one git command that reaches the network, run by the builder in a `fetch` job from its steward (hearth/docs/design/domain/steward.md, 3.3), with the repository's URL and the branch the machine follows from the steward's desired revision (hearth/docs/design/protocol/standalone.md, 1):

  ```
  git --git-dir=<store>/repo fetch --quiet --no-tags --no-recurse-submodules --prune
      <url> +refs/heads/<branch>:refs/hearth/<branch>
  ```

  A desired commit must be reachable from `refs/hearth/<branch>`, or, imported from a bundle, from `refs/hearth-dev/<commit>` (hearth/docs/design/domain/development.md, 6.2).
- **Its environment is 2.2's,** plus the credential's one variable; the URL never carries a credential.
- **The read credential is the builder's own,** never a secret of the secret store: a forge token or a deploy key under the store's `credentials/`, 0700, user `hearth`, imported at bootstrap (hearth/docs/design/domain/machine/bootstrap.md):
  - over HTTPS, `GIT_ASKPASS=/usr/local/lib/hearth/askpass`, a program the hearth task ships that prints `hearth` for a username and the token in `<store>/credentials/token` for a password;
  - over SSH, `GIT_SSH_COMMAND=ssh -F /dev/null -i <store>/credentials/deploy_key -o IdentitiesOnly=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile=<store>/credentials/known_hosts`.
- **Bounded** by the `fetch` job's deadline (hearth/docs/design/domain/steward.md, 5.2) and the store's free space; only git's exit and the tail of its errors are read.

## 3. The pipeline

```
revision ──git──▶ 1 decode ──▶ 2 compose ──▶ 3 validate ──▶ 4 exports ──▶ 5 conditions
                                                                              │
                  spec ◀── 8 assemble ◀── 7 collisions ◀── 6 files ◀──────────┘
```

Step 1 and discovery run once per revision; the rest once per target, and step 4 reads other targets of the same environment. A target's settings are composed when it renders, another target's task or placement when step 4 reads it, and none is kept past its use (`crates.md`, 1.4).

**Four results,** each made from the one before: the revision's inputs, decoded (3.1); a target's settings, composed, validated and given what it reads of its environment (3.2 to 3.4); its admitted entries, the concrete declarations once checked and free of collisions, with the files they make (3.5 to 3.7); and its spec (3.8). A diff compares two revisions' specs (`diff.md`, section 1). The records between them are the code's: a stage that adds no rule of its own adds no record.

### 3.1 Decode

- **The listing classifies every path** (hearth/docs/design/protocol/format/README.md, section 2) as a configuration document, a declaration, a template, a static file under `tasks/*/tree/`, or a file outside the four roots, read only if a `file` setting names it. A path under the four roots that the format does not allow is a `repository` error: in `envs/`, anything but an `<env>.json`; directly in `hosts/<base>/`, a `.json` that is neither of its documents; a file directly in `hosts/` or `tasks/`; a link where a document or declaration must be; any submodule. An absent `fleet.json` or `<base>.json` is an empty layer.
- **The domain asks for blobs by kind, and the protocol layer decodes each:** JSON through skein's tokenizer into `hearth-protocol-config`, which refuses duplicate keys, non-integer numbers and U+0000, drops `_note`, and gives typed values with their line and column; a declaration into its fixed shape (hearth/docs/design/protocol/format/tasks.md, 1), its expressions and interpolated strings through `hearth-protocol-template`; a template into a flat sequence of instructions (`crates.md`, 1.3). A static file, or a file a `file` setting names, is never decoded or held: the protocol layer streams it from `cat-file` into the store's cache by SHA-256, hashing as the bytes pass, and the domain receives its object id, size and digest.
- **A decode error fails what depends on the file,** not the revision: the targets that have a broken task or compose a broken document; only the whole revision's limits fail it all (2.3). A repository error fails the targets of its task or base; one of neither (a stray file in `envs/`, or directly in `hosts/` or `tasks/`) fails no target and is the revision's error all the same.
- **Errors of the revision print whatever targets are asked:** step 1 and discovery read every document and declaration. A target's error prints only when that target is asked, and a task's templates are read only when an asked target has the task (3.3). While a `hosts/` document is refused, a named target discovery cannot find is left out under that error, which exits 2, not 64's unknown target (hearth/docs/design/client/README.md, 3.3).

### 3.2 Compose and merge

- **Discovery** makes the targets from the listing and the bases' `members` (hearth/docs/design/protocol/format/targets.md, section 1): base, member, environment, name. Two targets of one name are an error.
- **Each target's layers merge** in the format's order by RFC 7386 (hearth/docs/design/protocol/format/targets.md, 1.2, and hearth/docs/design/protocol/format/README.md, 4.1).
- **Presence** comes from the base, member and base-env layers alone (hearth/docs/design/protocol/format/targets.md, 1.3); the `hearth` task joins every target but a container (hearth/docs/design/protocol/format/tasks-hearth.md, 1), and the composed settings are restricted to the present tasks.
- **Every value keeps its provenance,** the layer, file, line and column that set it, so a later error names the right file. An object's is the layer that made it (RFC 7386's `result = target`), not a later one that patched its members, and each member keeps its own.

### 3.3 Validate

- **Once per revision, each declaration is checked against itself:** its schema is well formed and its defaults have their types. **For each task an asked target has,** every `settings.<name>`, `tasks.<task>.<setting>` and `exports.<task>` its templates and expressions name is declared (hearth/docs/design/protocol/format/schema.md, 1.4), its templates are used and its scripts declared.
- **Per target, the composed settings are checked against the schemas** (hearth/docs/design/protocol/format/schema.md, 1.5), and defaults filled at every depth (hearth/docs/design/protocol/format/schema.md, 1.3); `debian` and `placement` against their fixed schemas.
- **Owners of what persists have fixed ids.** Once the declarations are concrete (3.5), every owner and group of a data path, a secret file or an identity file is a declared user or group with a fixed `uid` or `gid`, package users included, and a download's is `root` or such a package user or group (hearth/docs/design/protocol/format/tasks.md, 6; hearth/docs/design/protocol/format/README.md, 8.1). Why: the packages layer is rebuilt from scratch, and an id a postinst took could move under data that keeps the old one.

### 3.4 Exports

- **For each task whose exports a task of this target reads,** render composes every target of the environment that has that task, validates that task's settings there and evaluates its exports (hearth/docs/design/protocol/format/exports.md, section 1), once per revision for all their readers. No cycle is possible: exports depend only on their own task's settings and on `target` (hearth/docs/design/protocol/format/exports.md, 1.1).
- **Placements are read alike:** `target.containers` holds the environment's targets placed on this one (hearth/docs/design/protocol/format/targets.md, 1.5), each entry the fields of that container's `target.placement`. A container's `on` names a target of its environment that is not a container, and a build environment has none (hearth/docs/design/protocol/format/targets.md, 1.4); the containers placed on one host have id ranges that overlap neither each other nor the builder's [M5] (hearth/docs/design/protocol/format/targets.md, 1.5).
- **An error in another target's exports fails this render too,** naming both targets: a spec never reads a guess. What else fails on the exporter (another task, its own `requires`, its files) leaves its exports as they are, and its readers render. When they can't be computed (a document the exporter composes is broken, the task's settings fail validation, an export's expression fails, or a refused `hosts/` document hides which targets have the task, 3.1), each reader fails with an `exported` error (`errors.md`, 2). A container whose placement can't be composed fails its host.
- **A container reads of its host its placement alone,** that `on` names a target of its environment that is not a container; its context holds nothing else of the host (hearth/docs/design/protocol/format/templates.md, 7), and its data part none of the host's policies (`spec.md`, 1.3). So what else fails on the host leaves its containers to render. A host whose placement can't be composed fails each container placed on it: while a document the host composes is refused (3.1; `errors.md`, 2), the container is left out under that error, which prints once and without a target (`errors.md`, 1); a host whose documents compose no placement gives each container asked a `composition` error at its `on`, beside the host's own when the host is asked too.

### 3.5 Conditions

- **`requires` first** (hearth/docs/design/protocol/format/tasks.md, 9): a false guard is an error with its message.
- **Then every declaration entry:** `for_each` expanded, `if` evaluated for each element, keys and string fields interpolated, typed fields evaluated (hearth/docs/design/protocol/format/tasks.md, 2). The result is each task's concrete declarations, of every kind that document lists.

### 3.6 Files

- **Every file of the tree is produced and digested:** a static file of `tree/` and a `copy` by the digest step 1 gave it, a template of `tree/` or `templates/` by evaluating it in the task's context (hearth/docs/design/protocol/format/templates.md, 7), a link by its target; `customize.sh` and `software.sh` fragments likewise.
- **A rendered file is written to the store's cache by digest** once complete and dropped from memory: a target holds one rendered file at a time.
- **Each output is bounded** (2.3); an undefined value, a type error or an exceeded limit is an error at the template's line.
- **Render adds what tasks may not write themselves:** `/etc/subuid` and `/etc/subgid`, one text of subuid(5)'s `<user>:<start>:<count>` line for each user with a subordinate range, by name, two files `0644 root:root` in the config part's tree and neither when no user has a range; a `tmpfiles` line of type `d` for each data entry's mode and owners (hearth/docs/design/protocol/format/tasks.md, 6 and 7); and the initramfs configuration (`spec.md`, 1.2).

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
| a path a `live` entry covers | several entries, of one task or several: their consumers are a union (hearth/docs/design/protocol/format/tasks-live.md, 1) |
| a uid or gid; a subordinate range | one user or group; ranges never overlap |
| packages, initramfs modules | a union |

- **Identically** is field for field once defaults are filled; positions and the tasks themselves are not compared.
- **Paths collide when one is the other or lies beneath it;** two claims of one task collide as two tasks' do.
- **A collision is an error naming both tasks and both lines,** where os overwrote silently, in alphabetical order of task.

### 3.8 Assemble

- **Each part's record** (`spec.md`, section 1) is built from the admitted entries and the produced files, sorted as hearth/docs/design/protocol/format/README.md, 4.3 says, encoded canonically and digested.
- **Render gives the resolution key** (`spec.md`, 2.1), and the layer keys when the store holds that key's resolution (`spec.md`, 1.5); otherwise the key is needed, and the layer keys unknown until it is resolved (`spec.md`, 2.7).
- **The live part maps each covered path to its consumers** (hearth/docs/design/protocol/format/tasks-live.md, 1). Render decides nothing about a change: the builder pairs two specs' live parts with two outputs' tree files when it plans a trial (hearth/docs/design/domain/slots/trial.md, 7; hearth/docs/design/domain/steward.md, 3.3).
- **The output is the spec.** Every file it names is in the store's cache by digest. Specs are never stored: a builder renders again when it needs one (hearth/docs/design/domain/steward.md, 3.3).

### 3.9 Checks and probes [M5]

- **A task's checks are evaluated as its other declarations** (3.5), and their rendered form enters the checks part (hearth/docs/design/domain/operating.md, 2.2).
- **A probe with `from` is exported:** render puts it, rendered, into its task's exports with the target it probes, and each target of the environment that has the task `from` names renders it into its own checks part (hearth/docs/design/domain/operating.md, 2.6). A change to one probe changes the probed target's and its probers' parts, and no other.
- **Refused at render:** a `blessing` check with `from`; a `from` that no target of the environment has; a command check whose `user` is `root`, or neither `nobody`, a sysusers user nor a package user a task of the target declares (hearth/docs/design/protocol/format/tasks.md, 6; hearth/docs/design/domain/operating.md, 2.2; hearth/docs/design/domain/activator/operations.md).

### 3.10 Costs

- **A revision is decoded once** and shared by all its targets. Decoded documents, templates and static files' digests are kept by git object id, so the next revision, a diff's second side among them (`crates.md`, 1.4), decodes only the blobs that changed; object ids name contents, so this memory changes no result.
- **A target's render** composes, validates, evaluates and hashes its files in milliseconds, so rendering every target of two revisions, as a diff does, is cheap enough for every change.

## 6. Its world

### 6.1 Worlds

- **Render's domain world** runs `hearth-domain-render` under a scripted parent that plays the protocol layer and the store: typed documents, declarations and templates in an order and at a pace drawn from the seed. It checks every world's invariants (`skein/docs/foundation/testing-strategy.md`, section 6).
- **Machine worlds** run each protocol machine against recordings of the real tool, cut at random (`skein/docs/foundation/testing-strategy.md`, 2.4).
- **The builder's system world** runs render with the fakes of git and mmdebstrap on the fake machine (hearth/docs/design/testing/fake-machine.md, hearth/docs/design/testing/tool-fakes.md). Its budgets are counts, the requests a run makes, asserted against 3.10's costs (hearth/docs/design/testing/README.md, 4.3).

### 6.2 Golden repositories

- **hearth's own:** `tests/render/<name>/`, each a repository's files and the expected printed spec of each target (`expected/<target>`), or the expected errors. The fake git serves a directory; one test in the real tier builds a real repository from it and renders through real git.
- **The repositories:** `fleet` (a miniature of the fleet: x1-prod, east-prod, west-prod, carbon-prod, carbon-lab, chat-prod), `merge` (presence, defaults, removal, maps, lists), `templates` (each statement, filter and whitespace rule), `exports` (the VPN, the mail pair, two environments), `cluster` (stoa's members), `errors` (one case per kind of `errors.md`, 2 a repository causes; `tool` and `resolution` are builder-world cases with a fake failing, 6.6), `resolution` (a target whose resolution the store holds, printed with its `keys` line).
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

  Digests and keys are cut to 12 hex digits, as every table of the client's is (hearth/docs/design/client/README.md, 3.2); its JSON gives them whole. A link prints its target, and an empty file the digest of no bytes (`e3b0c44298fc`). With the resolution in the store, its line reads `resolution 0c7d9e41c0d2 known`, and a last line gives the layer keys: `keys  packages … software … config …`.
- **`HEARTH_BLESS=1` rewrites the expected files and still fails,** so a blessed change is always reviewed.

### 6.3 Properties

Over generated repositories, valid by construction and then mutated:

- **Determinism:** one revision renders to the same digests in every world, whatever the order of blob arrival and chunking: render's world over seeds, the protocol world's pipes cut at every byte, the simulated client and real git, compared by the whole digests the JSON form prints.
- **Representation:** reordering a JSON object's members, reformatting whitespace or editing a `_note` changes no digest.
- **RFC 7386's laws:** `merge(t, {}) = t`; `merge(merge(t, p), p) = merge(t, p)`; a non-object patch replaces; a key the patch lacks keeps its value; no result holds `null`.
- **Locality:** a setting in one target's own document that no export reads changes that target's spec alone; one that an export reads changes the exporting target and its environment's readers, and no other; a production file never changes a lab spec.
- **Order:** the order in which render takes tasks and targets changes nothing; a collision names the same pair, the task first in name order at `at` and the other at `also` (`errors.md`, 1).
- **Digests:** two different records of a part give different digests over generated pairs; the encoding's own laws are hearth/docs/design/domain/store/README.md's (section 9).

### 6.4 Fuzzed decoders

- **Every decoder is fuzzed** on its own (`skein/docs/foundation/testing-strategy.md`, 2.4): configuration documents and declarations, templates and expressions, `ls-tree` and `cat-file` streams, `Inst` lines and `Packages` stanzas; and what render hands the store and its tools, store names and argument vectors, swept in the builder's world. Corpora start from the golden repositories and the recordings.

### 6.5 MiniJinja as an oracle

- **Test code is ordinary Rust,** so it may run MiniJinja 2 with `trim_blocks`, `lstrip_blocks` and `keep_trailing_newline` on, strict undefined, and hearth's own filters registered.
- **Generated templates of the subset** render byte for byte the same in both. Where hearth differs on purpose (undefined, `first` on an empty list), the generator avoids it and the test lists the difference. hearth keeps hearth/docs/design/protocol/format/templates.md 4's precedence, which binds `~` below `+ -` and `is` below the comparisons, where MiniJinja binds `~` above them and `is` with the filters: both are listed, and the generator parenthesizes a `~` beside `+ -` and an operand of `is` that is not a lookup chain.

### 6.6 Resolution's fakes and conformance

- **`hearth-fake-mmdebstrap`** resolves against the fake machine's package universe and writes `Inst` lines and `Packages` indices of the real shape (hearth/docs/design/testing/tool-fakes.md).
- **Conformance** runs real `mmdebstrap --simulate --mode=chrootless` against the test tier's local mirror, and the fake against the same universe, and compares closures (hearth/docs/design/testing/README.md, section 3). Its runs are recorded, and the recordings are what `spec.md`, 2.3 and 2.5 are held to.

### 6.7 os's goldens

- **Inspiration, not an oracle:** os's fixtures are rewritten in hearth's format where the case still exists, with hearth's expected output, reviewed; never compared with os's, which strips final newlines and records no modes.

## 7. Open questions

- **A decompressor for the indices:** a chrootless simulate run with `--skip=cleanup/apt/lists` leaves `InRelease` and the decompressed `Packages` files, as the recordings show (6.6), and M1 refuses a compressed one (`spec.md`, 2.3). One is needed only if an apt keeps them compressed (`Acquire::GzipIndexes`).
- **The `Inst` line with several origins** (`(2.41-12 Debian:13.1/stable, Debian-Security:13/stable-security [amd64])`), to verify against recordings.
- **Chrootless or unshare for resolution:** os and the bash fleet resolved chrootless and built unshare; a reason for preferring unshare was found once and not recorded. A conformance scenario compares the closures of both modes (hearth/docs/design/testing/README.md, 3.2), and hearth/docs/design/domain/build/packages.md, 7 refuses any build whose installed set differs from its closure. If the modes ever disagree, resolution moves to unshare and needs the builder's subordinate range. A lead, from mmdebstrap 1.5.7's code: chrootless installs the essential set with `dpkg --force-depends` from outside and the rest with an outside `apt-get install`, while unshare and root skip that last step for `--variant=custom`, so the two may diverge for containers' variant.
- **apt's version and solver:** recorded beside the closure from M2 (`spec.md`, 2.4), not keyed.
