+++
id = "all-home-surface"
part = "across"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-domain/src/address.rs", "temper/crates/temper-client-domain/src/pages/all_home.rs", "temper/crates/temper-client-domain/src/pages/parts.rs", "temper/crates/temper-client-domain/src/tests/address.rs", "temper/crates/temper-client-domain/src/tests/pages/all_home.rs", "temper/crates/temper-client-domain/src/tests/pages/parts.rs"]
effort = "high"
minutes = 100
+++

## Goal

Give Home on All projects its address, `Address::AllHome` at `/home`,
and the domain's surface the view, the terminal and the later reads
build on: `pages::all_home`'s facts and its pure intro and brief
composition, and `pages::parts`, the pure rules of what Home and Needs
you draw. No page reads anything new yet: `all_home::facts` names the
person's projects with every status unknown; the next task opens the
reads.

Acceptance:

- `/home` and `/home/` parse to `AllHome`, a query on it is dropped, and
  `print(&Address::AllHome)` is `/home`, as rows of the address table.
- `all_home::intro`, `all_home::brief`, `all_home::first_sentence`,
  `parts::home` and `parts::needs` hold every rule of
  `temper/docs/plan/ui-p2-home/notes/contracts.md`, sections 2 and 3, as
  rows of table tests over hand-made `Facts`.
- Every crate that matches on `Address` compiles with the new arm, and
  the focused tests of `temper-client-domain`, `temper-client-view`,
  `temper-client-tui`, `temper-client-shell-tui`,
  `temper-client-shell-terminal`, `temper-client-gallery` and
  `temper-client-world` pass.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *Home on All projects*
  (its intro), *Home* (*Home on any project*, the brief's rule) and *A
  page's parts*.
- `temper/docs/design/client/browser.md`, 2.3, the `AllHome` row.
- `temper/docs/plan/ui-p2-home/notes/contracts.md`, sections 1 to 3:
  binding names, types and rules.
- `temper/docs/plan/ui-p2-home/notes/common.md`.

## Owns and may change

- Owns the files above: `address.rs`'s `Address` and its codec,
  `pages/all_home.rs` and `pages/parts.rs` (both new), and their tests
  (`tests/pages/all_home.rs`, `tests/pages/parts.rs`, new; rows added to
  `tests/address.rs`).
- `temper-client-domain/src/pages.rs`: the two `pub mod` lines; the
  arms the compiler asks for (`AllHome` beside `Inbox` wherever the frame's
  pages are listed); no page opening for `AllHome` yet.
- `temper-client-domain/src/pages/home.rs`: `impl Facts { pub fn
  parts_facts(&self) -> parts::HomeFacts }` alone (`across` false,
  `projects` from nothing, 0; `brief` from `Facts::brief`; `open` from
  `Needs::Count`; `showing` from `away`'s; `to_begin` from `to_begin()`).
- `src/tests/pages.rs`: the two `mod` lines.
- Any crate's compiler-forced arm for `Address::AllHome`, edited only for
  that: `temper-client-domain` (`route.rs`, `pages/*.rs`, `forge*.rs`,
  `sign_in_tab.rs`, tests), `temper-client-view` (`root.rs`, `frame.rs`,
  `pages/*.rs`, `shared/refusal.rs`, `forge/*.rs`, `choice.rs`, tests),
  `temper-client-tui`, `temper-client-shell-tui`,
  `temper-client-shell-terminal` tests, `temper/tools/gallery/src/samples.rs`,
  `temper/tests/client`, `temper/tests/one-process`. Where an arm decides
  what a page is, `AllHome` reads as `Inbox` does (a project page in the
  open project's frame), with the rail's "Home" current on it.
- `temper-client-view/src/root.rs`: one `else if` for `AllHome` drawing
  an empty `Role::Main` styled `look::MAIN`, so that the tree holds its one
  main; the page itself is a later task's.
- Options, each pre-approved: if a match decides where a page's "back" or
  sign-in lands, `AllHome` lands as `Inbox` does, and you say so; if
  `HomeFacts::projects` for a project's Home is needed by a caller, leave it
  0 (only `across` reads it); if the address table test lists every
  variant, add `AllHome`'s row there.

## Steps

1. `address.rs`: add `AllHome` after `Home { project }`; print `/home`;
   in `parse`, `(b"home", b"", b"")` gives `AllHome`. Rows in
   `tests/address.rs`: `/home`, `/home/`, `/home?x=1` (printed `/home`),
   and `/home/1` missing.
2. `pages/all_home.rs`, as the contracts' section 2 has it. `facts(root,
   wall)`: `me`'s projects, the first `ACROSS`, each `Project { project,
   name, status: Status::Unknown }`; `needs`: `Needs::Count` over
   `root.pages().needs().needs(inbox, root.forge().approvals())` when the
   open inbox is unfiltered and known (its `cut` as Home's), else
   `Needs::Unknown`; `decided` empty, `showing` 0, `away` None.
   `intro(facts)`:
   - `last_looked`: the newest `previous` of the projects whose status is
     `Shown`;
   - `landed`: over the projects shown, N the sum of `landed` less `seen`
     (saturating), P those with it above 0: `Across` for P two or more,
     `In` for one, else `Last { at }` with the newest `last_landed` of a
     project whose `landed` is above 0, else `Nothing`;
   - `needs`: `facts.needs`; `away`: `facts.away`; `brief`: `brief(facts)`.
   `brief(facts)`: each shown project's brief new by Home's rule (move
   `home::Facts::brief`'s test into a `pub(crate) fn new_brief(document:
   &w::ProjectStatus) -> Option<&w::StatusBrief>` in `home.rs` and call it
   from both, if that keeps `home.rs`'s change to one function; else
   repeat the rule and say so), newest `at` first, at most four, each
   `BriefLine { project, name, sentence: first_sentence(lede) }`; `at` the
   newest's. `first_sentence`: bytes up to and with the first `.`, `!` or
   `?` that is the last byte or followed by a space; else the whole.
3. `pages/parts.rs`, as the contracts' section 3: the structs, the two
   `const fn` rules, and `needs_facts(root, wall)`: `known` from the open
   inbox's `known()`, `open` from the merged open cards, `showing` 0 (the
   reads are a later task's).
4. `impl all_home::Facts { pub fn parts_facts(&self) -> parts::HomeFacts }`
   (`across` true, `projects` the rows' count, `brief` from `brief(self)`,
   `open` from `needs`, `showing`, `to_begin` false), and `home::Facts`'s.
5. Tests, tables, one row a case:
   - `tests/pages/all_home.rs`: `intro` over hand-made `Facts` (a status
     built with the wire's `ProjectStatus::new`, as `tests/status.rs`
     builds them): nothing landed; landed on one project (`In`); on three
     (`Across`, P 3); none since the look with a `last_landed` (`Last`,
     the newest); `last_looked` the newest `previous`, a project unknown
     ignored; `brief` with five new briefs keeps the four newest, newest
     first; an old brief (its `landed` at `seen`, `at` before `previous`)
     left out; `first_sentence` rows: `"One. Two."` gives `"One."`,
     `"v1.2 is out. Then"` gives `"v1.2 is out."`, `"No stop"` itself,
     `"Why? Because."` gives `"Why?"`, `""` empty.
   - `tests/pages/parts.rs`: each rule of the contracts' table, a row for
     its holding and one for its failing, To begin hiding all of Home's
     but Projects.
   For each rule you pin, break it once by hand, see its row fail, and
   restore (the implementer's text).
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-domain -p temper-client-view -p
   temper-client-tui -p temper-client-shell-tui -p
   temper-client-shell-terminal -p temper-client-gallery -p
   temper-client-world -p temper-one-process-world --all-targets -- -D
   warnings`; `heavy cargo nextest
   run -p temper-client-domain`, then the focused tests of the other
   packages named in the acceptance. Commit and push.

## Report

What you changed, each forced arm's file in one line, the calls you made
inside the sketches (where `new_brief` went), the tests and their counts,
the rule each new row bites on, and your implementation and check minutes
apart.
