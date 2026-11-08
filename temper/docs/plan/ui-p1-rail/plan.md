# ui-p1-rail: the rail, Work and Deployment

The request: `inputs/request.md`, milestone P1 of
`temper/docs/design/draft/whole-story.execution.md`: `surfaces.md`, sections 2, 5
and 8. The design this cut wrote, on the trunk with the first push:

- `temper/docs/design/client/README.md`, 3.5: the page table's rows for `Work`,
  `Members` and `Limits`; *Work* (new); *Goals* and *Changes* as Work's filters;
  *Settings and Deployment* (Settings split, Builders shown as Workers, Members
  and Limits new); 5.13's Changes switch line.
- `README.md`, 5.3: *The rail*, *The rail's parts* (new), the tab bar, *Projects
  in the rail*, *All projects* (new), the person's menu's place for "What temper
  remembers about you".
- `temper/docs/design/client/browser.md`: 2.3's paths and *Work's filter is its
  query*; 2.5's *The rail as the whole story draws it*, *Work's look* and
  *Deployment's frame*, and the tab bar's four columns; 7.2's states.
- `temper/docs/design/client/greenfield/README.md`: the whole story's mockups'
  precedence.
- Status: `README.status.md` and `browser.status.md`'s **Not yet** entries;
  `draft/surfaces.status.md`'s lags 2, 5 and 8 closed, 10, 12, 13 and 14 narrowed.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `addresses` | `tasks/addresses.md` | | `Work`, `Members`, `Limits` and their paths; Work's five addresses share the goals', changes' and chats' reads; reserved style slots | 130 |
| `rail` | `tasks/rail-parts.md` | | `pages::rail::parts`, `facts`, `across`: the rail's parts as one pure function | 70 |
| `rail` | `tasks/rail.md` | addresses, rail-parts | the view's rail, new box, foot, switcher with All projects, tab bar, in both shells; `temper.css`'s `ui: rail` | 220 |
| `rail` | `tasks/readers.md` | rail, work-page, deployment | the client world's, browser's, real loop's, one process's and `temper`'s stories walk the new rail | 150 |
| `work` | `tasks/work-list.md` | | `pages::work::place`: what goes where on Work's one list | 110 |
| `work` | `tasks/work-page.md` | addresses, work-list | Work's page, its head and filters, its cards; Goals and Changes under Work's head; kind 7's overrides; `work::facts` | 270 |
| `work` | `tasks/joint-world.md` | rail, work-page, readers | the client's joint world: Work across two projects | 120 |
| `deployment` | `tasks/deployment.md` | addresses | Deployment's frame and tabs, Workers, Members and its reads, Limits; How work runs' two tabs | 240 |
| `checks` | `tasks/fuzz.md` | rail-parts, work-list, work-page | section 12's rules over `rail::parts`, `work::place` and `work::shown`, random settings and moves | 90 |
| `checks` | `tasks/gallery.md` | rail, work-page, deployment | the gallery's fifteen new states | 150 |
| `checks` | `tasks/regenerate.md` | rail, work-page, deployment | the terminal shell's goldens and the stale web journeys regenerated | 120 |
| `help` | `tasks/help.md` | | the help pages' navigation words | 50 |

About 1,720 implementer-minutes in all.

**Critical path:** `addresses` (130) → `work-page` (270) → `readers` (150) →
`joint-world` (120): about eleven hours. Beside it, `deployment` (240) and `rail`
(220) end before `work-page`; `gallery`, `regenerate` and `fuzz` run beside
`readers`.

**Who waits on whom, and why:** the three page tasks (`rail`, `work-page`,
`deployment`) run at once, each on its own files in `temper-client-view`, after
`addresses` settles the shared ones. Every story outside the view that presses the
old rail waits for all three, so that one task (`readers`) rewrites each story once;
likewise the goldens and journeys (`regenerate`), which all three change. Until
`readers` lands, the client world's stories that press "New chat", "Settings",
"Goals" or "Chats" are red on the trunk; the page tasks list them under `## Left`.

## What was decided

- **Work is one address with two filters of its own,** All and Drafts; Goals
  and Changes keep their addresses as Work's other filters, drawn under Work's
  head, and Chats keeps its page, reached from Work's "Chats" filter
  (`surfaces.md` 2's default: kept). Work's five addresses share the three reads
  its list joins, so a move among them reads nothing again.
- **What goes where is one pure function** (`work::place`) over plain facts, and
  so is the rail (`rail::parts`), as `surfaces.md` 12 asks; the fuzz test runs on
  them.
- **Kind 7's Goals and Changes switches hide Work's filters, not rail entries:**
  section 12's overrides, so the switches keep a meaning until How work runs (P3)
  takes them to its foot. Stock kind 7 holds both Hidden, so a stock project's
  Work draws no Goals or Changes chip while its goals and changes still list under
  All and their addresses still open; a way that uses goals sets Goals shown. The
  rail no longer reads kind 7 at all.
- **Team is in the rail's parts but never shown:** no read gives the rail a
  project's members or a standing role, and Team has no page until P4. The rule
  is in `rail::parts` and its fuzz test; the status names the lag.
- **All projects goes to `/`,** Needs you across projects, where a sign-in lands,
  until Home across projects (P2) has an address; `rail::across` is the one
  function P2's Home joins. The rail's count on a page across projects stays the
  open project's until Needs you counts across projects (P2).
- **Deployment is a frame, not a page of its own:** Accounts, Forges, Models and
  Builders keep their addresses and acts, drawn under the kicker "Deployment";
  Builders reads "Workers"; Members reads jig's `Read::Project` per project (the
  deployment's own roster has no read); Limits draws the `deployment` document's
  capacities (the configuration's limits have no read). Both lags are in the
  status.
- **"What temper remembers about you"** has its place named in the person's
  menu, between "Accounts" and "Sign out", drawn once M4 gives it a page.
- **Tasks beside each other in one crate:** `addresses`, `rail-parts` and
  `work-list` all write `temper-client-domain`, and the three page tasks will all
  write `temper-client-view`. Each owns its own files; the shared ones are
  settled first: `addresses` owns `address.rs`, `pages.rs` and `route.rs` and
  reserves the style slots; phrase keys and style slots are split by range
  (`notes/contracts.md`, C6). This departs from one writer per crate, to run the
  three pages at once.
- **No stored format, protocol or public interface of the engine changes.**
  The client's `Address` gains three variants and their paths.

## Who hears first

- **The TUI's owner:** `temper-client-domain` gains `Address::Work { project,
  filter: WorkFilter }`, `Address::Members` and `Address::Limits`
  (`addresses`), with forced arms in `temper-client-tui`'s `view.rs` and
  `view/launchpad.rs` and `temper-client-shell-tui`'s `host.rs`, each as `Goals`
  or `Accounts` today; the modules `pages::rail` (`rail-parts`) and `pages::work`
  (`work-list`), and `Work`'s, `Goals`' and `Changes`' addresses now holding the
  goals', changes' and chats' pages open together (`addresses`). The TUI's foot
  reads `needs().count(..)`, unchanged.
- **The owner of coding runs:** no change to `temper-engine-domain` or
  `forge_route.rs`.

## Overlaps with drafts beside this one

- **`ui-p2-home`** (Home and Needs you): this plan keeps off `pages/home.rs`,
  `pages/needs.rs` and `pages/inbox.rs` in the domain and the view, but forced
  arms. Unavoidable: `temper-client-domain`'s `address.rs`, `pages.rs` and
  `route.rs` (each plan its own arms and blocks); `temper-client-view`'s
  `words.rs` (phrase keys: this plan's from 4600, far from today's 4317) and
  `look.rs`'s `NAMES`, whose dense token array both plans append to, so the second
  to land renumbers its tokens; `README.md` 3.5's page table, whose Home and
  Inbox rows sit near this plan's rows; `surfaces.status.md`'s lags 2 to 5,
  consecutive lines, of which P2 closes 3 and 4 and this plan 2 and 5; the
  greenfield README's paragraph, written here for all of section 13; the gallery's
  `names.txt` and `tests.rs` and the journeys, regenerated on a catch-up.
- **`look-round-24`:** its `role-pages` task added a `Roles` page to the trunk
  it's on (`address.rs`, `pages.rs`, `frame.rs`'s tests): a third side of
  `Address`'s arms when both land.
- **`ui-reaches`, `ui-drafter`:** no file known in common yet.
- `whole-story.execution.md`'s state column for P1 is left to the landing, as
  P2's row is beside it.
