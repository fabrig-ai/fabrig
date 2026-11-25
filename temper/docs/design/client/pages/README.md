# temper's pages

## 4. Reading pages

Before its document arrives, a task page says “Reading task N…”, or one
sentence explaining a lasting refusal; after a transient read end it asks
again while open, waiting one second, doubling to at most thirty seconds.
A conversation's activity reads, a chat's and an agent's task's, follow
the same rule: a read that ends transiently (busy) is asked again from
the position it asked for while the page is open, on the same waits, so
a committed reply always arrives without a further delivery to prompt it
(jig's `client/domain/client/README.md`: a read is one attempt, and its caller
asks again).

No page claims empty until its read has answered. "Nothing needs you.",
"No chats yet.", "None.", "No updates." and "No goals yet." stand only on a
list whose read has answered with its value. A read that ends without one
(busy, unreachable, refused, past a bound) is no answer, and neither is a
watch that has delivered nothing, whether it is opening or Behind. While a
read hasn't answered, on first load or after the link comes back, a page
keeps what it last knew, rows and words alike, through every refresh; with
nothing known, a section draws its title with no line under it.

A page redraws from whole answers alone. A read answers whole once its
value has come, every page of it read through to its last; a read or a
page of it that ends without its value is no answer, and so, for its row,
is a row's part not made (`../forge/parts.md`, 5). Neither changes what
the page drew from the answers before it: a list's rows, its count and
whether it is cut, a row's headline, the milestone a goal is under. Only
a whole answer replaces them, and it replaces them whole. So a page whose
reads answer again, in any order and with any of them ending without its
value, while nothing changes in the store, draws the same text every
time. The `inbox` watch delivers again on any task's activity in a
project the person sees, not only on a change of its needs (jig's
`client/domain/contract/watches.md`, 1.2), and a list's watch on any of its
rows' activity, so on a running project a page's reads answer again every
few seconds; what they find unchanged moves nothing.

## 5. Address contracts

Moved to [`address-contracts.md`](address-contracts.md), section 1.

## 6. Words and facts

- **Words and facts:** a chat's status from jig's facts (jig's
  `client/domain/client/helpers.md`), in jig's words, with temper's
  own in places ("Waiting for you", "Parked"); a conversation from
  jig's thread model; jig's enums and a decision's facts in jig's
  words, under temper's phrasebook (`../view.md`, 1);
  each card temper's own (`../README.md`, section 1).

## 8. Documents

- `address-contracts.md`: the pages' addresses, watches, reads and acts.
- `shared-behaviour.md`: task titles, Away and terminal approval across pages.
- `needs-you.md`: the browser's needs and Only you.
- `needs-you-terminal.md`: the terminal inbox and its cards.
- `needs-you-tell.md`: filing a person's words through Tell temper.
- `needs-you-state.md`: the needs, replies and decisions the client keeps.
- `settings.md`: the settings and deployment frames.
- `settings-workflow.md`: How work runs and choosing a way.
- `settings-proposals.md`: proposing, reviewing and deciding settings.
- `home.md`: a project's Home and the rules for its parts.
- `home-across-projects.md`: Home's aggregation across projects.
- `home-findings.md`: findings' severity, groups and cards on Home.
- `home-decisions.md`: Decided for you and overturning a decision.
- `welcome.md`: the welcome's frame, Forge and Ready.
- `welcome-model.md`: connecting and trying a model in the welcome.
- `accounts.md`: the accounts page and connecting an account.
- `accounts-words.md`: the accounts' phrasebook, waits and sign-in words.
- `ask-temper.md`: Ask temper's page and starting its chat.
- `chats.md`: the Chats page and its list.
- `chat/README.md`: a chat's layout and acts.
- `chat/composer.md`: a chat's composer.
- `chat/conversation.md`: a chat's conversation.
- `chat/document.md`: a chat's document.
- `chat/terminal.md`: a chat's terminal.
- `goal.md`: a goal's page.
- `milestone.md`: a milestone on the pages.
- `remembered.md`: what temper remembers about the person.
- `talk.md`: talking with a standing role.
- `task.md`: a task's page.
- `team.md`: the project's team and roles.
- `told.md`: telling the person and their devices.
- `work.md`: Work and Goals.
- `workers.md`: workers' settings and state.

## 1. A chat

## 2. Across pages

Moved to [`shared-behaviour.md`](shared-behaviour.md), section 1.

## 7. Open questions

- **The two-step Home and night look after landing** (`README.md`, `../view-navigation.md`, 1): temper/ui
  compares `ask-temper-start`, `to-begin-two-steps` and `night-mode` with
  main at 1440, 1000 and 420 px, with menus and disclosures open and
  each state their messages name (`../web/look-requests.md`, 4).
  Include both steps left, each alone, done and skipped, Home with a need
  and a chat left out while either step remains, and their return after
  the last step or a skip. Check the restricted navigation on Home
  and Ask temper, its three tabs on a phone and composer above them.
  Check the night switch at every width, in the rail folded beside a
  Split, open and closed, and at a phone's chat bar, reload, the three
  night pages, their shadows, and readability
  elsewhere. The night palette is the user's first pass
  to refine; system preference and the placeholder's night look remain
  `../web/README.md`, section 3's choices. Default: focused tables, gallery
  states and replayed prints establish no rendered equality or browser
  reload result.

- **Account email after landing** (`README.md`, `accounts-words.md`, 1; the request
  `ready-account-email`): temper/ui checks a real ChatGPT sign-in and a
  real Claude Code login on Ready, the Model step, the model-check pane,
  the picker and Accounts, including the terminal's words. The check
  establishes whether the real provider's claims yield an email and
  compares the rendered look. Default: focused view and integration
  checks with fake verified claims and program statuses; browser and
  process stories and the real-login render check follow landing.

- **The connect-flows comparison** (`README.md`, `../forge/family.md`, 4, `../view.md`, 2): temper/ui compares
  request pages 01–23 with main, including named states and open menus
  and disclosures at 1440, 1000 and 420 px, with screenshots also at
  1280 px (`../web/look-requests.md`, 4). Default: the source and print
  account is distinct from rendered equality; differences return as
  findings after landing. Compare the request-owned parts, including
  Settings' select chevrons, the sheet's chooser, captions, list and empty
  search, More's summary, connection actions and each model way's path;
  the shared H1 and open project menu row keep their frame's design.
- **The trial's later choices** (`welcome.md`, 1, *The try*, `accounts-words.md`, 1): whether Settings
  offers "Try it", whether deployment accounts are tried, how a second
  account is tested while selection stays a new chat's, and retaining a
  failed line across a restart are `../../domain/trial.md`, section 8.
  Its project-only key limit stands there too. Default: the account that
  a new chat selects, personal accounts alone, and no retained line or
  additional Settings act.

- **The rail, Work and Deployment after landing** (`work.md`, 1, *Work*;
  `settings.md`, 1; `../view-navigation.md`, 1): temper/ui compares main with the whole
  story's mockups, `web/whole-story/work.html`, `pair/work.html`,
  `team/work.html` and each page's `nav.rail`, at 1440, 1000 and 420 px, by
  day and by night, with the new box's menu, the switcher's All projects
  and the person's menu open, Work under each filter and Deployment's six
  tabs; and walks them on a box (`../../draft/whole-story.execution.md`,
  section 2). Owner: temper/ui. Default: each difference a finding filed
  back to the plan's repair.
- **Home and Needs you across projects after landing** (`home.md`, 1, *Home*,
  *Home on All projects*; `needs-you.md`, 1): temper/ui compares main with
  the whole story's `web/whole-story/home.html`, `needs.html`,
  `phone-home.html` and `phone-need.html`, and `pair/`'s and `team/`'s
  Home and Needs you, at 1440, 1000 and 390 px, by day and by night, with
  Only you's region against `needs.html`'s through `../web/gallery.md`, 2's
  `needs-only-you` and `night-needs-only-you`, and Accounts'
  `accounts-expiring`; walks them on a box across two projects, a
  sign-in signed out reaching Accounts from Only you and renewed there;
  and records the journey of Home to a
  need, answered (`../../draft/surfaces.md`, 14). Owner: temper/ui. Default:
  each difference a finding filed back to the plan's repair.
- **The phone after landing** (`needs-you.md`, 1, *On a phone*; `ask-temper.md`, 1, *Ask
  temper's page*; `../view-navigation.md`, 1, *On a phone*): temper/ui compares main with the
  whole story's `web/whole-story/phone-home.html` and `phone-need.html`,
  and the Ask tab with `phone-tell.html`'s top bar and tabs, at 390 px, by
  day and by night, through `../web/gallery.md`, 2's `all-home`, `needs-list`
  and `ask-chat`; presses each target of the top bar, the tab bar, Home,
  Needs you, Work and a chat's bar and composer, measuring its 44 px; and
  walks a need answered from Home and back on a phone-sized page. Owner:
  temper/ui. Default: each difference a finding filed back as a repair.
- **Home's and Needs you's later parts** (`home.md`, 1, *Home on All projects* and
  *A page's parts*; `needs-you.md`, 1; `../../draft/surfaces.md`, sections 3, 4,
  12 and 15): a project's Now on its row, rolled from its owners', its
  tiers, a follow's parity and "owned" (R1 to R3, F1); who decided, on a
  Decided for you row, and the intro's What
  reaches you; on an Only you item, what waits for it ("Two agents on vsr
  wait for this login.", `../../draft/what-reaches-you.md`, section 8), once
  jig's `accounts` document names the tasks waiting on each of a person's
  own accounts, as the `deployment` document does the deployment's (jig's
  `client/domain/contract/documents.md`, 1); Only you's other kinds, a forge's
  token to replace, and a person task asking an act rather than a choice,
  which stays among the cards meanwhile; Only you in the terminal's inbox
  (`../tui/pages/README.md`); a need's goal, role and asker on its from
  line; one brief across projects written per person,
  `../../draft/surfaces.md`, 15's open question; and more than 16 projects
  read. Each joins *A page's parts* with the read that gives it. Default:
  none is drawn until it is built, the brief is one a project, joined, and
  the first 16 projects in `me`'s order are read.
- **What a real box's walk has not shown** (`README.md`; `../../protocol/setup/owned.md`, 6; the
  requests `ui-final-walk` of 2026-11-05 and `ui-walk-3` of 2026-11-06): the pinned
  Claude Code fetch, from a box with directories of its own up to the sign-in prompt
  and reused on a second run, and a real browser's Sign out ran by hand, on fresh
  loopback deployments, and no test keeps them. Not shown: a provider's
  sign-in or a paid call; TLS, a proxy or a browser on another machine; Accounts
  with a project or an account on it; the small engine profile whole, whose client
  limits alone ran; another platform's build, a failed or corrupt download, two
  fetches at once; a copy from a public host. Default: none, until a plan walks them.
- **Pages the mockups do not show:** a project's page with its
  resources, findings, notes and settings (policy, people and roles,
  spend; jig's `client/domain/pages.md`, section 6, and the forge's
  parts in `../forge/repositories.md`, 3, and `../forge/README.md`, 6.3), but the settings' Delete (`README.md`); the deployment's page but its LLM
  accounts, which the accounts page shows an owner, and its forges,
  which the forges' page shows (`README.md`); the "Tasks" list of a project,
  which the mockups draw, after jig's pages (`jig/docs/design/client/domain/pages.md`, 6.3
  and 7); of the deployment's hosts, which the mockups draw as "Workers",
  all but what Settings' Builders tab reads (`README.md`): its capacity and its runs;
  a procedure's task beyond `task.md`, 3's page, and a person task beyond a
  tracked task's header and plan; making a service and showing its
  credential once; another party's kept work. Default: not in temper's
  client; the person acts on them through the engine's configuration,
  or not at all.
- **The playground's and the agent's placeholders** ("Try a suggested
  request…"; "Write to T12…", the
  agent's task's composer, `task.md`, 4): skein-ui's text area takes a
  placeholder, which a chat's composer and `NewChat`'s, "What do you want
  done?" in every project, the playground's start among them, draw
  (`README.md`). Default: neither drawn, the agent's waiting for its page's
  owner.

- **A page refused busy.** How a page says its rows have not arrived,
  rather than calling them absent, belongs to the client's presentation
  (`../README.md`), outside watch sizing. Default: keep the handle's
  behind state and its retry (jig's `client/domain/client/watches.md`, 1).
