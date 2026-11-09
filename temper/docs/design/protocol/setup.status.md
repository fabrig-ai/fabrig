# setup: status

2026-11-09. Where temper's code lags or departs from [setup.md](setup.md).

Sections 6.6 and 11's shared fetch and launcher are present:
`temper-fetch` owns the machine and Claude pin, `temper-fetch-shell`
the TLS host and Claude Code's fetch under its lock, with no smith
crate, and `temper-launcher` the launcher, the placing of its links in
a private directory through `smith-agent-document`, and the
substitution. Forgejo's pins and bottle extraction stay with forge
setup, whose old fetch paths are re-exports. The pin/progress tests in
`temper-fetch`, the substitution's in `temper-launcher`, the retained
local-TLS fetch suite and the agent's launcher rows cover the split; the
process scan exempts `temper-launcher/src/lib.rs` for the launcher's
exec and process id. Neither forge-setup crate is in the agent's normal
closure, and the forge's setup links no launcher, so `temper-engine`'s
closure row holds (`README.status.md`, 4.1). Section 6.6's `claude`
link names the worker's agent program, the box's `temper` or a
deployment's `temper-agent`, both at worker start and at program login
(`temper-worker-shell/src/worker.rs`, `sources`;
`temper-worker-shell/src/program_login.rs`, `command`). Section 15's
real Claude download walk and other deferred work remain unchanged.

Section 8.3 has no known implementation gap in the binary supplying
every project's rule 1 and no deployment rule, or supplying stock
charter 5 and template 4 with their delegation. The documents carry none
of those settings. Applying the reference's combined project settings
in a walk remains unobserved (`configuration.status.md`, 13.9);
section 15 keeps that after-landing work.

Section 7.2's sign-in block names the TUI command at the link's origin,
on stderr in the first block alone. Serve keeps the engine's private
link file at each `LocalLink`, never a printed link, and removes it when its loop
returns (`../client/README.md`, 6.12). No known implementation gap
remains in those additions; a forced kill can leave a stale file as
designed. Section 13.4's box stories
(`crates/temper/tests/suite/box_listeners.rs`) hold the file's link at stdout's address and apart from every printed
link, and left as it was when the first printed link is spent.

Section 7.6 has no known implementation gap in the TUI in front, by the
look request `tui-box-front`: the front chosen once at start, serve's 64
bytes, the front's descriptors and `tui.log`, the terminal's lines to
`box.log` and the front while it runs, the 256 lines held, its end the
box's stop, a stop signalling it first, a give up leaving it running,
and the foreground taken back after it. `temper-box`'s host and line
tables hold each but the last (`temper-box/src/tests/host.rs`,
`front.rs`), and the focused binary story
`a_box_in_a_terminal_runs_its_tui_in_front_and_stops_with_it`
(`crates/temper/tests/suite/box_front.rs`) a box on a pseudo-terminal
signing its front in, the front's raw modes, and its stop with no
secret in a log. The foreground taken back
(`crates/temper/src/box_command.rs`) is held by skein's world for the
call alone (`skein/docs/design/shell.status.md`); no temper test reads the
terminal's foreground group after the front. Every other box story starts
the box on pipes, so `--no-tui`'s and a non-terminal's box are 7.2's as
before. No real terminal has run it; that check follows landing
(`../client/README.md`, section 11), and section 15 keeps a box started
in the background and a front killed outright.

- **Not yet:** 13.4: the file's secret on no log: the fresh box's story
  reads stdout, stderr and `box.log` for it, not `serve.log`, which
  holds serve's stderr. By reading, serve writes none there: its host
  keeps each local link in the file and prints none
  (`temper-service-shell/src/host.rs`).

Sections 5.2, 5.3 and 17.2 have the identity handoff: an owned Forgejo's
given values include its person's login, managed and fake entries give
none, and local reconcile answers become `Is` or `Gone`. Convert, paste
and GitHub reconcile compare the kept `AppMade` and `AppId` and the App's
account type before sending `PersonOf`; absent or invalid evidence is
`NotTheirs`. Failed reads send no person answer. The given-value and
lifecycle tables cover these decisions; they establish no live GitHub
account observation (`skein/docs/design/forge-github.status.md`, 13).
The connector's remaining scope gap is in `../domain/forge.status.md`, 4.6.

Sections 8.2 and 8.3 have no known gap in generated documents holding
providers, endpoints, sources and endpoint limits, with models supplied
as binary defaults. Provider discovery remains section 15's later work.

Sections 8.1 and 8.3 have no known gap in four worker slots and the
workspace and git-operation limits that rise with them. The bare document
and owned example agree with the default. The process fixture explicitly
keeps one slot when lowering its workspace and git-operation limits to
one; no runtime result for that story is supplied here. The four-agent
laptop memory check is deferred in section 15.

Section 7.5 has the new-sandbox nine-port override, occupied-port
refusals and retention of the saved ports when given another valid
choice. Its running-box restart proof remains after landing (15).

- **Departs:** 7.5: a malformed `TEMPER_FAKE_BOX_PORTS` refuses even a
  retained sandbox, where the section says it keeps its ports whatever
  the variable says. `crates/temper/src/fake_backends.rs`, `run`, parses
  the variable before `prepare` checks for a retained sandbox. The
  retained path ignores a parsed choice, but never receives an invalid
  one; no reason for this limit is recorded.

Sections 6.2 and 6.3 have no known implementation gap in the hour-long
owned session or the later preparation's best-effort source update.
Section 6.5 has no known implementation gap in taking a signed-in box's
late handover before connecting: the restored forge reads again after its
first reconcile, takes and removes a present file, and reconciles again;
an absent file needs no second reconcile. The service's setup-lifecycle
tests cover both answers and the single read without sign-in.
Sections 7.1 and 7.2 have no known implementation gap in an ordered hangup
stop, ignored hangups, or Linux's parent-death wrapper. macOS's lack of
a parent-death signal is the designed limit, deferred in section 15.

Section 7.2's orderly serve stop cancels the worker's runs and waits for
every agent process, including a spawn in flight, before closing the
worker. The real-loop held-Claude-turn test checks the program's interrupt,
its end and serve's drained end. No known implementation gap remains in
that ordering. Programs surviving a forced agent-group kill remain
section 15's contained-tree work.

- **Not yet:** 7.2, *Its children*: the owned-forge Claude Code landing
  story (`crates/temper/tests/live.rs`,
  `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge`) has no
  supplied live rerun establishing no surviving descendant after the
  orderly-stop fix. The held-turn test does not establish that live
  outcome. The live judge retains its no-survivor assertion and reports
  each survivor's command line, parent, group and session, with the
  parent's and group leader's rows at stop, through the log's filter.

- **Not yet:** 13.4: the hangup/rebind and Linux killed-box stories in
  `crates/temper/tests/suite/box_lifetime.rs` are present in the process
  binary and compiled, with their assertions retained. Their process-tier
  runtime is unobserved in `limits-heavy`; both passed before the move in
  the focused suite. The original 15.9 CPU s estimate transfers to the
  process tier, whose current total and remaining margin are unknown
  (`../domain/testing.status.md`, 2.6). The ignored-hangup path remains
  focused in `crates/temper/tests/suite/box_listeners/lifetime.rs`.

- **Not yet:** 13.4: the supplied inventory contains no run results for
  `ui-restart-red`'s five repaired process stories: the CSV landing, the
  playground copy's source read, the person's plan change, the person's
  change and the setup/removal cuts. Their requested all-five pass is
  unverified here; landed parts and focused test code do not establish
  those outcomes. The started cut's fixture disables sign-in alone,
  restoring it before the final setup and removal cut (13.4).

- **Not yet:** 7.1, 7.2, 13.4: the `ui-repo-link-2` request's combined
  1440-pixel walk with the default runner, a restart, a moved temper port
  and a pseudo-terminal hangup is not established by the named proof
  (`../../development/ui-repo-link-review.md`): that proof disables the
  runner and covers neither the port move nor the hangup. The supplied
  inventory has no result for this combined walk, so its outcome is
  unknown. The separate source-update and lifetime tests do not establish
  the complete sequence. The real-box sign-in story's runtime check is
  still pending as `sign-in.status.md`, 9.9 records.

- **Not yet:** 8.2, 13.4: a first `temper box` from a binary built without its browser says nothing of it: `crates/temper/src/init.rs`, `run`, alone writes init's line for no browser, and the box's first run calls `core` (`crates/temper/src/box_command.rs`, `configuration`), so of init's words `box.log` holds `First run:` alone, as 7.2 lists them, where 13.4 has that line among them. 6.8: for an owned Forgejo, the printed command has neither its binary nor its `--work-path` and `--config`: `temper-engine-shell/src/forge_sign_in.rs` gives `sign_in::instructions` the program `forgejo` and no place, whatever the forge's kind, so the command `temper help server`'s step 1 has a person run works only with those supplied, as the process tier's story supplies them (`crates/temper/tests/support/forgejo_sign_in.rs`, `owned_command`), and the page does not say so. 2: a connected Forgejo made managed by "Connect as its admin": `temper-engine-domain-forge/src/connect.rs`, `admin`, refuses it *not taken*, a kept forge's row offering `admin` on a managed forge alone (`forges.rs`, `admin_connect`), as 5.5's first point and `../client/README.md`, 4.6 say, where section 2, 5.3's given bot and 5.5's bot take the connected user over (`../draft/github.md`, 12.2). 6.5, 8: "New repository" on a connected owned or managed forge's card: the client offers no such action. 7.2: an owned forge's hands name the person's login alone, not the paths of their read and write header files (`crates/temper/src/box_command.rs`, `owned_notices`). 7.2: a fetch line it can read from a group started already, one running its fetch again, is named and dropped but not kept from the terminal: `temper-box/src/host.rs`, `children`, marks a line for the log alone only when it relays it or its step has exited, so this one shows under `--echo steps` and in a failure's tail. 7.2, *On a terminal*: no run on record has a real `temper box` on a real terminal, where the request `box-small-2` of 2026-11-06 has the user try it, and emacs vterm's ignoring the sequence is that request's word. The form's rule and bytes are held on the simulated host (`temper-box/src/tests/terminal.rs`, `temper-box/src/tests/host.rs`), the choice on real descriptors, `/dev/ptmx` among them (`crates/temper/src/tests/box_link_form.rs`), and the bare form on a real pipe (`crates/temper/tests/suite/box_listeners.rs`), so `&std::io::stdout()` in `run_configuration` (`crates/temper/src/box_command.rs`) is the one line no test holds. 8.4: git's smart HTTP on the fake forge. 8.4: a real forge ended by its removal still refuses `StartPlayground` until its record is erased (`temper-engine-domain-forge/src/forges.rs`, `real_forge_kept`, reads every record, ended ones among them), where the welcome offers the playground once the forge leaves the `forges` rows; the record stays while any kept task names one of its repositories (`../domain/forge.md`, 4.1), so the refusal lasts until every chat that worked on one is erased, with its project while retention is not built (jig's `protocol/store.md`, 11.1). 8.4: the playground's project allows only the scripted model: the scripted account's uses name the project alone (`temper-engine-domain/src/route/playground.rs`, `Reply::Uses`), but nothing limits the project's models to the scripted one, so an account of a real provider the deployment holds may serve it. 13.4: the first run into the playground and out by its reset, and a reset the box's death cut (`crates/temper/tests/support/playground.rs`, `end_and_check` and `interrupt_reset`, and `support/boxed.rs`, `run_ctl_resuming_playground`, have no test); and GitHub gone on after a restart, on the fake backends. 17.1: the App's statuses and actions, write: skein's `skein-forge/src/github/setup.rs` asks both as read. 17.1: a GitHub `Connect` on a clients' listener at port 0 is not refused (`temper-engine-domain-forge/src/connect.rs`, `make`). 5.5, 10: no run on record shows a managed Forgejo's hook delivering: the process story asserts its hook's URL alone (`crates/temper/tests/support/forgejo_elsewhere.rs`, `assert_hook`), and the request `managed-listener` of 2026-11-06 leaves delivery to dev.fabrig.ai's next deployment. 5.6: that request asks that a hook that can't deliver be shown to leave polling working, and `a_managed_forge_whose_hook_delivers_nothing_stays_connected_and_keeps_reading` (`temper-service/src/tests/setup_admin.rs`) holds the forge connected, nothing ended and no read of a hook's deliveries, but for all its name asserts no read. Polling past such a hook stands on `a_watched_pulls_ci_at_the_same_head_wakes_its_change` (`temper/tests/engine/forge/tests/suite/simulation.rs`) alone, polling with no hook heard: the fake-box walks that landed on its Forgejo elsewhere, whose hooks are off (`temper-fake/src/forge.rs`, `hooks: None`), went to lower worlds by the request `walks-lower` of 2026-11-08 (`../domain/testing.md`, 2.6), and the two that stay land nothing. 8.3: the walk the user's request of 2026-11-03 ends with, a fresh box through the reference case's steps 1 to 6 with no override and no settings change beyond the reference's charters and template: its QA rung, `rm3`, was holding on findings in repair on 2026-11-04, so a fresh box's scale stands on its tests alone (`temper/tests/engine/domain/tests/suite/fabrig_scale.rs`, 80 chats and a tree of 384; `crates/temper/tests/suite/startup.rs`, the start) and the measured box below.
- **Measured** (8.3, the resource gate, 2026-11-04, on a development host, dev build): a fresh box at the one-slot init defaults of that date, excluding agent processes: anonymous memory (smaps Anonymous, box and serve summed) idle 12.75904 MB (serve 7.647232 MB, box 5.111808 MB), with one goal running 13.23008 MB, its sampled peak 13.23008 MB, after 80 chats and the playground's change 17.567744 MB, after a restart 33.988608 MB; beside it, summed, RSS idle 100.27008 MB, with one goal running 106.770432 MB, PSS idle 66.407424 MB, with one goal running 53.19168 MB; the store on disk fresh 0.014139 MB, after 80 chats and the playground's change, and a restart, 11.121464 MB (the data directory 11.122122 MB).
- **Kept** (8.1, 8.3; the user's request of 2026-11-03 asks that it be said here): the rest of the machine's concurrency stays as `temper init` set it, sized by no figure of that request: `fleet.workers` 2, `fleet.configured_workers` 8, the clients' streams 8 with their 12 exchanges, and the owned runner's capacity 1, so a box's CI runs one job at a time beside its runs.
- **Departs:** 7.1, 7.3, 13.4: serve writes no group's file: the box link sends `start N owned` or `start N fake`, the box writes `<data>/box/groups/N` holding the kind alone, and the box and `temper forge fetch`, `prepare`, `run` and `remove` compose forge N's setup entry and port from `application.machine` by that kind (`temper-service/src/route/box_link.rs`, `start`; `temper-box/src/pipe.rs`, `Command::StartKind`; `temper-box/src/group_records.rs`; `temper-forge-setup/src/machine.rs`, `group_file` and `entry`; `temper-engine-shell/src/forge.rs`, `entry`). The code records no reason; at most one owned and one playground forge are kept at once, so each kind has the machine's one setup. 8.4: the scripted model's `otherwise` sends the person to Accounts to connect a model, where the banner's "set up temper" is the way out (`temper-init/template/temper.json`, `application.demo.llm.otherwise`). 8.2: init's fallback lines name "forge 3's person" (`crates/temper/src/init.rs`). 5.5, 16: `Connect` carries no bot: every forge's record is made with the bot `temper`, `temper@temper.invalid` (`temper-engine-domain-forge/src/make.rs`, `bot`), so two deployments on one managed forge share a bot login. 7.5: the real loop's `suite::sign_in::a_sign_in_beside_a_browsers_idle_issuer_sockets_completes_and_past_the_cap_is_turned_away` (`tests/real-loop/tests/suite/sign_in.rs`) holds sign-in and turn-away past the cap, and `suite::fake::the_oauth_fake_logs_each_connection_it_turns_away_at_its_cap` (`crates/temper/tests/suite/fake.rs`) holds each rejection's log. The real-loop story opens the browser's idle issuer sockets at the authorize hop, that hop's own socket kept and a spare beside it, where the user's request of 2026-11-05 opens two just before the visit, because on a lowered cap two held before the visit would refuse the visit itself, where a browser's fail the token POST. 13.4: the first link opened at once: `the_first_link_opened_at_once_waits_and_signs_in` (`crates/temper/tests/suite/box_listeners.rs`) drops the connection it opens on reading the link, signs in by `temper open` on a connection of its own, and passes whether or not serve listened yet, saying which on its stderr. So no test holds a connection waiting in the backlog and then signing in, where the request `box-link-first` of 2026-11-05 has a browser's early connection do so. Its plan's cut took being early as the machine's luck: what the tests hold is that the socket accepts from the block on, and the block before any spawn (`temper-box/src/tests/host.rs`, `the_block_is_the_terminal_before_any_spawn`).
- **Design departs:** 7.2, from the request `ui-welcome` of 2026-11-05, decided by its plan's cut: an owned forge's hands go to its Forgejo step's own log, `forgejo-N.log`, where the request has the box's log: the box had no log of its own when that plan was cut. `box.log` now exists (7.2), and the hands stay: no request asks to move them. 6.3, from the request `runner-labels` of 2026-11-05, decided by its plan's cut: a preparation keeps the row when `runner/config.yaml` names a `uuid`, where the request has it keep a declared runner's labels and version when its uuid matches: no Forgejo runs under the preparation's lock and its command line lists no runners, so no row is read or compared, and a `runner/secret` replaced by hand beside a kept configuration does not reach the row.
- **Design departs:** 8.1, from the request `first-goal-fixes` of 2026-11-05, item 4, decided by its plan's cut: `documents.text_bytes` is 512 in `temper init`'s documents, where the request has jig standard's 8,192 unless that breaks a bound a start checks. A start takes 8,192 and compares it with none of the web and terminal clients' bounds (section 15), and they refuse longer document text, which `crates/temper/src/tests/init_clients.rs` holds for init's documents. So a client reads a task's spec in pieces of 512 bytes (section 16). The playground keeps 128 (`temper-playground/src/documents.rs`, `stories/a/deployment.json`), where the request keeps it only if its size needs it, and whether it does is unknown: no document records it.

Section 8.3's printed link and approval are enabled in both init
documents and the owned example. The service table checks that the
approval setting reaches an ask. The TUI's browser flow and recorded
approval journey are present. Its fresh-box terminal check remains
later work in section 15 and `../client/README.md`, section 11.
