# Landed drafts

A draft landed in one of a project's repositories, which its writer says
is ready to start a goal (`../../draft/surfaces.md`, section 5;
`../../draft/whole-story.execution.md`, P8). The connector keeps a row
for each, as it keeps a person's Away (`attention.md`, 4): it reads the
forge once, when the writer says so, and nothing after. Work lists the
ready ones and starts a goal from each (`../../client/pages/work.md`, 1,
*Work*); `temper ctl draft` is where a writer says so
(`../../client/ctl/areas.md`, area 7).

## 1. Its row

- **`LandedDraft { repository, path }`:** the repository, the
  connector's identity of an adopted repository, its forge's number and
  its own; and the path in it. A path is 1 to 1024 bytes of UTF-8,
  relative, ending `.md`, with no empty, `.` or `..` segment between its
  `/`s. At most `DRAFTS`, 64, a project; held in memory, restored whole
  at the restart step, as `Away` is; a row past the bound is refused at
  restart, as each family's is. It holds:
  - `project`, the repository's project when the row was written;
  - `branch`, the landing branch read: the repository's default branch,
    `Repository.settings.default_branch`, as a tier following the
    landing resolves it (`../after-landing.md`), at most 1024 bytes;
  - `commit`, that branch's head when the file was read there, the
    forge's id of it;
  - `title`, the draft's title: the rest of the first line beginning
    `# ` in the file's first 4096 bytes, trimmed of spaces and cut to 200
    bytes at a character's boundary; empty with no such line, or where
    those bytes are not UTF-8;
  - `writer`, the party who last said it ready, and `at`, the wall then;
  - `chat`, option: the chat its writer names as where it was written;
  - `goal`, option: the goal last started from it;
  - `ready`, bool.
- **Listed** while `ready` and with no `goal`: Work's Drafts lists it
  (`../../client/pages/work.md`, 1, *Work*). A row with a goal, or not
  ready, is **unlisted**.
- **Erased** with its project, and with its repository's forge, as the
  repository's landing log is (`../after-landing.md`, section 10),
  marking the subject `drafts` (3).

## 2. Saying so: `SetDraft`

- **`SetDraft { project, repository, path, change }`**
  (`../../client/forge/records.md`) is admitted by the core as `SetAway`
  is (`attention.md`, 4): the project its scope, a party with a role
  there, in a project not being deleted. `repository`, an option of an
  owner and a name: the repository adopted into the project under that
  owner and name, on a forge not ended, the lowest forge number first
  where two match; none, the project's home. `change` is one of:
  - **`ready { chat }`:** the connector reads, in this order, the
    repository's landing branch, `Read::Branch`, then the path at the
    head it answered, `Read::File { head, path, max_bytes: 4096 }`,
    holding the request in memory while they are out, as an adoption's
    reads are held (`adoption.md`). On a file, in the request's
    decision, the row is made or replaced: `branch`, `commit` and
    `title` from the reads; `writer` the party and `at` the wall;
    `ready` true; `goal` none, so that a draft said ready again is
    offered for a new goal; `chat` the request's, else the row's.
  - **`not_ready`:** no read; in the decision, the row's `ready` false,
    the rest kept.
  - **`started { goal }`:** no read; in the decision, the row's `goal`
    the request's, the rest kept. The client sends it once the goal it
    started from the draft is created (`../../client/pages/goal.md`, 3).
- **Its answers:** *kept*, the row as it stands once committed, the
  answer held behind the commit, in memory, as `SetAway`'s; a row that
  comes out the same writes nothing and is answered kept. Or refused,
  writing nothing:
  - `no repository`: no repository adopted into the project matches, or,
    with none named, the project has no home;
  - `path`: the path breaks 1's rule;
  - `missing`: the forge answers the branch or the path missing;
  - `not a file`: a directory at the path;
  - `unreachable`: any other read's failure, the forge not connected
    among them;
  - `unknown`: `not_ready` or `started` with no row;
  - `full`: a new row with the project at `DRAFTS` and none of its rows
    unlisted. A new row at the bound otherwise erases the project's
    unlisted row with the oldest `at`, the lower path first on a tie, in
    the same decision.
- **A restart** while its reads are out loses the request: it is
  answered unknown, as a set up's is, and its writer says so again. A
  restart after its commit answers it unknown too, as `SetAway`'s held
  answer is; the row, restored whole and listed in `drafts`, says what
  stands.
- **Two at once** for one row each read and decide on their own; the
  later decision stands.

## 3. Its document: the subject `drafts`

- **Of the project's scope,** for a party with a role there, its window
  unused, a subject number of its own, as `changes` has
  (`../../client/forge/records.md`, *A project's landed drafts*). Its
  document lists every row of the project, listed or not, newest `at`
  first, then the lower path: the client chooses what it draws.
- **Marked** at each row written or erased.

## 4. Open questions

- **Saying a draft is ready on the web, or in a design session in the
  terminal** (`../../draft/surfaces.md`, 11.2). Default: `temper ctl
  draft ready` alone; Work lists and starts.
- **When a draft landed:** the row knows when its writer said it ready
  and the commit it was read at, not the commit that brought the file.
  Default: Work says "ready since", from `at`.
- **A goal started by proposal** (`../../client/pages/goal.md`, 3): the
  client sends `started` only once the goal is created, so a proposal
  accepted later leaves its draft listed until its writer says
  `not_ready` or `started`. Default: so.
- **A draft in each of two repositories** of one project: Work names its
  directory, not its repository (`../../client/pages/work.md`, 1).
  Default: so, `temper ctl draft list` naming both.
