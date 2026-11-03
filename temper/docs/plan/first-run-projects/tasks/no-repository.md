+++
id = "no-repository"
part = "no-repository"
kind = "feature"
needs = ["welcome-ready"]
files = [
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/forge/form.rs",
  "temper/crates/temper-client-domain/src/forge/listed.rs",
  "temper/crates/temper-client-domain/src/tests/form.rs",
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/chats.rs",
  "temper/crates/temper-client-view/src/tests/forge_page.rs",
  "temper/tests/client/tests/suite/adoption.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "medium"
minutes = 60
+++

# A project with no repository says so, and its first adoption is one choice away

## Goal

A project with no repository is usable and its chats page says so where its repository would
show: "No repository yet", with "Adopt a repository" to the adoption form into this project
(K4). The form, opened while a set up's first repository is made and adopted nowhere, opens
filled for it, so that adopting `work/hello` is one press of "Adopt"
(`temper/docs/design/client/README.md`, 3.5 "A project with no repository", 4.4 "From a project
with no repository").

Acceptance:

- On the chats page of a project no row of the delivered `forges` watch names, before the
  composer: a node holding "No repository yet" and the link "Adopt a repository", bound to
  `Act::Go { address: Address::Adopt { project } }` (or the view's existing link to it). Nothing
  of it while the watch is refused, ended or not delivered, for a project with a repository, or
  for the playground's project while the playground is read.
- `Event::Went { Adopt { project } }` opens the form filled from the lowest-numbered real
  forge's row whose `first` is `Made` and whose organization and name no row's `repositories`
  names: "Forge number" that forge, "Owner" the organization, "Name" the name, "Landing branch"
  `main`; other fields at their defaults. With none, `form(project)` as now. If the rows arrive
  after the form opened and no field has been written, it is filled then.
- The tests below pass; the status entries below are gone.

## Inputs

- `temper/docs/plan/first-run-projects/notes/common.md`; `notes/contracts.md`, K4.
- `temper/docs/design/client/README.md`, 3.5: "A project with no repository"; 4.4: the table's
  defaults and "From a project with no repository"; 4.6: the row's `first` and `repositories`.
- `temper/docs/design/protocol/setup.md`, 6.7 (the repository made on `main`, unadopted).

## Owns and may change

- `temper-client-domain`:
  - `forge/listed.rs` (default; or `forge/connect.rs`): two helpers on the rows, so that domain
    and view share the rule:

    ```rust
    /// K4: Some(true) once the rows are delivered and none names `project`.
    pub fn without_repository(forges: &forges::State, playground: Option<&Playground>, project: u32) -> Option<bool>
    /// 4.4: the first repository made and adopted nowhere, on the lowest-numbered real forge.
    pub fn unadopted_first(forges: &forges::State) -> Option<(u16, &[u8], &[u8])>
    ```

    `without_repository` is `None` (say nothing) for `Unknown` or `Ended` rows and for the
    playground's project while the playground is read.
  - `forge/form.rs`: a way to fill a `Form` from `unadopted_first` (default `fill_first(form,
    forge, org, name)`, counting each field it writes as the domain's write, so the view
    rewrites the values). `reset` after an adoption keeps `form(project)`'s plain defaults.
  - `forge.rs`: `Event::Went` fills the new form; the forges watch's delivery fills a form open
    and unwritten (every field's `written` 0).
- `temper-client-view`:
  - `pages/chats.rs` `main`: in the non-playground branch, after the page bar and before "Start a
    chat", while `without_repository` is `Some(true)` for `root.pages().project()`: a `Slot`
    holding a `Paragraph` "No repository yet" and a link "Adopt a repository" (`Word::ForgeAdopt`)
    to `Address::Adopt { project }`. No new style: the look plan places and styles it
    (`notes/common.md`); use the existing `look::CONTEXT` if the slot needs one, nothing else.
  - `words.rs`: `Word::NoRepositoryYet`, "No repository yet", at the next free number.
  - **Overlaps:** the look plan `look-welcome-chats` moves the chats page into a left pane, and
    this plan's step 4 offers "Tell temper how you work" above the composer: both edit
    `pages/chats.rs` `main`. Add the one node and nothing else there.
- Tests:
  - `temper-client-domain/src/tests/form.rs`: the form opened filled for an unadopted made first
    repository on forge 2 (a playground having taken 1), its four fields as above; not filled
    once a row's `repositories` names it, for a failed or making `first`, or for the playground's
    fake row; filled on a late delivery while unwritten and not after a write; reset after an
    adoption to the plain defaults.
  - `temper-client-view/src/tests/chats.rs`: the node and its link for a project with no
    repository; none with a repository, none with the watch refused, none for the playground's
    project while it is read.
  - `temper-client-view/src/tests/forge_page.rs`: the page draws the filled values.
  - `temper/tests/client/tests/suite/adoption.rs` (package `temper-client-world`): one story,
    an owned Forgejo set up (as the existing stories make one), Ready's "Open Default", the
    chats page's "Adopt a repository", "Adopt" pressed with nothing typed, adopted
    `work/hello`, and then the chats page with no "No repository yet".
  - Any other test that fails because the chats page draws the node (a tree compared whole, a
    count of paragraphs): its expectation alone, listed in the report.
- Status, as `notes/common.md` says: `temper/docs/design/client/README.status.md`, **Not yet**:
  remove the lines from `3.5, A project with no repository: the chats page says nothing of it`
  through `with owner, name and landing empty).`, leaving `4.4: the picker, …`.

## Steps

1. The helpers and the form; `heavy cargo nextest run -p temper-client-domain --lib -E
   'test(form) | test(listed)'`.
2. The chats page's node; `heavy cargo nextest run -p temper-client-view --lib -E 'test(chats) |
   test(forge_page)'`.
3. The story; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(adoption)'`.
   The journeys still print: `heavy cargo nextest run -p temper-client-journeys` (the
   playground's project is excluded, so `welcome-playground` gains no node; if it does, the
   rule is wrong: fix the rule, never the print).
4. The status edit; the citations check.
5. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-client-domain -p
   temper-client-view -p temper-client-world --all-targets -- -D warnings`; the focused tests of
   `temper-client-domain`, `temper-client-view` and their direct dependents
   (`temper-client-tools`, `temper-client-shell-web`, `temper-client-shell-terminal`,
   `temper-client-world`). Commit the code and tests, then the status text; push.

## Report

As the implementer's text says, with each test whose expectation moved.
