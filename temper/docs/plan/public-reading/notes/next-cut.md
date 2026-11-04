# public-reading: the code batch, sketched

Written by the design cut (2026-11-03) for the plan that builds public reading once the design's
security review has answered (`plan.md`; the report's Question). A sketch, never a spec: that
cut checks every file and name below against the trunk it then finds, surveys the readers,
and writes the specs; where the review changes the design, the design changes first.

The design it builds:

- jig: `jig/docs/design/domain/people.md`, section 13 (and 4, 5.1, 12); `protocol/client.md`,
  17.1, 17.5 and section 19; `client/domain/contract.md`, 3.2, 5.1 and section 12;
  `domain/engine.md`, 11; `domain/root.md`, 3 and 10; `domain/authority.md`, 6;
  `domain/tasks.md`, 3; `protocol/configuration.md`, 6; `protocol/store.md`, 4.4 (read by
  `jig-protocol-store`'s `tests/words.rs`, for its sections 2, 5.3, 6 and 10 alone).
- temper: `temper/docs/design/client/forge.md`, section 11 (and 9's table);
  `client/browser.md`, section 8 (and 2.3, 2.4, 5); `protocol/sign-in.md`, 2.4;
  `client/tools.md`, 5.2 and 5.3.

Every one of them has its Not yet entries in its status file; each task removes the ones it
builds.

## Contracts to fix first, in the next cut's `notes/contracts.md`

- **The public party in code:** a variant of the core's requester (no party number), the
  role `public` a name `jig-protocol-config`'s `check_policy` and the core's policy edit and
  settings check refuse; where the people child's `Role` enum (`jig-domain-people`,
  `boundary.rs`) is open, `public` is refused as a name, never a variant a policy can hold.
- **`PublicRead { owner, version, kind, numbers }`** and its answer `PublicAnswer`
  (document bytes, unknown, busy, version not served), in `jig-client-protocol-wire`'s own
  public schema (`public-v1.schema`, its own version), with `PublicProjects`, `PublicBoard`,
  `PublicGoal`, `PublicCard`, `PublicLine`; jig's kind numbers (1 projects, 2 board, 3 card).
- **An application's public kinds:** what it declares at start to
  `jig-protocol-engine-clients` (owner, kind, whether it takes a number past the project, its
  answer's bound), and the connector's ask for a public document (the root's `Ask::Read` with
  the public party's scope, or an ask of its own).
- **The limits:** `clients.public.reads`, `.rate`, `.bytes`, `.max_age`,
  `clients_http.public_calls`, every profile's zero; the root's `public_watermark`.
- **`SetVisibility { project, public }`, `ShowTitle { task, shown }`** in `contract::Ask` and
  the client wire; `me`'s project `public: bool`.
- **temper's public records** in `temper-client-protocol-wire`'s `public-v1.schema`:
  `PublicChange`, `PublicLandings`, `PublicLanding` and their parts (`PublicStep` without
  text, `PublicHold` as a kind, `PublicGate`, `PublicMember`), and the forge's kind numbers
  (1 change, 2 landings, 3 landing).

## The tasks, as sketched

Part `jig-public` (slice 1: a public project's board read by `GET` with no credential, a
private one unknown, past the rate busy while a signed-in read is answered):

| Task | Kind | Needs | Owns, roughly | Minutes |
|---|---|---|---|---|
| `public-surface` | interface | | `jig-domain-people` (the public party, `SetVisibility`, `ShowTitle` refused `Shape` until built), `jig-protocol-config` (`public` reserved in `check_policy`; the limits, zero in `profile/small.rs` and standard), `jig-client-domain`'s `contract::Ask`; compiler-forced arms in jig and temper | 90 |
| `public-wire` | interface | | `jig-client-protocol-wire`'s `public-v1.schema`, generated code, record goldens, the text golden of public fields and the closure test; the client wire's `SetVisibility`, `ShowTitle` and `me`'s `public` (in place, as configuring-p2-build's `settings-wire` did); `jig-protocol-engine-clients` and `jig-client-protocol-link` translations | 120 |
| `public-store` | feature | surface | `jig-protocol-store`: the people family's project row's `public`, the tasks family's goal title flag and whether its spec is its requester's (a schema version each, codec, goldens) | 120 |
| `visibility-core` | feature | surface, store | `jig-domain-core` and `jig-domain-people`: `SetVisibility` (owner only, project 0 and a deleting project refused), `ShowTitle` (requester, own `Create`, cleared by another's amendment or a move), `me`'s flag; unit tests in `jig-domain-core/src/tests/` | 120 |
| `public-reads-core` | feature | surface, wire, visibility-core | `jig-domain-core`: `PublicRead` admitted on visibility, jig's three projections from live tasks and an ended load, goal trees only, the connector's ask for an application's kind; the root's public watermark (`jig-test-service`'s root and `jig-test-domain`); unit tests | 180 |
| `public-component` | feature | wire, surface | `jig-protocol-engine-clients`: the `PublicRead` exchange with no credential, declared kinds, the public room, the reads and bytes buckets on the monotonic clock, busy at once; every other credential-less exchange still signed out; `tests/engine-clients` stories | 150 |
| `public-http` | feature | component | `jig-protocol-engine-clients-http` (the `GET` route's canonical path, no cookie or bearer read, `200`/`404`/`503` with their headers, the public calls' room) and `jig-client-protocol-http` (the `Exchange` with no bearer); the mapping's world | 120 |
| `public-story` | feature | reads-core, http | jig's core world (`jig/tests/core`, people.md 13.6's stories and referee) and the client-protocol world, a public read over native HTTP bytes; `ops` declares no public kind | 120 |

Part `temper-public` (slice 2, the walk: a public project on fakes read at `/public` in the
browser, its board, a task with its change and the landings, with no sign-in and no stream;
made private, unknown):

| Task | Kind | Needs | Owns, roughly | Minutes |
|---|---|---|---|---|
| `forge-public-wire` | interface | | `temper-client-protocol-wire`'s `public-v1.schema`, goldens, the text golden and closure test | 90 |
| `forge-public` | feature | forge-public-wire, public-reads-core | `temper-engine-domain-forge`: the three projections from its changes' and landings' state, a change outside a goal's tree unknown; its kinds declared through `temper-engine-domain`'s root and `temper-service`'s start; forge world stories (forge.md 11.4) | 150 |
| `public-ctl` | feature | public-wire, visibility-core | `temper-client-tools`: `project amend --visibility`, `goal amend --title`, `project show`'s visibility; `docs/help/` pages and `reference/` lines for them; golden | 90 |
| `public-web` | feature | forge-public-wire, public-wire, public-http | `temper-client-domain`'s address codec (the `/public` paths), `temper-client-view`'s public pages, `temper-client-shell-web`'s public mode (no sign-in, reads by `GET`, its timer, hidden tabs, busy and unknown), the signed-out page's link; step tests | 180 |
| `public-walk` | feature | forge-public, public-web, public-ctl, public-story | temper's browser tier story (browser.md 8.4) and a `tests/client` story through ctl: a project made public, read by `GET` through temper's server, made private | 120 |

Critical path as sketched: `public-surface` (90) -> `visibility-core` (120) ->
`public-reads-core` (180) -> `forge-public` (150) -> `public-walk` (120), about 660
implementer-minutes; `public-wire` -> `public-component` -> `public-http` -> `public-web`
(570) runs beside it. Six tasks start at once: `public-surface`, `public-wire`,
`forge-public-wire`, then `public-store`, `public-component` as their needs land.

## What the next cut must check

- The people child's `Role` (open enum or not) and how `Policy { role }` names roles, for
  where `public` is refused.
- How the root admits a read today (`Ask::Read` to the connector, `reads.rs` `start`), for
  whether an application's public kind is a read with the public party's scope or an ask of
  its own.
- Whether `client-v19` is still in place on its trunk or a v20 is open, for `SetVisibility`'s
  and `me`'s fields.
- The ended-goals load for the board: a task query's ended part by project and label, or a
  load of its own.
- What the review changed.
