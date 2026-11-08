+++
id = "deployment"
part = "deployment"
kind = "feature"
needs = ["addresses"]
files = ["temper/crates/temper-client-domain/src/pages/members.rs", "temper/crates/temper-client-domain/src/tests/pages/members.rs", "temper/crates/temper-client-view/src/pages/settings.rs", "temper/crates/temper-client-view/src/pages/deployment.rs", "temper/crates/temper-client-view/src/pages/builders.rs", "temper/crates/temper-client-view/src/tests/deployment.rs", "temper/crates/temper-client-view/src/tests/settings.rs", "temper/crates/temper-client-view/src/tests/models.rs", "temper/crates/temper-client-view/src/tests/builders.rs"]
effort = "high"
minutes = 240
+++

# Deployment: Accounts, Forges, Models, Workers, Members and Limits

## Goal

Split Settings' frame as `temper/docs/design/client/README.md`, 3.5, *Settings and
Deployment*, has it: How work runs keeps the project's two tabs, How work runs and
Repositories, under the kicker "PROJECT · how work runs"; Deployment holds Accounts,
Forges, Models, Workers (Builders on the page), Members and Limits under the kicker
"Deployment". Today's pages keep their addresses and acts. Members reads jig's
`Read::Project` for each of the person's projects; Limits draws the `deployment`
document's capacities. Both shells.

Acceptance:

1. On `Settings` and `Repositories` the frame's kicker is "PROJECT · how work runs",
   its tab bar a navigation named "How work runs" with "How work runs" and
   "Repositories". On `Accounts`, `Forges`, `Models`, `Builders`, `Members` and
   `Limits` with a project open the kicker is "Deployment", its tab bar a navigation
   named "Deployment" with "Accounts", "Forges", "Models", "Workers", "Members" and
   "Limits", to 3.5's addresses, the current one current, Forges' and Accounts'
   marks kept. Each tab's heading and lede are 3.5's table's. With no project,
   Accounts, Forges, Members and Limits draw alone, as Accounts and Forges do today.
2. The Builders tab says "Workers" and "worker" wherever it said "Builders" and
   "builder", as 3.5, *Workers*, words it; the code keeps its names.
3. `Members`: the domain reads `Read::Project` for each project `me` lists, one at
   a time, in `me`'s order, again when `me`'s projects change, keeping each
   project's outcome (contracts.md, C5); the page draws "The deployment"'s line and a
   section per project with its members, each its name or login, its kind and its
   role, a cut document saying so, an unread project its title alone, a refused one
   "Read by the project’s members.", and the `.facts` line.
4. `Limits`: the `deployment` watch opens on `Limits` as on `Builders`, for a person
   with a role in the deployment; the page draws "Capacities" and "By project" as
   3.5, *Limits*, words them, "Limits are seen by the deployment’s owners." with no
   role, the title alone until read.
5. The flows' paths are "Deployment / Forges / Connect a forge" and "Deployment /
   Accounts / Connect a model" (3.5, *The flows' paths*); 5.13's Changes switch line
   is "The landing queue, as Work's filter.".
6. The focused tests of `temper-client-domain`, `temper-client-view` and
   `temper-client-gallery` pass, clippy clean; the README.status.md entry "**Not
   yet:** 3.5, *Settings and Deployment*" is gone.

## Inputs

- `temper/docs/plan/ui-p1-rail/notes/common.md`; `contracts.md`, C1, C5 and C6.
- `temper/docs/design/client/README.md`, 3.5: the page table's rows for `Members`,
  `Limits`, `Models`, `Builders`, `Forges` and `Accounts`; *Settings and Deployment*,
  whole; 5.13, *What the project shows*, its table's Changes row.
- `temper/docs/design/client/browser.md`, 2.5, *Deployment's frame*.
- jig's `jig/docs/design/client/domain/contract.md`, 5.1 (the project document) and
  6 (one-shot reads); `jig/docs/design/domain/people.md`, sections 3 and 4 (roles).
  The types: `jig_client::contract::{Read::Project, Project, ProjectMember,
  HeldRole, PartyKind, Deployment, Capacity, ProjectCapacity}`.
- The code: `temper-client-view`'s `pages/settings.rs` (`Tab`, `open`,
  `open_alone`, `flow_path`), `pages/builders.rs`, `pages/models.rs`,
  `pages/accounts.rs` and `forge/forges.rs` (how they choose `open` or
  `open_alone`); `temper-client-domain`'s `pages/builders.rs` (the deployment watch
  and its role rule) and `pages.rs`'s `builders_open`; a one-shot read issued and
  answered in the domain, as `pages/goals.rs` reads a report.

## Owns and may change

Owns: `temper-client-domain`'s new `pages/members.rs` and
`src/tests/pages/members.rs`; `temper-client-view`'s `pages/settings.rs` (its
frame, tabs, paths and the shows section's words), the new `pages/deployment.rs`
(Members' and Limits' pages), `pages/builders.rs`, and the tests
`src/tests/deployment.rs` (new), `tests/settings.rs`, `tests/models.rs` and
`tests/builders.rs`.

Reader and neighbour files, and what may change there:

- `temper-client-domain`'s `pages.rs`: `pub mod members;`, a `Pages` field and
  accessor for it, the `went` block opening it on `Members` and leaving it, the
  `builders_open` arm true on `Limits`, and the answers' routing to it, beside
  `builders`' own. Nothing else in `pages.rs`: `addresses` owns its Work blocks.
- `temper-client-view`'s `root.rs`: `else if let` branches for `Members` and
  `Limits`, after `Builders`'; `pages.rs`: `pub(crate) mod deployment;`;
  `src/tests.rs`: `mod deployment;`; `pages/accounts.rs`, `pages/models.rs` and
  `forge/forges.rs`: their call of `settings::open` with the right tab, nothing
  else.
- `words.rs`: Deployment's words, keys 4750 to 4799 (C6), beside the Builders
  words; the Builders tab's phrases changed in place to "Workers" (their keys
  stay); `SettingsChangesLine`'s phrase.
- `look.rs`: slots 672 to 675 (C6) if Members or Limits need a style
  Repositories' lines lack.
- `temper.css`: under `/* ==== ui: settings ==== */`.
- Tests that assert the Settings frame or its tab words, their frame lines alone:
  `tests/accounts.rs` (`account_management_keeps_both_account_lists`,
  `the_tab_draws_your_accounts_with_its_count_and_act_in_its_head`, and any test
  finding the kicker or the tabs), `tests/forges.rs` (the kicker and tabs),
  `tests/settings.rs`'s `each_flow_draws_its_path_in_the_browser_and_none_in_the_terminal`'s
  path lines alone (its rail-current lines are `rail`'s, beside you); the
  gallery's rows for `settings*`, `repositories`, `forges-in-settings`,
  `accounts-in-settings`, `builders*` and `models*` where they read the kicker,
  the tabs or "Builders"; `temper-client-domain`'s `tests/pages/builders.rs` if
  Limits shares its watch's rule.

Not yours, left stale and listed under `## Left`: the terminal goldens
(`regenerate`); the client world's `builders.rs` and `models.rs` suites and the
browser stories (`readers`). Not yours at all: `frame.rs` (`rail`, beside you, draws
"Deployment" in the rail) and Work (`work-page`).

Pre-approved options:

- **If `Tab` is better split** into the two frames' enums: do so, and say so.
- **If `Read::Project`'s answer is cut** (`cut`): draw the members it holds and
  the cut line as `Accounts` does.
- **If a member's `login` and `name` are both empty:** "Party N", N its party.
- **If the store's mark is zero:** "4.1 MB" alone, no " of ".
- **If Limits' watch must differ from Builders' rule:** keep Builders' rule (a role
  in the deployment), and say so in the report.

## Steps

1. **Domain, Members:** `pages/members.rs` (C5): the reads, one out at a time, in
   `me`'s order; again on a change of `me`'s projects; each outcome kept. `went`
   opens it on `Members` and releases it on leaving; `builders_open` on `Limits`.
   Tests, `tests/pages/members.rs`: one table: reads one at a time and in order;
   again when `me` gains a project; a refusal kept; leaving releases; Limits opens
   the deployment watch for a role and not without. Break one rule by hand (two
   reads out at once) and see its row fail.
2. **View, the frames:** `settings::open` draws How work runs' or Deployment's
   kicker and tab bar by the tab; the table's headings and ledes; the paths.
3. **View, Workers:** `pages/builders.rs`'s words, as acceptance 2.
4. **View, Members and Limits** in `pages/deployment.rs`, as 3.5 words them, a line
   as Repositories' lines are; `root.rs`'s branches.
5. **Tests,** `tests/deployment.rs`: a table over Members' states (read, cut,
   unread, refused, no deployment role, owner) and Limits' (read with waiting
   attempts, two projects, no role, unread, a zero mark); `tests/settings.rs`'s
   `every_tab_draws_its_head_and_tabs_in_order` over both frames;
   `tests/models.rs`'s tab order; `tests/builders.rs`'s words, `the_page_says_builder`
   now holding "worker" and neither "builder" nor "host". Break two rules by hand
   (Members' tab current on Limits; Limits drawn without a role) and see their rows
   fail; restore.
6. **Readers** as Owns says, then **check** as `notes/common.md` says, for
   `temper-client-domain`, `temper-client-view` and `temper-client-gallery`, and
   `heavy cargo check --tests` for `temper-client-shell-terminal`,
   `temper-client-world` and `temper-client-browser-world`. Remove the status entry.
   Commit and push: the message says the deployment's tabs leave a project's
   Settings for Deployment's frame.

## Report

Follow the implementer's text. Also give the rules you broke and the failures you
saw; the words and keys added and changed; any style slot taken; each test's CPU
time; and, under `## Left`, each stale golden and story by file and line.
