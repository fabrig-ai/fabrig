# debt-13: skein D013 and jig D004

Two independent parts (`inputs/request.md`). skein D013 is paid and deleted here. jig D004 is size L: this plan cuts
what both roots copy word for word, and `root-close` rewrites the item to say what's left.

## Parts and tasks

| Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---:|---|
| `forge-tail` | `forge-tail` | | medium | 30 | `grow::LogTail` over lib's `Tail`, with lib's `into_boxed` |
| `forge-entry` | `forge-entry` | | high | 75 | `List<ForgeEntry>`, `Owner`, `check_added`, `swap_remove`; methods still take a position |
| `forge-entry` | `forge-common` | `forge-entry` | high | 120 | `Component { forges, common }`, every step takes its entry, `Connection::role`; Measure 0 |
| `forge-entry` | `forge-dialect` | `forge-common` | high | 90 | `Dialect` matched once per walk, `Active` for GitHub's work |
| `forge-exchange` | `forge-calls` | `forge-dialect` | high | 75 | `Waiting::side`, one dispatch, the cross arm answers `Unavailable` |
| `forge-exchange` | `forge-exchange` | `forge-calls`, `forge-tail` | high | 150 | a connection with no dialect, holders keep typed exchanges; D013 goes |
| `root-admission` | `root-admission` | | medium | 35 | `Class` in `jig-domain-link`, `LinkUp::new_work` |
| `root-timers` | `root-timers` | | medium | 30 | `Env::wall_due`, `decision_header` |
| `root-sender` | `root-sender` | | medium | 20 | `Party::sender` |
| `root-leaves` | `root-leaves` | `root-sender` | medium | 50 | the hub link's job, settled answer, brief and no-workspace |
| `root-charter` | `root-charter` | `root-leaves` | high | 75 | `hosted` and `message` in `jig-protocol-charter` |
| `root-now` | `root-now` | `root-charter` | high | 60 | `hub_link::now`; `ops`'s `link.rs` loses its repeats |
| `root-close` | `root-close` | `root-admission`, `root-timers`, `root-now` | medium | 25 | D004 rewritten, the status files true |

## Critical path

`forge-entry`, `forge-common`, `forge-dialect`, `forge-calls`, `forge-exchange`: 510 minutes, one crate, so one writer
at a time. jig's longest path is `root-sender` to `root-close`: 230 minutes. The two parts share no crate but
`skein-lib`, in different files.

## Decisions

D013, after re-measuring: 155 `get(at)` and 54 `unreachable!`, both as the item says. D010 and D016 changed neither.

- **A connection speaks no dialect, and its holder keeps the exchange.** Removing the dialect's `unreachable!`s by
  rewording them, or by answering `None` on a mismatch, would hide the panic the item objects to. Generics and traits are
  out of step code. Two typed connection slabs would cost an id enum and a second slab. An exchange lives only from
  `start` to its end, so it can sit with whoever holds the connection, and the mismatch can't be written. The pump's
  64 movements and their order stay, so calls don't interleave differently (`notes/contracts.md`, 3).
- **A call's variant and its forge's meet in one place,** and a mismatch there answers `Unavailable`, as a call to a
  gone forge does. A plan and a GitHub state sit in different owners, so one comparison has to exist somewhere: it's one
  total match with a safe answer, in place of about twenty panics. `remove` ends a forge's calls, so it isn't reached.
- **`Start` stays one public enum:** worlds match on it. GitHub's `unreachable!`s on it go by `Active`, a local enum.
- **`Connection::role` replaces the scans** in `call_connections` and `assign`, which walk every forge while one is
  held. It's the one piece of `forge-common` that isn't mechanical, so that task commits it apart.
- **`grow::Tail` keeps a wrapper,** `LogTail`, because the runner's failure marker and `dropped` are its own. Lib gains
  `Tail::into_boxed`, so a log's finish makes no second buffer of the body's size.
- **One assertion goes, the cut's call:** `the_tail_keeps_the_last_max_bytes_across_writes`'s `capacities` column pins
  `grow::Tail`'s own growth, which is the mechanism the item removes. Its bytes and `dropped` stay, and lib's tests pin
  how a list grows. It's no visible event or outcome.
- **D020 stays out,** as the request doesn't name it: `deadline` sites and flags are carried over as they are. The
  neutral connection leaves it less to do.

D004, from the survey of both roots:

- **Much of what the item calls copied isn't a copy.** The test root has no host child, so `host::Request`'s arms are
  `ops`'s alone. Its charter assembly lives in `jig-test-service`, and its restart differs in substance. `jig-domain-hub-link`
  is already the item's "engine link" part.
- **Cut:** `Class`, `LinkUp::new_work`, `wall_due`, the decision's header, `sender`, the job's assignment, the settled
  answer, the brief's plan and completion, the fixed workspace answer, the charter's hand-over, the link's `Now`s, and
  `ops`'s own repeats in `link.rs`.
- **Left, each because it changes what a test sees or needs the user:** the test root rebuilt in `ops`'s shape, restart,
  `RunPrepared`, `RelayBusy`, `DropCall`, the `Held` arms, the `document` loop and temper's copies. A branch stays in
  its tree, so temper's copies aren't touched.
- **The measure ends about 3,600, not the review's 2,000:** most of what stays in `route.rs` is `ops`'s own routing.
- **`Class` goes to `jig-domain-link`, not skein-lib:** it's the shape of a jig root's admission, and the worker's has
  two variants and temper's root none.
- **`hosted` depends on `jig-domain-brief`,** which `graph.rs` refuses the charter crate today. The design now says the
  charter reads the brief's parts (`jig/docs/design/domain/hosts.md`, 8), so `root-charter` adds it to that one list.
- **`LinkDown::Message` goes to the charter crate,** as `jig/tests/boundary/src/tools.rs` keeps `host` out of
  `jig-protocol-tools`. So `ops`'s root stops rendering words, and the test root keeps its call.
- **`root.md`, 12's module list gains `admission`, `link`, `client_documents` and `steps`:** they're what `ops` is, and
  the document listed fewer.
- **`Now::BriefCorePlanned` and `Now::WorkspaceRequest` stay in the roots,** as `root.md`, 14 says, spelled once.

## Overlaps with other drafts

None to avoid: no listed draft changes `skein-forge-connection`, `skein-lib`'s `list.rs`, `tail.rs` or `env.rs`, or
anything under `jig/`. `repository-copies-1` and `fetch-progress-1` are in other crates. `forge-tail` edits
`skein/tests/forge/github/src/runner.rs` and `skein/tests/fake-peers`, which no listed task touches.

Inside this plan, `forge-tail`, `forge-entry` and `root-timers` each add to `skein-lib`, in different files, so
they run at once. Status entries that would meet on one line stay out of `lib.status.md` for that reason: both
additions are named in `lib.md` and land with this plan.
