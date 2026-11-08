+++
id = "ways-view"
part = "read"
kind = "feature"
needs = ["ways-read", "ways-derive"]
files = ["temper/crates/temper-engine-domain/src/route/ways.rs", "temper/crates/temper-engine-domain/src/route/ways/**", "temper/crates/temper-engine-domain/src/route/kinds.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/tests/engine/domain/tests/suite/ways.rs", "jig/crates/jig-domain-core/src/reads.rs", "jig/crates/jig-domain-core/src/reads/settings.rs"]
effort = "high"
minutes = 180
+++

## Goal

Answer the forge's read `View` in a project's scope: parse a `view`
block's address and answer a way's path, a repository branch's path, a
waiting proposal's path before and after, or a charter's card, from the
derivation `ways-read` gathers for, so that a reply's views draw what is
stored (`temper/docs/design/client/README.md`, 5.16 and 4.6, *A view*).

Acceptance:

- `route/ways/view.rs` (or beside `ways.rs`) parses an address, words
  split on ASCII spaces, case-sensitive but for a way's name:
  - `way NAME`: NAME the rest of the line, matched against
    `config.ways_stock`'s names with any case and single spaces;
  - `path OWNER/NAME [BRANCH]`: a repository adopted into the project, as
    the settings inventory names it, and a branch, its landing branch when
    left out;
  - `proposal tN/K` and `proposal pN/K`, as `see`'s parser reads them
    (`route/see.rs`, `proposal_name`);
  - `charter N`;
  - `path …` and `charter N` may end `@revision N`.
- Its answers, `ForgeReadAnswer::View`:
  - a way: `View::Way` with the way's name, line and path over the
    project, and `current` where the project runs it (`ways-read`'s
    `document`);
  - a path: `View::Path`, `ways-read`'s `inputs` over that repository and
    branch, a branch other than the landing branch one segment with a
    chat's start, judged against that branch, its bar named by it (extend
    `inputs` with the repository and branch it takes; its other callers
    keep theirs);
  - a proposal: read as `see proposal` reads one (the core's
    `NamedAction::ReadProposal`, answered later, as `RevisionSetting`
    reads are answered after their metadata loads); waiting, `View::Proposal`
    with `before` the project's own path and `after` the path with the
    proposal's items of temper's kinds (`Domain::proposal_settings` of its
    review's key) laid over, those of other projects left out;
  - a charter: `View::Charter` with its card: `who` "an agent"; `name`
    its place on the project's path ("the coding agent", "the reviewer",
    "the coordinator", as `notes/contracts.md`, section 2 names them), else
    "charter N"; facts "Models" (its order as shown names with their
    default efforts, joined by ", ", then ", then the rest" where `rest`),
    "Can" (`ways::can`), "Wakes" (its wake policy in words: "at once on
    words, results, failures and questions" for jig's default, its other
    reasons named as jig names them), "Asked to" (`ways::first_sentence`);
  - otherwise `View::NotDrawn`, one line: `no way NAME; the ways are You
    approve, Agents review and Draft first.`; `no repository OWNER/NAME in
    this project.`; `no branch BRANCH in OWNER/NAME.` where the forge knows
    none; `proposal tN/K is decided; a decided proposal's paths aren't
    drawn yet.`; `no proposal tN/K.`; `no charter N.`; `revision N isn't
    drawn yet; views draw the newest settings, revision M.` for an
    `@revision` that is not the newest; `no view ADDRESS; views: way NAME ·
    path OWNER/NAME [BRANCH] · proposal tN/K · charter N.` for any other.
- The stub `ForgeRead::View` answer of `route/kinds.rs` is replaced.
- Tests in the domain world's `tests/suite/ways.rs`, one table a row an
  address over a project with a home repository with CI and a main tier,
  the stock's ways given: `way agents review` (current on a fresh
  project, its words those of `ways-derive`'s table), `way Draft first`
  (not current), `path OWNER/NAME`, `path OWNER/NAME other`, `path
  OWNER/NAME @revision 1` where 1 is not the newest, `charter 4` (the
  reviewer's card), `charter 99`, a waiting proposal of a rule 1 with an
  approval by role 0 (its `after` holds "you approve" and no "review"),
  an accepted one, an unknown way, a malformed address; a member's read
  answered as an owner's; a party with no role in the project refused.
- `temper/docs/design/client/README.status.md`: nothing (the views'
  entry is `views-draw`'s).

## Inputs

- `temper/docs/plan/ui-ask-m3a/notes/common.md`; `notes/contracts.md`,
  sections 1, 2 and 4 (binding).
- `temper/docs/design/client/README.md`, 5.16 and 4.6;
  `temper/docs/design/domain/forge.md`, 8.13;
  `temper/docs/design/draft/ask-temper.md`, 6.2; its status's *Design
  departs* (`@revision`, decided proposals).
- `route/see.rs` and `route/see/proposal.rs` (the proposal read and its
  answer), `route/kinds.rs` (`ForgeRead` and the revision read's
  metadata), `route/ways.rs` (task `ways-read`).

## Owns and may change

- **Owns:** the view's parser and answers under `route/ways/`; the
  `ForgeRead::View` arm of `route/kinds.rs`; `inputs`' new parameters in
  `route/ways.rs`; the table above.
- **Options, pre-approved:** if the proposal's read can't be answered in
  the read's own flight, add a flight kind beside the revision read's and
  say so. If a decided proposal's review key no longer gives items, its
  answer is the decided line above.

## Steps

1. The parser and its words; the way, path and charter answers.
2. The proposal's flight and answer; `@revision`'s check against the
   newest revision.
3. The table; break a rule once (answer a decided proposal as waiting) to
   see its row fail, and restore.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain -p
   temper-engine-domain-world --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-engine-domain --lib` and `-p
   temper-engine-domain-world --test suite -E 'test(/ways/)'`. Commit and
   push.

## Report

As the implementer's text asks.

## Answers

2026-11-08. *`NamedAction::ReadProposal` needs a run's claim and the party
read refuses a member a task's settings proposal: add a project-scoped
proposal read to jig, or make proposal views owners' only?* Add the read to
jig; the member acceptance stands. A project's reader seeing a proposal of
settings that touches the project is the design's
(`jig/docs/design/domain/engine.md`, 17.6, *It belongs to the projects its
items touch*: "project-filtered reads show it in each of its projects"), and
`reads::settings::revision_metadata` is the precedent for an application's
member-scoped read. The missing mechanism is jig's, so it goes there first,
in this task (its files widened to `jig-domain-core`'s `reads.rs` and
`reads/settings.rs`, and `temper-engine-domain`'s `domain.rs` for the
flight's map). The view does not go through `NamedAction::ReadProposal`:
it calls the new read, as `revision_begin` calls `revision_metadata`.

- In `jig/crates/jig-domain-core/src/reads/settings.rs`, beside
  `revision_metadata`:

  ```rust
  /// A project's member reads a proposal of settings that touches the project, for the application's own reads
  /// (jig/docs/design/domain/engine.md, 17.6). Other scopes' items and core values stay as the run's read gives them.
  pub fn project_proposal(
      core: &mut Core,
      env: &Env<Limits>,
      reply: ReplyTo,
      sign_in: u64,
      project: u32,
      proposer: people::Proposer,
      number: u64,
  ) -> Queue<Request>
  ```

  Admitted as `metadata_allowed` admits (signed in, an open project other
  than the deployment, a role in it), and checked again when a load
  returns, as `metadata_loaded` does. The proposal must touch the project:
  a task's, its task's project; a party's, `project` among its `projects`;
  otherwise `None(Unknown)`, never "not allowed", so a non-member learns
  nothing. Waiting, it is built as the run's read builds it
  (`documents::run_proposal`, no party, so no owner check) and answered
  `Answer::Document(Proposal)`, its settings review and key as there;
  decided, withdrawn or replaced, loaded as `run_proposal` loads it (a new
  `Destination`, `reads.rs`, and `proposal_from_store` with `project`)
  and answered `Answer::Ended`; none, `None(Unknown)`; signed out,
  `None(SignedOut)`; no load room, `None(Busy)`. Its answer is terminal,
  as `revision_metadata`'s is. Tests in that module beside the metadata
  ones: a member reads a task's waiting settings proposal of its project;
  an owner's read equals it; a party with no role in the project, another
  project's proposal and a signed-out sign-in each get none; the
  membership is rechecked when a load returns.
- In temper, the pre-approved option: a flight beside `revision_reads`
  (`domain.rs`), answered in `route/kinds.rs` as `revision_answer` answers
  a revision read, the items of temper's kinds then read through
  `Domain::proposal_settings` of the review's key and those of other
  projects left out. A proposal with no review key or whose items are
  gone is the decided line, as the options already say.


2026-11-08. *A decided party proposal (`pN/K`) no longer retains the
projects its items touched: answer it `None(Unknown)`, or widen jig's
stored proposal history to keep them?* `None(Unknown)`; don't widen.
`project_proposal` admits a party proposal only where it can establish
that `project` is among its `projects`: waiting, from
`settings_proposals`; decided, withdrawn or replaced, it can't, so it
answers `None(Unknown)` without a load, as for a proposal of another
project (never "not allowed"). Don't reuse `proposal_row_visible`'s
decided party arm for this read. Its doc comment says so in one line.
temper's view then answers a decided `pN/K` `no proposal pN/K.`; a
waiting `pN/K` and waiting or decided `tN/K` stay as specified, and the
table's decided row stays a `tN/K`. Add a row to the jig tests: a
decided party proposal touching the project gets none. Retaining a
party proposal's projects is a stored-format change for a later plan;
name it under `## Left`.
