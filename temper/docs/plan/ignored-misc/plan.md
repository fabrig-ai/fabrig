# ignored-misc: the plan

Six of temper's ignored tests each name a gap in the product
(`inputs/request.md`). This plan fixes each gap at its cause, by the
design its reason cites, and the tests run again. Four tasks, as the
request numbers them, each its own part and slice, none needing another:
all four start at once. Effort medium, as the request says.
`notes/common.md` holds the rules every task keeps.

The cut wrote the design first:

- `jig/docs/design/domain/engine.md`, 17.5, and `connectors.md`,
  section 17: at each start a connector also names, by kind, scope and
  number, each default it stored that the application no longer gives;
  only a setting following one of those is removed in the release's
  revision. A setting following a default no connector reports or
  names, as after a reset to a default the application gives for no
  one scope or not at all, keeps its state and slot.
- `jig/docs/design/domain/engine.md`, section 8 (*A draining worker
  leaves placement*), `hosts.md`, 6.7, and `../protocol/channel.md`,
  5.1 (the `draining` notice) and 5.3 (its order).
- `temper/docs/design/protocol/configuration.md`, section 2 (*A
  sign-in's hosts*): a sign-in's host that does not resolve at start is
  held at `127.0.0.1:0`, as a held forge's, and the start goes on.
- `temper/docs/design/domain/testing.md`, 5.3's 4.6 row: the process
  story runs, ws saying it drains.
- **Not yet** entries the tasks close: jig's `engine.status.md` (17.5,
  reworded to the chosen contract; 8), `connectors.status.md` (17),
  `hosts.status.md` (6.7), `channel.status.md` (5.1, 5.3); temper's
  `configuration.status.md` (2) and `testing.status.md` (2.6, the
  no-DNS sentence moved to a bullet of its own so that two tasks don't
  edit one paragraph).

## Parts and tasks

| Part | Task | What | Needs | Effort | Minutes |
|---|---|---|---|---|---|
| `restore-kept` | `restore-kept` | jig's `SettingsDefaults` carries `dropped`; `prepare_start` erases only a state following a dropped default; `jig-test-connector` and temper's forge name theirs; the engine-domain and client restore stories lose `#[ignore]` | | medium | 150 |
| `story-a-walks` | `story-a-walks` | `temper_playground::story::prepare` lays a story's disk and handovers for both worlds; story A walks in the one-process world; its memory test bound at figure + 25% | | medium | 120 |
| `sign-in-dns` | `sign-in-dns` | `accounts_runtime::resolve` takes its lookup and holds an unresolved sign-in host at `127.0.0.1:0`; the no-DNS box story runs by name | | medium | 60 |
| `drain-placement` | `drain-placement` | `LinkUp::Draining` from the hub at shutdown; worker wire kind `0x010b`; both translations; the fleet's sticky mark; the 4.6 cluster story runs by name | | medium | 180 |

The critical path is `drain-placement`, the widest: about 180
implementer-minutes, with all four started together.

## Decisions

- **Restore: a connector contract, not a stored format.** The core's
  stored state can't tell a reset to an absent default from an empty
  default a release dropped (`engine.status.md` said so: a stored-format
  or connector-contract change). A stored format is the user's to
  change; the connector already stores the defaults it was last given,
  so naming the dropped ones at the start it already reports in needs no
  new record. jig's test connector reported a dropped default as a
  zero-sized one; it names it instead. One fix site in jig, both
  stories.
- **Story A: the world's wiring first.** The survey found the one-process
  world calls `machine::setup` alone, where the playground world also
  lays the deployment's handovers, among them the playground forge's
  admin token; without it the forge never connects and the client polls
  Starting at 0 ns. The task shares one `prepare` between the worlds and
  follows any further cause, with a pre-approved fallback: a cause that
  needs a design section that doesn't exist keeps both tests ignored,
  their reasons naming it (the request's rule).
- **Story A's memory: ignored for a missing skein mechanism.** The
  walk lands; its memory test fails skein's exact release check because
  the world's observer, inside the service's span, fills the referee's
  shared store. skein keeps harness storage outside process spans
  (`skein/docs/design/testing.md`, 5) but gives a service world's
  observer no way to do so, and no design section names one: larger
  than this task, so the memory test keeps its `#[ignore]`, its reason
  naming that, until a skein plan adds it (answer of 2026-11-08).
- **No-DNS: held at start, not resolved at use.** skein's accounts
  driver takes addresses resolved before its loop and resolves nothing
  while it runs (skein's `oauth.md`, 6.2); io has no DNS client. So
  resolution can't move to first use, and the design takes the other
  branch the request offers: a failed lookup refuses no start, as a
  held forge's doesn't (`setup.md`, 8.3). `sign-in.md`, 3.1 is Google's
  sign-in, which still refuses; this story's registration is the LLM
  provider's (`engine.llm`), so its rule went to `configuration.md`,
  section 2.
- **Drain: a new notice on worker wire version 9.** No drain-start
  signal existed; the hub closed admission on its own side. Kinds are
  added within version 9 (`read list` and `list read`, 2026-11-07), and
  an engine and its workers are released together
  (`channel.md`, 4.1), so this is no protocol change for the user. The
  fleet's busy mark lifts as a slot frees; the drain's mark stays for
  the channel's life. An attempt chosen for the channel before the
  notice is placed again unrefused, so the race that journaled the
  extra attempt has no other path.
- **Two tasks cross trees.** A branch stays in its tree, but the request
  fixes four tasks, and two gaps sit on both sides of the jig/temper
  boundary: `restore-kept` adds a field to jig's event, which temper's
  root fills from a small new forge method; `drain-placement` adds a
  `LinkUp` variant, whose temper readers gain arms. Each temper side is
  small, named in the spec's file list, and proved by the temper story
  the task unignores. Splitting each would have made six tasks.
- **Process stories by name, alone.** The two process stories run in
  `process-temper` by their exact names, each once, as the request
  allows for repaired findings; nothing else of the tier runs. The
  drain story failed 3 in 85 runs under load, so one pass shows it
  runs; the fleet's unit tests hold the rule.

## Overlaps with other drafts

- `temper/docs/design/protocol/configuration.status.md`: `restore-kept`
  removes sentences from its long **Not yet** paragraph (one line),
  which `ui-reaches`' unmerged `ways-away` lists too. A merge there may
  conflict; the line is one paragraph, so no boundary avoids it.
  `sign-in-dns` edits only a bullet at the file's end.
- `temper/docs/design/client/tools.status.md` (`restore-kept`) is listed
  by `ui-reaches/ways-away` too.
- No other draft lists the code files these tasks own. `story-a-walks`
  avoids `temper-client-domain/src/forge.rs` and `forge/ways.rs`, which
  `ui-reaches` changes; its fallback in the product names
  `forge/playground.rs` alone.
