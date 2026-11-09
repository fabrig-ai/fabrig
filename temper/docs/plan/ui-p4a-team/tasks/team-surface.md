+++
id = "team-surface"
part = "model"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/team.rs",
  "temper/crates/temper-client-domain/src/tests/address.rs",
  "temper/crates/temper-client-domain/src/tests/pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/team.rs",
  "temper/crates/temper-client-domain/src/tests/pages/project_pages.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 100
+++

# Team's address and its model

## Goal

Give the client the page `Team { project }` at `/projects/<project>/team`,
and `pages::team`'s model: its plain facts, `gather` from a project's
document, and `parts`, section 12's one pure function for Team. Nothing is
read or drawn yet: `team-reads` reads, `team-page` draws. This is the
surface the plan's other tasks build on, so it stays small.

Acceptance:

1. **The address** (`notes/contracts.md`, T1): `Address::Team { project:
   u32 }` after `Rule`; `print` gives `/projects/<project>/team`; `parse`
   reads it, as it reads `/projects/<project>/changes`; the doc comment
   listing the paths names it. A project's page: `address_project` gives
   its project, the standing inbox watch is open on it (the `showing`
   match), and choosing another project keeps it (`Act::Project`, about
   `pages.rs` line 925: `Team { project }` becomes `Team { project:
   chosen }`).
2. **The model** (contracts T2), in the new
   `temper-client-domain/src/pages/team.rs`, `pub mod team` in `pages.rs`:
   `OTHERS`, `Waits` (with `total`), `Person`, `Facts`, `Parts`, `Shown`,
   `parts`, `gather`, `shown` and `people`, each as T2 writes it. `parts`
   and `gather` follow T2's rules whole; `shown` answers `Shown::Reading`
   and `people` `None` for every project, each with a doc line saying
   that no read gives Team its facts yet.
3. **Forced arms** of `Address::Team` in every exhaustive `match` of
   temper's tree, each edited for `Team` alone, by contracts T1's rule:
   a project page with no read of its own, current on no rail entry and
   on no Settings tab. The sites, as the survey found them on main
   (`rg -n 'Address::Builders' temper/` finds them): `temper-client-domain`'s
   `address.rs`, `pages.rs` (eight), `route.rs` (two), `sign_in_tab.rs`,
   `forge/trials.rs`, `pages/home.rs`, `pages/terminal.rs`,
   `pages/connect_model.rs` (three), `pages/needs.rs` (two),
   `pages/roles.rs` (two), `pages/switcher.rs`; `temper-client-view`'s
   `choice.rs`, `frame.rs` (two), `root.rs` (two), `pages/terminal.rs`,
   `pages/connect_model.rs`, `pages/roles.rs`, `pages/settings.rs`,
   `shared/refusal.rs`, `pages/terminal/commands.rs`; the TUI's
   `temper-client-tui/src/view.rs`, `view/launchpad.rs` and
   `temper-client-shell-tui/src/host.rs`, each as `Builders`; the tests'
   and gallery's `temper-client-domain/src/tests/pages/project_pages.rs`,
   `temper-client-view/src/tests/fixtures.rs`, `tests/rail.rs` (no entry
   current on Team) and `temper/tools/gallery/src/samples.rs` (two, as
   `Changes`; no sample). Any other site the compiler names takes the same
   rule. `root.rs` draws no body for `Team`: its page dispatch is an `if
   let` chain, and `team-page` adds the branch.
4. **Rows, no new test function where a table takes them:**
   - the codec's table, `tests/address.rs`'s `each_page_round_trips_through_its_path`
     (about line 8): `Team { project: 0 }` `/projects/0/team`, `Team {
     project: 3 }` `/projects/3/team`, `Team { project: u32::MAX }`;
     `a_path_that_is_no_page_is_missing` (about line 126):
     `/projects/3/team/1`; `a_number_not_in_canonical_decimal_is_missing`
     (about line 190): `/projects/03/team`;
     `one_trailing_slash_is_ignored` (about line 113): `/projects/3/team/`;
   - `tests/pages/project_pages.rs`: choosing project 4 on `Team { project:
     3 }` goes to `Team { project: 4 }`, in the table or test that holds
     `Builders`' row;
   - a new `tests/pages/team.rs` (`mod team;` in `tests/pages.rs`), two
     table tests: `gather_lists_each_person_once_the_reader_first` with a
     row each for the reader first; a service left out; a party listed
     twice kept once, the first; an empty name giving the login; the
     reader's waits and another's attached; a pair for an unlisted party
     dropped; a reader who is no member; `cut` copied. And
     `parts_show_what_the_facts_hold`: People shown and hidden, the
     triager's line shown and hidden.
5. **Status:** delete, whole, the **Not yet** entry "3.5, *Team*: no `Team
   { project }` address and no `pages::team`" in
   `temper/docs/design/client/README.status.md`, and "2.3:
   `/projects/<project>/team` reads as no page" in
   `temper/docs/design/client/browser.status.md`. No other design line.

## Inputs

- `temper/docs/plan/ui-p4a-team/notes/common.md`;
  `temper/docs/plan/ui-p4a-team/notes/contracts.md`, T1 and T2.
- `temper/docs/design/client/README.md`, 3.5: the pages' table's
  `Team { project }` row; *Team*, its first paragraph, *People*, *Its
  parts*.
- `temper/docs/design/client/browser.md`, 2.3, the addresses' table.
- jig's `Project` and `ProjectMember`:
  `jig/crates/jig-client-domain/src/contract/project.rs`; `HeldRole` in
  `contract/refusals.rs`; `PartyKind` in `contract/common.rs`.
- Models: `Changes { project }` in `address.rs` and `pages.rs`; the
  table tests of `tests/pages/` (`rail.rs`'s style on ui-p1-rail's trunk is
  the same: `git show origin/ui-p1-rail/integration:temper/crates/temper-client-domain/src/tests/pages/rail.rs`).

## Owns and may change

Owns the files in the header. Reader and neighbour files, and what may
change there: the forced arms of acceptance 3, in each file named there
and any other the compiler names in temper's tree, each for `Team` alone;
`temper-client-domain/src/pages/needs.rs`, `pages/home.rs` (ui-p2-home's)
and `temper-client-view/src/frame.rs` (ui-p1-rail's) take the arm and
nothing else.

Options, each decided:

- **A site where `Changes` reads or is current on something** (a forge
  read, the rail's "Changes", a How work runs read): Team takes the arm
  that reads nothing and is current on nothing, as contracts T1 says.
- **A test that lists every address** but holds no `match` (the shell-web
  host tests, the terminal's): add a row for `Team` only where its table
  lists every address, so that it still does; else leave it.
- **`List`'s capacity in `gather`:** the document's member count; no
  bound of Team's own.

## Steps

1. Read the Inputs. The address and its codec rows (acceptance 1, 4's
   first item); bite: print `team` as `teams`, see the round trip's row
   fail, restore.
2. `pages/team.rs` and its two tests (2, 4's last item); bite: let
   `gather` keep a service, see its row fail, restore.
3. The forced arms (3), compiling with `heavy cargo check -p PKG --tests`
   for `temper-client-domain`, `temper-client-view`, `temper-client-tui`,
   `temper-client-shell-tui`, `temper-client-shell-web`,
   `temper-client-shell-terminal`, `temper-client-gallery` and
   `temper-client-world`.
4. The project switch's row (4); the status entries (5).
5. Runs through `heavy`, counts in the report: `cargo nextest run -p
   temper-client-domain`, `-p temper-client-view`, `-p
   temper-client-shell-web`, `-p temper-client-shell-terminal`, `-p
   temper-client-gallery`; `cargo check -p temper-client-tui --tests`, `-p
   temper-client-shell-tui --tests`, `-p temper-client-world --tests`;
   clippy on every package changed; then as `notes/common.md` says before
   the push. Commit, saying that Team has its address and its model, and
   push.

## Report

The implementer's report: the address and its paths, `pages::team` as
written, every forced arm by file and the arm it took, the rows and the
bites with their failures, the runs' counts, and what is left.
