+++
id = "all-home-reads"
part = "across"
kind = "feature"
needs = ["all-home-surface"]
files = ["temper/crates/temper-client-domain/src/pages/all_home.rs", "temper/crates/temper-client-domain/src/pages/parts.rs", "temper/crates/temper-client-domain/src/tests/pages/all_home.rs", "temper/crates/temper-client-domain/src/tests/pages/parts.rs"]
effort = "high"
minutes = 170
+++

## Goal

Make Home on All projects and Needs you read across projects: on
`AllHome`, each project's `status` and `decided`, its look, its
decisions' tasks, the unfiltered inbox; on Needs you, each project's
decisions. `all_home::facts` and `parts::needs_facts` then say what the
reads hold. Add the Ask temper line's acts, and accept Overturn off a
project's Home.

Acceptance, as rows of the domain's tables (`src/tests/pages/all_home.rs`,
`src/tests/pages/parts.rs`, `src/tests/away.rs`):

- Going to `AllHome` with `me` listing three projects watches `status` and
  `decided` for each, once, and the unfiltered inbox; leaving unwatches
  every one. With `me` delivered after the page opened, the watches open
  then; a project `me` adds later is watched once; more than `ACROSS`
  projects watch the first `ACROSS`.
- Each project's `Looked` goes once, after its facts settle as Home's
  look does; a project's status counts once its look is answered and its
  document delivered; a watch refused or ended leaves that project's
  status `Unknown` and the others shown.
- `facts`' `decided` holds the rows showing across projects, at most 7,
  the newest kept, oldest first, `showing` all of them, each row's task
  read (`Read::Task`) as Home reads its rows, overturned rows marked;
  `away` the newest spell whose `off` is after the newest `previous`.
- `needs`: the open cards of the unfiltered inbox with every `approvals`
  row, `cut` as Home's. A card's choice, Accept, Reject, Release and Leave
  held answer on `AllHome` as on a project's Home (`needs::Act` through
  its held send, the confirmation they have).
- On an `Inbox` address, each project's `decided` is watched (filtered by
  the address's project where it has one), and `parts::needs_facts`'
  `showing` counts its rows; a `pub fn decided(root: &Domain, wall: Wall)
  -> Box<[all_home::Decided]>` in `all_home.rs` gives Needs you its rows,
  as `facts` gives Home's.
- `away::Act::Overturn { project, number }` overturns a live row shown on
  `AllHome` and on an `Inbox` address as on `Home { project }`.
- `Act::AskLine` and `Act::AskWith` as the contracts' section 4: words
  held and read back; `AskWith` from a project's Home and from `AllHome`
  opens `Ask { project }` (the open project on `AllHome`, else `me`'s
  first) with the words in its composer, unsent; empty words do nothing.
- The focused tests of `temper-client-domain`, `temper-client-view`,
  `temper-client-tui` and `temper-client-world` pass.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *Home on All projects*
  (*Its reads*, *The intro*, *Decided for you*), *Home* (*Home on any
  project*: its look), *Away* (*Home, while a decision shows*, *Overturn*),
  *Needs you* (*Decided for you*) and *A page's parts*.
- `temper/docs/plan/ui-p2-home/notes/contracts.md`, sections 2 to 4.
- `temper/docs/plan/ui-p2-home/notes/common.md`.

## Owns and may change

- Owns `pages/all_home.rs` and `pages/parts.rs` (their bodies, as
  `all-home-surface` left them) and their tests.
- `pages/home.rs`: reuse `Home` per project rather than copy its reads:
  `Home::new` opens `status` and `decided`; add a constructor that opens
  `decided` alone (for Needs you), and make `pub(super)` what `all_home`
  calls (`shown_status`, `away`, `read_tasks`, the look's state). No change
  to what a project's Home draws or reads.
- `pages.rs`: a field holding `AllHome`'s per-project reads and one for
  Needs you's decisions; opening them in the address's change as `Home`'s
  are opened, leaving them where `home.leave(out)` is called, routing
  their `Told` as `home`'s is routed; `look` for each project as
  `home::look` does for one; `Act::AskLine`, `Act::AskWith` and
  `Pages::ask_line`; the held words written into the Ask page's composer
  (`chats.write`) once `Ask { project }` has opened, then dropped.
- `pages/away.rs`: `overturn`'s address check and its row lookup widened
  to `AllHome` and `Inbox` (the row found among that page's decisions);
  its toasts unchanged.
- `pages/needs.rs`: nothing but the compiler's arms.
- `temper-client-view`, `temper-client-tui`, `temper/tools/gallery`,
  `temper/tests/client`: the compiler's arms for the two new `Act`
  variants alone.
- Options, each pre-approved: if a per-project `Home` holds state no page
  across projects needs (its add sheet, its skip, its copies), leave it
  idle rather than split `Home`; if the handles a page may take are bounded
  (`pages.next`), take two a project from it as `Home::new` does; if
  `me`'s projects change while `AllHome` is open, open the new ones and
  leave the ones gone; if `chats.write` cannot run before the Ask page's
  chats are made, hold the words until the first event after it opens, and
  say so.

## Steps

1. Read `pages/home.rs` (`Home::new`, `told`, `decided_told`,
   `shown_status`, `away`, `read_tasks`, `look`), `pages.rs`'s address
   change (where `Address::Home` opens `home`, `chats` and `inbox`) and its
   `told` routing.
2. `all_home.rs`: `pub(crate) struct AllHome` (the per-project `Home`s in
   `me`'s order, at most `ACROSS`), its `new`, `leave`, `told`; `facts`
   filled from them (status per project; decisions merged across
   projects, newest seven, oldest first; `showing`; `away` from
   `root.forge().away(project)`); `decided(root, wall)` for Needs you.
3. `parts.rs`: `needs_facts`' `showing` from Needs you's decisions.
4. `pages.rs`, `away.rs`: the wiring above; the Ask line's acts.
5. Tests, one row a case of the acceptance, beside `tests/pages/home.rs`'s
   fixtures (its status and decision documents); for each rule you pin,
   break it once and see its row fail.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-domain -p temper-client-view -p
   temper-client-tui -p temper-client-gallery -p temper-client-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-client-domain`, then the focused tests of the others. Commit and
   push.

## Report

What you changed, how `AllHome` holds its per-project reads and what of
`Home` it reuses, the handles a page takes, the calls you made inside the
sketches, the tests and their counts, each new row's rule and how it
failed when broken, and your implementation and check minutes apart.
