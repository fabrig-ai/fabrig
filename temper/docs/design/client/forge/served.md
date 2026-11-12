# What each part is served from

Each part is the forge's family's (`family.md`): read
or watched by the forge's client side through the family's own reads
and subjects, under the forge's owner number, each naming its scope, a
project, the deployment or the party's own (jig's
`client/domain/contract/README.md`, 5.5; jig's
`client/domain/root.md`, section 7), their records in
`temper-client-protocol-wire` beside `Adopt` (`records.md`).
The first read served is the playground's (`records.md` and `family.md`, 7
), from its `Playground` row; the first watch, a change's panel (2); then
the deployment's part, its forges (`deployment.md`), as the subject `forges`, whose
rows carry each forge's number, kind, mode and connection (`records.md`
); the rest of a forge's row (its `web` address, its last connect's
end, its installations, its load and its actions) and a change's result
and its hold (`README.md`, 2.5) follow.

| The client | jig's part and slot | The domain |
|---|---|---|
| a change's way to landing, its gates, requirements and heads | a panel, on its task's page | `../../domain/forge/change/README.md` |
| a change's result | a result, on its task's page, in a chat, in an update | `../../domain/forge/change/README.md`, 2 and 4 |
| its holds | a hold, on the held card | `../../domain/forge/change/repairs.md` and `../../domain/forge/change/README.md`, 3; jig's `domain/connectors.md`, 4.3 and section 11 |
| what a proposed write would do; the outbox | an effect, on a proposal card; an outbox entry | `../../domain/forge/README.md`, section 4 |
| a project's queues, its branches' red and its last landings | temper's `Changes` page, from the subject `changes` (`records.md`) | `../../domain/forge/queues.md`; `../../domain/after-landing.md`, section 10 |
| approvals | a person task's card; a panel; a change waiting on a review, a card under Needs you from the subject `approvals`, of the party's own scope (`README.md`, 4.1; `records.md`) | `../../domain/forge/change/gates.md`, 1 and 2, and `../../domain/forge/change/questions.md`, 2; jig's `domain/people/README.md`, section 7 |
| adoption, repositories, health, landing rules | temper's `Adopt` page; a project's part | `../../domain/forge/adoption.md` and `../../domain/forge/keeping-up.md` and `../../domain/forge/change/gates.md`, section 2; jig's `domain/connectors.md`, section 12; jig's `domain/authority.md`, section 10 |
| goals' issues, landings as news, findings | a project's part; a goal's page | `../../domain/forge/README.md`, section 5 and `../../domain/forge/issues.md` |
| the forge itself | the deployment's part | `../../domain/forge/keeping-up.md`, 4 |
| the picker (`repositories.md`, 1) | temper's `Adopt` page, through the `Reachable` request (`../engine.md`) | `../../protocol/forge/README.md`, the setup's host's `Reachable`; `../../domain/forge/adoption.md` |
| each forge's connection, its mode, installations and actions (`deployment.md`, `repositories.md`, 4) | temper's `Forges` page; the deployment's part | `../../protocol/forge/README.md`, at start, connecting at runtime, a forge stopped, GitHub's installations and the setup's host; `../../protocol/credentials.md`, the forges' records |
| the browser's steps of a connect (1) | none of jig's: temper's routes on the clients' listener | `../../protocol/forge/README.md`, the setup's host |
| a public project's changes and landings, for a visitor (`README.md`, section 9) | jig's public reads, `PublicRead`, of the forge's public kinds | `../../domain/forge/change/README.md` and `../../domain/forge/queues.md`; `../../domain/landing.md`, section 14 |
| the playground: its forge, project, model, repository, state and suggested requests | the deployment's read, `playground` (`records.md`), for its banner, its pages, `Chats` and `NewChat`; its start and its reset, `StartPlayground` and `EndPlayground` | `../../protocol/setup/playground.md` |
| a forge removed | the deployment's request `RemoveForge` (`records.md`), from the Forges page | `../../domain/forge/adoption.md`, 1 |
| a person's facts: what they dismissed, the goals they started from a chat | the project's request `Dismiss` and the subject `dismissed`, of the party's own scope (`records.md`): the forge side's facts, which no page reads (`../README.md`, section 5) | `../../domain/forge/attention.md`, 1 |
| a project's stock ways and their paths, and a view of a path, a way, a proposal or a charter | the project's subject `ways`, on Ask temper's page, and the project's read `View`, for a `view` block in a chat (`records.md` and `../pages/settings.md`, 2 and `../pages/chat/conversation.md`, 3), each made by the root from the core and the forge | `../../domain/forge/change/branch-path.md`; `../../protocol/configuration/settings.md`, 8 |
| temper's settings: a landing rule's value, the forge's charter, a charter's words, a project's questions, ways and words, and a pending proposal's items of temper's kinds | the deployment's reads, and a project's reads of its own kinds (`../../protocol/configuration/settings.md`, 2, *Read from the forge*), `Setting { kind, scope, number }`, `RevisionSetting { kind, scope, number, revision }` and `ProposalItems { key }`, the key jig's settings review gives a pending proposal of settings naming temper's kinds (jig's `domain/engine/settings.md`, 6), beside jig's `settings` document, which gives their states and sizes | `../../protocol/configuration/settings.md`, 2; jig's `domain/engine/settings.md`, 5 |

The revision-qualified setting read serves an explicit set item from a
retained revision. It uses jig's authenticated revision read to confirm
that revision's item, then reads the forge's value saved before the next
overwrite, or its current value when no later overwrite exists. The
forge retains overwritten defaults and explicit values in its replaced
slots for the same history as jig's revisions. A release retains the old
default before changing it. Unknown or forgotten revisions, non-set
items and missing retained values return no value; today's value never
stands in for a missing historical one. The read is one bounded value,
with the setting answer's byte ceiling. The tools decode its landing
rule or charter before joining it with revision metadata (`../ctl/tools.md`, 2).

## 1. The browser's steps, on the clients' listener

A GitHub connect's browser steps are temper's routes on the clients'
listener (`../engine.md`): temper's server asks jig's clients'
component first whether it claims a path (jig claims `/jig/...`), then
routes `/forges/N/github/...`, N a number in decimal, then serves the
browser's bundle. GitHub sends the person's browser to them, and
nothing of temper's client calls them. Every other path, the client's
`/forges` and `/forges/connect` among them, is the bundle's
(`../web/README.md`, 2.3).

| Route | What it does | Answers |
|---|---|---|
| `GET /forges/N/github/manifest?state=S` | temper's page for that state, rendered from what the setup's host's `Manifest` run answered, the form's target and the manifest, and held with it, at most `page_bytes`: one form, the manifest its hidden `manifest` field, escaped as an attribute's value, posting temper's manifest to `WEB/settings/apps/new?state=S`, the person's own account's, which posts itself as the page loads; a browser that runs no script, or one come back to the page through its history, shows instead "temper’s App, on GitHub", its button "Continue to GitHub" and "Back to temper", a link to the connect's `back`, else `/forges/connect?forge=N`; the manifest's redirect and setup addresses are whole URLs on the clients' listener, `http://LISTENER/forges/N/github/redirect` and `http://LISTENER/forges/N/github/setup`, and its `url`, the App's homepage, `http://LISTENER/` | `200`, the page |
| `GET /forges/N/github/redirect?code=C&state=S` | `Convert`: the code exchanged for the App's id, slug and key, each kept before the browser is answered, and its user; the App's hook points at `https://SLUG.invalid/`, so no webhook secret is kept on GitHub; then `Install` for a fresh state, held with the connect's `back`, whose link answers the browser, and the forge connects | `302` to the install page, `Install`'s link for that state |
| `GET /forges/N/github/setup?installation_id=I&setup_action=A&state=S` | `Accept`: the installation checked and accepted, then the forge's installations handed to its component again | `303` to the state's `back`; with none, `200`, a short page: "temper has the installation; you may close this tab." |

N is the forge's `number`, WEB its record's `web` address and
LISTENER the clients' listener's address, as temper builds the
manifest from it (`../../protocol/forge/README.md`, the setup's host).

- **The tab goes and comes back** (with the user, 2026-10-23): the
  page that sends `Connect` or `Install` names `back`, its own address,
  which the setup's host holds with the state (a connect's passes to
  the install `Convert` starts), and the browser goes to the link in
  the page's own tab, where a sign-in has a tab of its own (`../pages/README.md`,
  `ConnectModel`, The sign-in's tab). The setup route sends it back
  there, so the page that started it shows how it ended, as a new
  client: the welcome's Forge step, `ConnectForge` at the forge, or the
  Forges page. A connect from the terminal names none, and its routes
  answer their pages as before.
- **The manifest's page's headers:** `Content-Security-Policy:
  default-src 'none'; form-action WEB; script-src 'sha256-H';
  style-src 'sha256-S'`, H the hash of its one inline script and S
  its one inline style. The script submits the form unless the
  page was reached through the browser's history (its navigation's
  type `back_forward`), so that back from GitHub's form stops on the
  page rather than posting again; `Cache-Control: no-store` and
  `Referrer-Policy: no-referrer`. It loads nothing, runs nothing but
  that script, and may post only to the forge's `web`.
- **The pages' look** is the one of a Google sign-in's endings
  (`../../protocol/sign-in.md`, 4.3; the look request `outside-pages`):
  temper's mark in a `header`, a link to `/`, then the column, its
  heading level 1, its first act filled (`primary`) and a following
  link quiet (`quiet`). Their one inline style takes the rules of the
  bundle's `files/pages.css`, but the `@font-face` rules, since
  `default-src 'none'` fetches nothing: the system's sans stands in.
  The manifest's page's "Continue to GitHub" is filled and its "Back to
  temper" quiet. Each hash, S and H, follows its inline text.
- **The state is the only authenticator.** A route takes no cookie and
  no credential of jig's: the state, 128 bits or more drawn by the
  setup's host for one connect or one install, is good once, for its
  life (`setup.state_life_ms`), for that forge alone. The host keeps at
  most `setup.states` of them, in memory, so a restart forgets them
  (`../../protocol/forge/README.md`, the setup's host).
- **Where GitHub's way can't go on,** the route answers a short page
  of its own, temper's mark above one column, as a Google sign-in's
  endings are drawn (`../../protocol/sign-in.md`, 4.3): its level-1
  heading, why, and "Back to temper", a filled link, in place of
  sending the tab back; each with
  `Content-Security-Policy: default-src 'none'; style-src 'sha256-S'`,
  S the hash of its one inline style, `Cache-Control: no-store` and
  `Referrer-Policy: no-referrer`, and nothing is kept. The pages:
  - *Ended:* a state used, unknown or past its life, or a code GitHub
    refuses, is answered `410`: "This link has ended", "It was used
    already, it expired, or temper restarted since. Start again from
    temper.", its link to `/forges/connect?forge=N`, since the state's
    `back` is gone with it. So is a run that stops otherwise:
    `Convert`'s `Stop`, whatever its failure, an `Install` that stops,
    and `Accept`'s `Stop`.
  - *A request:* GitHub's visit with `setup_action=request` and no
    installation, after a member of an organization who is not its
    owner asked for the App there. Nothing is accepted: the routing
    takes the state's `back` from what it holds (removing it), and the
    setup's host's state is left to the end of its life, since `Accept`
    needs an installation to use it; a later visit with that state and
    an installation is accepted as any other, and answered as one with
    no `back`. The route answers `200`: "GitHub sent a request, not
    an installation", "Only an owner of an organization can install
    temper’s App there, and temper can’t wait for an owner to approve
    it yet: an installation approved later isn’t used. Install it on
    your own account, or on an organization you own.", its link to the
    state's `back`, else `/forges/connect?forge=N`. An owner's approval
    later makes an installation with no state of temper's, a stray
    (`../../protocol/setup/github.md`, 4); waiting for it is for later
    (`../../draft/github.md`, section 18).
- **A busy forge holds the visit.** GitHub's setup visit with an
  installation and a state, for a forge whose one run is live, as the
  reconcile after the App's `Connected` (`../../protocol/setup/github.md`, 5),
  which an install straight after a manifest's conversion meets, is held,
  not refused: the routing keeps its call, installation, state and
  `back`, at most `setup.states` of them, and starts its `Accept` once
  the forge's run ends. One held past the state's life
  (`setup.state_life_ms`), or with no room to hold it, is answered `410`
  as ended.
- **A visit with no state** to the setup route is GitHub's, after a
  person changed the installation's repositories on GitHub: for an
  installation accepted already it is a refresh, answered `303` to
  `/forges`, the routing handing the root `Repositories` for it so the
  picker lists them again, and nothing reaches the forge's component
  or the store; for any other installation it is refused, `410`, as
  ended. An update's visit with a live state is the same refresh, sent
  on to the state's `back`: GitHub may keep an install link's state
  through Configure to the update it saves, which the row’s "Choose its
  repositories on GitHub" counts on, and which the probe App is to
  show; where it doesn't, the tab lands on Forges.
- **Bounds and others:** a query malformed or past `routes.query_bytes`
  is answered `400`, in the refused page's words and headers; a method
  other than `GET` is answered `405`; a forge the store does not keep,
  or one that is not GitHub, `404`; any other path under
  `/forges/N/github/`, not found.
- **A fixed address with a GitHub forge.** The App's `redirect_url` and
  `setup_url` are those whole URLs on the clients' listener, and
  GitHub's API cannot change them, so a deployment configuring a GitHub
  forge gives its clients' listener a fixed address, never port 0; the
  engine
  checks it at start and refuses to start, naming it
  (`../../protocol/configuration/README.md`, the clients' listener). GitHub takes
  `http://127.0.0.1:PORT` for both, so a standalone deployment needs no
  public address: the browser that follows the link runs where that
  address reaches (`README.md`, section 8).
- **Tested without GitHub.** The fake forge's manifest post answers
  `200` with a small page holding one link, `Create GitHub App` (id
  `create`), to the manifest's `redirect_url?code=C&state=S`, which a
  test follows as a person clicks GitHub's form (a link, not a
  redirect, so the page's `form-action` is not crossed), and its install
  page answers `302` to the `setup_url` (skein's `testing/fakes/github.md`, 1.1);
  so the three steps run as plain requests on the clients' listener.
  The routes' tests are `temper-service`'s own, requests through
  `fixtures::http::Peer`; the bytes through the fake's GitHub layer are
  the real loop's (`../../testing/real-loop.md`), whose browser's tier
  opens the manifest's page once to see it post its form, by itself, to
  the forge's `web`; no test calls github.com.

## 2. A change's panel, watched

A change's panel (`README.md`, section 2) is the family's first subject, `change`,
watched by the forge's client side while the change's task page is open
(`../pages/README.md`). Its document, `ChangePanel` (`records.md`), is
made by the forge connector from what it keeps of the change
(`../../domain/forge/change/README.md`, 5).

- **Its subject** names the change task by its number, in the scope of
  the task's project. jig admits the watch on the party's role in that
  project (jig's `client/domain/contract/README.md`, 5.2). The connector
  answers unknown for a task that is not a change into a repository the
  project holds, and the page's slot says the panel is unknown (jig's
  `client/domain/root.md`, section 6). The watch's window is unused:
  the document is whole within its bounds.

  A panel whose watch the engine ended may be unknown before the forge
  registers its change. For the same live change task, or its ended
  document with a change result, the page reopens that panel once per
  later task document delivered, with a fresh watch. A SubjectEnded
  ending also reopens it immediately only if the page's document is
  already Ended with a Done(Change) result, at most once per document's
  activity; a second ending leaves it Unknown until a new document.
  Every other document, including a live procedure without a change
  record, waits for a later task document. An unsolicited Closed follows
  SubjectEnded because jig gives the client the same reason for both;
  a local unwatch's acknowledgement is ignored. Malformed or mismatched
  deliveries, another task, every other end reason, and an ended task without a
  change result remain terminal. Between the procedure's start and the
  next task document the page may say "no change"; it recovers on that
  next document, at the latest at the landing.
- **Level-triggered,** as every watch: each delivery is the whole panel
  as it is when made, and a slow client sees fewer of them. The
  connector marks the subject when the change's record changes (its
  step, its pull request, a gate's verdict, a bound used, its hold),
  and when a step reads CI at its head and finds a verdict other than
  the one kept. When a change enters its landing branch's queue, leaves
  it or moves in it, the changes behind it are marked too.
- **What it carries:**
  - the repository, by its forge's number and its owner and name
    (`README.md`, 2.5); the change's branch and its landing branch; its head;
  - its pull request: its number once opened, and its address on the
    forge;
  - its step, the one `README.md`, 2.1 names, with what the step says: its place in
    its queue and the tasks ahead of it, a repair's reason, or its merge
    commit once landed;
  - its hold, with the step it was held at;
  - the task its step runs (its producer, a repair, a resolution or a
    gate's review), at most one at a time;
  - CI at its head and `ci_required`, whether the repository and its
    effective landing rules ask for it. `ctl` text says `CI not required`
    when false; JSON carries `ci_required` beside the observed `ci`
    verdict, including a retained terminal verdict;
  - its gates (`README.md`, 2.2), each with its number, its kind, whether it is
    blocking or advisory, and its verdict at the current head, a clean
    update's carry counted (`../../domain/forge/change/gates.md`, 1);
  - its repairs, resolutions and updates, each used of its limit (8.4);
  - its questions (`../../domain/forge/change/questions.md`, 1): its route, the questions
    taken used of its bound, and its open questions, each with its
    asker, the first line of its words and where it stands, so that a
    person sees what a change waits on before its requester answers.
- **The address on the forge is the engine's.** The routing builds it as
  it encodes the document, from the addresses the forge's record keeps and its
  kind: `WEB/OWNER/NAME/pull/N` on GitHub, WEB its `web`, github.com's
  when null; `GIT/OWNER/NAME/pulls/N` on Forgejo, GIT its `git` base,
  since Forgejo serves its pages where it serves its repositories and
  its `web` is always null (`../../protocol/configuration/engine.md`).
  A fake forge serves no pages and gives none; nor does a forge with
  neither address. The page then offers no "View on forge"
  (`../pages/task.md`, 3); so does an address longer than its bound
  (`records.md`), which never fails the delivery.
- **Not in it,** since the connector's records do not keep them: the
  requirements at its head, judged only at the merge
  (`../../domain/forge/change/gates.md`, 2); its heads in order; each CI context and
  its failing job; the pull request's title and mergeability as the forge has them; its
  stall's times; its place in GitHub's merge queue (`README.md`, section 8).
- **As long as the change's record:** a change that has landed or ended
  keeps its last panel, delivered when its page opens, and nothing marks
  it again.

## 3. Open questions

- **A Forgejo's own links at its `web`** (`../../protocol/setup/box.md`, 4; `../../protocol/openid-provider.md`, 7): a
  Forgejo connected with a `web` apart from its `git`, as a host's own
  units' is, takes its browser origin from it for sign-in alone. Its pull
  links, the public page's repository address and a brief's `Address:`
  are made from `git` (`served.md`, 2), so on that walk they
  name its loopback address. Taking `web` when it has one changes those
  three and that section, whose "its `web` is always null" goes with
  them; the request of 2026-11-05 left `docs/design/client/` alone.
