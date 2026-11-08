+++
id = "rail-parts"
part = "rail"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-domain/src/pages/rail.rs", "temper/crates/temper-client-domain/src/tests/pages/rail.rs"]
effort = "high"
minutes = 70
+++

# The rail's parts, one pure function

## Goal

Decide what the rail and the phone's tab bar show in one pure function of the
client's domain, which both shells will draw from: `pages::rail::parts(&Facts) ->
Parts`, with `rail::facts` gathering its plain facts from the domain once a build,
and `rail::across`, whether the page is one across projects. Nothing draws from
it yet; the task `rail` does.

Acceptance:

1. `temper_client_domain::pages::rail` holds `Facts`, `Parts`, `New`, `parts`,
   `facts` and `across` exactly as `contracts.md`'s C3 gives them, the module's
   doc comment citing `temper/docs/design/client/README.md, 5.3`.
2. `parts` follows 5.3's table *The rail's parts*: a table test holds a row for
   each rule, its part shown and hidden, and the To begin rows.
3. `facts` gathers each fact from the domain as C3's comments say, `members` and
   `standing_roles` `None`; a test holds it on a domain at a project with two
   projects in `me`, template 5 listed and drafts offered, and on one with none of
   them.
4. `across` is true on `Inbox` with no project and false on `Inbox` with a
   project and on a project's page.
5. `temper-client-domain`'s focused tests pass, clippy clean.

## Inputs

- `temper/docs/plan/ui-p1-rail/notes/common.md`, and `contracts.md`, C3.
- `temper/docs/design/client/README.md`, 5.3: *The rail* (its first three items),
  *While To begin shows*, *The rail's parts* and its table, *All projects*.
- The code the facts come from, read, not changed: `pages/needs.rs`,
  `Session::count`; `pages.rs`, `drafts_offered`; `pages/home.rs`, `beginning`;
  `domain.rs`, `away_shown`; `pages/chats.rs`, `ASKING_TEMPLATE`, and how
  `temper-client-view`'s `frame.rs` asks `jig_client::shapes::template` for it;
  `Pages::me`.

## Owns and may change

Owns: the new `temper-client-domain/src/pages/rail.rs` and
`src/tests/pages/rail.rs`.

Neighbour files, one line each: `pages.rs`, `pub mod rail;` in its module list,
in order; `src/tests/pages.rs`, `mod rail;` after `mod project_pages;`.

Nothing else. In particular, no match on `Address` in `rail.rs`: `across` asks
`Inbox` by `if let`, since `addresses`, beside this task, adds variants to
`Address`, and an exhaustive match here would break where the two meet.

Pre-approved options:

- **If `count`'s answer for the open project needs more than `facts`' arguments**
  (the approvals, the client): take them from `root` as `frame.rs`'s `entries`
  does today, and say so.
- **If clippy asks for `#[must_use]` or a `const fn`:** do as it says.

## Steps

1. **The types and `parts`**, in `rail.rs`, as C3. `parts` reads the facts alone:
   `all_projects` while `projects >= 2`; `needs_you`, `work`, `how`
   while not `beginning`; `count` and `cut` copied, `count` kept even at zero (the
   view omits it); `team` while not `beginning` and `members >= 2` or
   `standing_roles >= 1`, each `None` counting as not; `ask` while `asking`;
   `away` while `away`; `new` `Hidden` while `beginning`, else `DraftOrChange` while
   `drafts`, else `Change`.
2. **`facts`** from `root`, with `u32::try_from` and a saturating fallback for the
   projects' count.
3. **`across`:** `if let Address::Inbox { filter } = root.pages().address()`, true
   while `filter.project` is `None`.
4. **Tests**, `tests/pages/rail.rs`: one table test of `parts`, a row per rule
   (each part shown and hidden, Team by members, by standing roles and by neither,
   the two `New` forms, and To begin with every other fact set); one test of
   `facts` on two domains, built with the fixtures `tests/pages/switcher.rs` and
   `tests/pages/project_pages.rs` use for `me`; one table of `across`. Break one
   rule by hand (Team at one member) and see its row fail; restore.
5. **Check** as `notes/common.md` says. Commit and push: the message says that the
   rail's parts are now one function both shells can draw from.

## Report

Follow the implementer's text. Also give the rule you broke and the failure you
saw, and each test's CPU time.
