# The agent's forge reads

This document holds the agent's forge reads, split from [`tools.md`](tools.md).

## 1. Forge reads

- **`repository`,** one of the project's repositories as `owner/name`:
  among its adopted repositories of any role, on forges not ended, the
  one named, or with no name the only one; with several and no name, or
  a name none has, refused, naming them. The root resolves it to the
  repository's resource as it builds a change's delegate (1),
  before it decides the call, so that the call's authority is checked
  on that resource (`../routing.md`, 2) and the run never names a number.
- **The forge's reads are three tools,** each taking a `what` that names
  the read, `repository` beside it, and the union of its reads' members,
  each member described once (`../../draft/archive/tools-by-role.md`, 4; the split
  in three keeps each within `declarations.md`, 3's bound):

  | Tool | `what` | Required | Optional | Formerly |
  |---|---|---|---|---|
  | `read_forge` | `branch` | | `branch` | `read_branch` |
  | | `branches` | `prefix` | | `read_branches` |
  | | `compare` | `before`, `after` | | `read_compare` |
  | | `file` | `path` | `commit` or `branch`, `max_bytes` | `read_file`, its `head` |
  | | `protection` | | `branch` | `read_protection` |
  | | `settings` | | | `read_settings` |
  | | `collaborators` | | `page` | `read_collaborators` |
  | | `permission` | `user` | | `read_permission` |
  | `read_issues` | `items` | | `since`, `page`, `item_kind` | `read_items` |
  | | `item` | `number` | `comment` | `read_item`, its `after` |
  | | `pull` | `number` | | `read_pull` |
  | | `pull_for` | `branch`, `base` | | `read_pull_for`, its `head` |
  | | `reviews` | `number` | `page`, `skip` | `read_reviews` |
  | | `remarks` | `number`, `review` | `page`, `skip` | `read_remarks` |
  | | `pull_files` | `number`, `commit` | `page`, `skip` | `read_pull_files`, its `head` |
  | `read_ci` | `checks` | `commit` | | `read_checks` |
  | | `job` | `commit`, `run`, `job`, `attempt` | `max_bytes` | `read_job`, its `head` |
  | | `statuses` | `commit` | `page` | `read_statuses` |

  - **A member keeps one meaning** across the three: `commit` a commit's
    id, 40 or 64 lowercase hexadecimal digits; `branch` a branch's name,
    in `read_forge` left out or empty for the repository's default branch
    (below), for `pull_for` the branch the pull merges from, and `base`
    the one it merges into; `before` and `after` the earlier and later commits of a
    comparison; `comment` a comment's id, the item's comments after it,
    0 when left out; `number` an issue's or a pull's; `review` a review's
    id; `user` a forge user's id; `page` from 1, 1 when left out; `skip`
    the rows of `page` already read, 0 when left out; `since` a time in
    nanoseconds, 0 when left out; `item_kind` `issue` or `pull`;
    `max_bytes` the read's limit when left out.
  - **`job`'s members** are exactly as `checks` and `statuses` name each
    Actions job: the job attempt skein resolved from the check's link (its
    run, the forge's own job id, its attempt: skein's `forge/forgejo.md`,
    2), never the link's index of the job in its run, which the answer may
    show beside it but no read takes.
  - **The declared form says what each `what` needs,** so that a run's
    first call is well formed without its page: the `what` member's
    description, the one description the three declared forms carry,
    lists each `what` that requires members, with them, in the table's
    order; `what`s that require the same members share one entry, and a
    `what` that requires none is left out (`declarations.md`, 2). Within
    a member's 120 bytes, the three read:
    - `read_forge`: `Needs: branches prefix; compare before, after; file
      path; permission user. No branch: the default branch.`
    - `read_issues`: `Needs: item, pull, reviews number; pull_for branch,
      base; remarks number, review; pull_files number, commit.`
    - `read_ci`: `Needs: checks, statuses commit; job commit, run, job,
      attempt.`

    The `Needs:` clause, up to its first full stop, is the table's
    required members exactly, held by a step test that reads both: in an
    entry, the `what`s come first, comma-separated, and the first space
    not after a comma starts its members. The whole forms keep their own
    `what` description, and the page's `what=…: requires …` lines come
    from the table (`../../protocol/documentation.md`, 8).
  - **A refusal at decode reads as a sentence,** to the person who sees
    it in a step line as much as to the run, naming the tool, then the
    read as `what=NAME`, then what is wrong, deciding nothing:
    - a `what` missing a member it requires names every member that
      `what` requires: `read_forge: what=file needs the member path.`,
      `read_ci: what=job needs the members commit, run, job and
      attempt.`, the last two joined by "and";
    - an unknown `what`, or none, names the tool's reads without echoing
      the call's: `read_ci: what is one of checks, job or statuses.`, the
      last two joined by "or";
    - `file` given both `commit` and `branch`: `read_forge: what=file
      takes commit or branch, not both.`

    A member another `what` takes is ignored, as a member no decoder
    reads is. A member's wrong type or range is answered as any host
    tool's is (`../routing.md`, 2).
  - **A branch left out is the default branch.** `read_forge`'s
    `branch`, `protection` and `file` take `branch` left out, or empty,
    as the repository's default branch: the one its adoption keeps as the
    forge's own, as last read (`../forge/adoption.md`, 8), which Ask
    temper's `file` reads at too (`ask-temper.md`, 1). smith decodes it
    as an empty name, and the root, resolving the repository's name,
    puts the default branch's name in its place, as it puts the
    configured read limit in place of a zero `max_bytes`, before the
    read is admitted, so that authority and the connector see the named
    branch. The answer is the named read's: `branch` the head commit,
    `protection` the rule. The former names decode the same way, so a
    `read_branch` or `read_protection` of an empty name, which the forge
    refused, reads the default branch too. A default branch the forge
    has moved since its last read answers as a missing branch, whose
    refusal points to `what=settings`.
  - **A file at a branch's head.** `file` reads at `commit` where it is
    given, as before; otherwise at the head of `branch`, or of the
    default branch where `branch` is left out too. smith decodes it as
    the engine's `Tool::ReadBranchFile` (below); the root resolves the
    repository and the branch's name as above, reads the branch, then
    reads the file at the head that read answered, each read admitted
    and correlated as any forge read is
    (`../forge/keeping-up.md`, 3). The answer is `file`'s at that commit,
    exactly: its heading names the commit it read (`PATH at COMMIT:`),
    and its bounds, cuts and refusals are a commit read's (*File reads
    fit the host answer's bound*, below). A branch read that fails
    answers as `what=branch` of that branch would, its refusal and
    guidance included (*A refused branch read says what to read
    instead*, below), and no file is read.
    - **The engine's shape** (`temper-engine-domain-translate`'s `tools.rs`,
      beside `ReadNamed`), the contract between smith's decoder and the
      root:

      ```rust
      /// A file at a branch's head: the branch, then the file at the head it answered.
      /// An empty `branch` asks for the default branch; a zero `max_bytes` the configured read limit.
      ReadBranchFile {
          repository: Option<Box<[u8]>>,
          branch: Box<[u8]>,
          path: Box<[u8]>,
          max_bytes: u32,
      },
      ```

      `repository` and `max_bytes` mean what `ReadNamed`'s do, and
      `path` is `file`'s as smith decodes it, a lone dot the root. A
      named read's empty `Branch` or `Protection` branch means the
      default branch too, said on `ReadNamed`.
    - **A busy read,** the branch's or the file's, waits in the host's
      bounded backoff, as any read does (`../forge/keeping-up.md`, 3),
      and the run hears one answer: the file's, or the branch's refusal,
      never the branch's head alone.
  - **Each `what` is answered exactly as its former tool was:** the same
    route, renderers, bounds and pages (`../../draft/archive/forge-reads-targeted.md`);
    the forms above that the former tools refused are `read_forge`'s
    additions. The former tools' names are decoded too, each as its
    `what`, but declared to no run (`declarations.md`, 4).
  - **Words name a read by its tool and `what`,** as `read_forge
    what=file`: the answers' and refusals' guidance below, the brief's cut
    lines (`room.md`, 2) and the stock texts.
- **A page the answer's bytes cut names how to read on.** A
  `read_issues` `reviews`, `remarks` or `pull_files` page cut at
  `forge.client.answer_bytes` (skein's `forge/vocabulary.md`, 1.2 and 1.3)
  ends with `Page cut: read on with page P, skip S.`, its own page and
  the rows read of it, so the next read loses no row; a page that ends
  uncut and has more says `More pages available.`, as today.
- **`read_forge`'s `permission` names the person:** its answer is the permission,
  the forge user's id and the login the forge names them by, as
  `Write; forge user 3, login ada` (skein's `forge/vocabulary.md`, 1.3), so that a
  run can match a name to an id; with no login from the forge, the same
  without it. `collaborators` lists ids, for `permission` to
  name.

- **A compare is by names, with no contents.** `read_forge`'s `compare` answers
  how many commits `after` has that `before` lacks and how many it lacks,
  whether `after` contains `before`, and the changed files' paths, at
  most `forge.client.rows` (8 as configured) with how many it left out
  (skein's `forge/vocabulary.md`, 1.2 and 1.3). It reads no file's contents, so two
  branches however far apart compare. Its answer, a line each:
  `AFTER is A commits ahead of BEFORE and B behind it.`, then `AFTER
  contains BEFORE.` or `AFTER lacks BEFORE.`, then `Changed files: N`,
  each path it holds, and, where it holds fewer than N, `[K not listed:
  read_issues what=pull_files lists a pull's files; read_forge what=file
  reads a path at each commit]`. A file's contents are `file` at `before` and at
  `after`, a side `Missing` where the file is absent; its description
  says so. A `compare` refusal (a forge that cuts its own listing)
  names its bound as any read's does, says that the commit ids were read
  and that no other id helps, since a run that isn't told retries with
  one it made up, and directs it to `pull_files` or `file`. A
  successful comparison whose rendered form exceeds the host answer's
  bound returns measured-size feedback with the same recovery instead of
  losing its body at the worker.
- **File reads fit the host answer's bound.** The adapter measures rendered
  file, directory and pull-file answers, including metadata and byte escaping.
  A file keeps the largest UTF-8 prefix that fits with its metadata,
  recovery feedback when room permits, and a visible `[file truncated]`
  marker, counting escaping (up to four output
  bytes per input byte). An explicit `max_bytes` still bounds the raw forge
  prefix; host fitting may cut it further. A cut is never the whole file,
  and there is no offset. A host-fitted prefix also names the measured
  rendered size and effective body bound, and gives configuration recovery
  when room permits; tiny bounds retain the cut marker alone. If metadata
  leaves no room for content, bounded error feedback directs a shorter path
  or raising the limiting core or agent allowance;
  lowering `max_bytes` cannot fit metadata. An oversized directory answer
  is an error naming its rendered byte size and numeric host bound. An oversized
  pull-file page retains a changed-path inventory as room permits, marks
  content omitted separately from absent sides, marks omitted paths, and
  keeps whether more pages follow. Omitted paths are recovered through
  directory reads at both commits. It directs selected `file` reads
  at the head for after and the comparison's base commit for before;
  the base commit must be resolved separately. Advancing pages alone cannot
  recover the oversized page. A file or pull-file forge size refusal, whose
  measured size is unknown, names that uncertainty, the forge bounds with
  their configured figures and the numeric host bound with the same
  tool-specific recovery. A compare's forge size refusal follows the compare
  guidance above. At a host bound too small
  for the guidance, the refusal itself is cut to fit.
- **A refused branch read says what to read instead.** Both refusals
  are right: `branches` takes a nonempty prefix, an empty one
  ending as `Error::Empty` before any request (skein's `forge/forgejo.md`,
  section 2), and `branch` of a name the forge lacks is
  `Error::Missing`. That is so for a goal's own branch where its goal
  holds none (a chat's `goal` names none, 2), and for a run branch
  before its first push (`../forge/branches.md`). `branches`'s
  `prefix` is declared "A nonempty prefix of the names, as temper/."
  Each refusal keeps its first line, `Forge read failed: empty` or
  `Forge read failed: missing`, and goes on after a full stop:
  - `branches`, `Empty`: "read_forge what=branches needs a nonempty
    prefix, as temper/ for temper's branches. read_forge what=settings
    names the default branch, and read_forge what=branch reads one branch
    by its name."
  - `branch`, `Missing`: "The forge has no branch of that name in
    this repository. read_forge what=settings names its default branch,
    and read_forge what=branches lists the names under a prefix, as
    temper/. A goal's
    own branch is there only where its goal holds one, and a run branch
    from its first push."
  - `branches`, `TooLarge`, first line `Forge read failed: too
    large`: "The names under this prefix pass forge.client.answer_bytes,
    the one bound a branch list has: name a longer prefix, as
    temper/goal/ in place of temper/, or read one branch by its name
    with read_forge what=branch." No row bound is named, since none applies to
    branch names (skein's `forge/vocabulary.md`, 1.7). The tool's description says
    the same in its words: the names fit forge.client.answer_bytes, and
    a longer prefix reads fewer.

  Every other tool's `Missing` and `Empty` keep their one line. A forge read
  refused as too large names the bound it passed and the size it met, in a
  bound's words (`../forge/change/repairs.md`): `Forge read failed: too large: past
  forge.client.rows (8): 22 rows.`, then how to read on, as its tool's
  description says. Each forge read's description names the bounds its
  answer meets and how to read past them: a listing by its next page, a
  file by path within `max_bytes` (`file`), a pull's files by page
  or a path at the head or the base, a comparison by names with each
  file's contents by path, a branch list by a longer prefix. The refusal
  stays marked an error and is cut to fit the host answer's bound, as
  the compare's is.

## 2. Open questions

- **A collaborator's login in `read_forge`'s `collaborators`** (`tools.md`). Default: ids,
  with `permission` naming each.

- **`permission`'s login held to the settings family** (`tools.md`). It
  reaches every run with the forge reads, as the forge's own answer about
  a user of a repository that run may already read; holding it would be a
  condition in `temper-engine-domain`'s render. Default: every such run.

- **A branch read naming its branch** (`tools.md`, *The forge's reads
  are three tools*): `read_forge what=branch` with no branch answers the
  default branch's head commit alone, as every branch read answers, so a
  run that wants the default branch's name reads `what=settings`.
  Default: the commit alone, until runs are seen to read settings for
  the name; naming the branch would change every branch read's answer.
