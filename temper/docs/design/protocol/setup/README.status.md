# setup: status

2026-11-10. Where temper's code lags or departs from [README.md](README.md).

`owned.md`, section 6, and `README.md`, section 7's shared fetch and launcher are present:
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
closure row holds (`../binaries.status.md`, 1). `owned.md`, section 6's `claude`
link names the worker's agent program, the box's `temper-box` or a
deployment's `temper-agent`, both at worker start and at program login
(`temper-worker-shell/src/worker.rs`, `sources`;
`temper-worker-shell/src/program_login.rs`, `command`). `README.md`, section 9's
real Claude download walk and other deferred work remain unchanged.

`init.md`, section 3 has no known implementation gap in the binary supplying
every project's rule 1 and no deployment rule, or supplying stock
charter 5 and template 4 with their delegation. The documents carry none
of those settings. Applying the reference's combined project settings
in a walk remains unobserved (`../configuration/reference.status.md`);
`README.md`, section 9 keeps that after-landing work.

`box.md`, section 2's sign-in block names the TUI command at the link's origin,
on stderr in the first block alone. Serve keeps the engine's private
link file at each `LocalLink`, never a printed link, and removes it when its loop
returns (`../../client/tui/README.md`, 2). No known implementation gap
remains in those additions; a forced kill can leave a stale file as
designed. `testing.md`, section 4's box stories
(`bin/temper-box/tests/suite/box_listeners.rs`) hold the file's link at stdout's address and apart from every printed
link, and left as it was when the first printed link is spent.

`box.md`, section 6 has no known implementation gap in the TUI in front, by the
look request `tui-box-front`: the front chosen once at start, serve's 64
bytes, the front's descriptors and `tui.log`, the terminal's lines to
`box.log` and the front while it runs, the 256 lines held, its end the
box's stop, a stop signalling it first, a give up leaving it running,
and the foreground taken back after it. `temper-box-machine`'s host and line
tables hold each but the last (`temper-box-machine/src/tests/host.rs`,
`front.rs`), and the focused binary story
`a_box_in_a_terminal_runs_its_tui_in_front_and_stops_with_it`
(`bin/temper-box/tests/suite/box_front.rs`) a box on a pseudo-terminal
signing its front in, the front's raw modes, and its stop with no
secret in a log. The foreground taken back
(`crates/temper-box-shell/src/box_command.rs`) is held by skein's world for the
call alone (`skein/docs/design/runtime/shell.status.md`); no temper test reads the
terminal's foreground group after the front. Every other box story starts
the box on pipes, so `--no-tui`'s and a non-terminal's box are `box.md`, 2's as
before. No real terminal has run it; that check follows landing
(`../../client/README.md`, section 5), and `README.md`, section 9 keeps a box started
in the background and a front killed outright.

- **Not yet:** `box.md`, 2, *Its children*: the owned-forge Claude Code landing
  story (`bin/temper-box/tests/live.rs`,
  `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge`) has no
  supplied live rerun establishing no surviving descendant after the
  orderly-stop fix. The held-turn test does not establish that live
  outcome. The live judge retains its no-survivor assertion and reports
  each survivor's command line, parent, group and session, with the
  parent's and group leader's rows at stop, through the log's filter.

- **Measured** (`init.md`, 3, the resource gate, 2026-11-04, on a development host, dev build): a fresh box at the one-slot init defaults of that date, excluding agent processes: anonymous memory (smaps Anonymous, box and serve summed) idle 12.75904 MB (serve 7.647232 MB, box 5.111808 MB), with one goal running 13.23008 MB, its sampled peak 13.23008 MB, after 80 chats and the playground's change 17.567744 MB, after a restart 33.988608 MB; beside it, summed, RSS idle 100.27008 MB, with one goal running 106.770432 MB, PSS idle 66.407424 MB, with one goal running 53.19168 MB; the store on disk fresh 0.014139 MB, after 80 chats and the playground's change, and a restart, 11.121464 MB (the data directory 11.122122 MB).
- **Kept** (`init.md`, 1, 3; the user's request of 2026-11-03 asks that it be said here): the rest of the machine's concurrency stays as `temper-box init` set it, sized by no figure of that request: `fleet.workers` 2, `fleet.configured_workers` 8, the clients' streams 8 with their 12 exchanges, and the owned runner's capacity 1, so a box's CI runs one job at a time beside its runs.
- **Design departs:** `box.md`, 2, from the request `ui-welcome` of 2026-11-05, decided by its plan's cut: an owned forge's hands go to its Forgejo step's own log, `forgejo-N.log`, where the request has the box's log: the box had no log of its own when that plan was cut. `box.log` now exists (`box.md`, 2), and the hands stay: no request asks to move them. `owned.md`, 3, from the request `runner-labels` of 2026-11-05, decided by its plan's cut: a preparation keeps the row when `runner/config.yaml` names a `uuid`, where the request has it keep a declared runner's labels and version when its uuid matches: no Forgejo runs under the preparation's lock and its command line lists no runners, so no row is read or compared, and a `runner/secret` replaced by hand beside a kept configuration does not reach the row.
- **Design departs:** `init.md`, 1, from the request `first-goal-fixes` of 2026-11-05, item 4, decided by its plan's cut: `documents.text_bytes` is 512 in `temper-box init`'s documents, where the request has jig standard's 8,192 unless that breaks a bound a start checks. A start takes 8,192 and compares it with none of the web and terminal clients' bounds (section 9), and they refuse longer document text, which `crates/temper-box-shell/src/tests/init_clients.rs` holds for init's documents. So a client reads a task's spec in pieces of 512 bytes (section 10). The playground keeps 128 (`temper-playground/src/documents.rs`, `stories/a/deployment.json`), where the request keeps it only if its size needs it, and whether it does is unknown: no document records it.

`init.md`, section 3's printed link and approval are enabled in both init
documents and the owned example. The service table checks that the
approval setting reaches an ask. The TUI's browser flow and recorded
approval journey are present. Its fresh-box terminal check remains
later work in `README.md`, section 9 and `../../client/README.md`, section 5.
- **Not yet:**  a first `temper box` from a binary built without its browser says nothing of it: `crates/temper-box-shell/src/init.rs`, `run`, alone writes init's line for no browser, and the box's first run calls `core` (`crates/temper-box-shell/src/box_command.rs`, `configuration`), so of init's words `box.log` holds `First run:` alone, as `box.md`, 2 lists them, where `testing.md`, 4 has that line among them.  for an owned Forgejo, the printed command has neither its binary nor its `--work-path` and `--config`: `temper-engine-shell/src/forge_sign_in.rs` gives `sign_in::instructions` the program `forgejo` and no place, whatever the forge's kind, so the command `temper help server`'s step 1 has a person run works only with those supplied, as the process tier's story supplies them (`bin/temper-box/tests/support/forgejo_sign_in.rs`, `owned_command`), and the page does not say so. 2: a connected Forgejo made managed by "Connect as its admin": `temper-engine-domain-forge/src/connect.rs`, `admin`, refuses it *not taken*, a kept forge's row offering `admin` on a managed forge alone (`forges.rs`, `admin_connect`), as `reconcile.md`, 5's first point and `../../client/forge/records.md` say, where `README.md`, section 2, `reconcile.md`, 3's given bot and `reconcile.md`, 5's bot take the connected user over (`../../draft/github.md`, 12.2).  "New repository" on a connected owned or managed forge's card: the client offers no such action.  an owned forge's hands name the person's login alone, not the paths of their read and write header files (`crates/temper-box-shell/src/box_command.rs`, `owned_notices`).  a fetch line it can read from a group started already, one running its fetch again, is named and dropped but not kept from the terminal: `temper-box-machine/src/host.rs`, `children`, marks a line for the log alone only when it relays it or its step has exited, so this one shows under `--echo steps` and in a failure's tail. `box.md`, 2, *On a terminal*: no run on record has a real `temper box` on a real terminal, where the request `box-small-2` of 2026-11-06 has the user try it, and emacs vterm's ignoring the sequence is that request's word. The form's rule and bytes are held on the simulated host (`temper-box-machine/src/tests/terminal.rs`, `temper-box-machine/src/tests/host.rs`), the choice on real descriptors, `/dev/ptmx` among them (`crates/temper-box-shell/src/tests/box_link_form.rs`), and the bare form on a real pipe (`bin/temper-box/tests/suite/box_listeners.rs`), so `&std::io::stdout()` in `run_configuration` (`crates/temper-box-shell/src/box_command.rs`) is the one line no test holds.  git's smart HTTP on the fake forge.  a real forge ended by its removal still refuses `StartPlayground` until its record is erased (`temper-engine-domain-forge/src/forges.rs`, `real_forge_kept`, reads every record, ended ones among them), where the welcome offers the playground once the forge leaves the `forges` rows; the record stays while any kept task names one of its repositories (`../../domain/forge/adoption.md`, 1), so the refusal lasts until every chat that worked on one is erased, with its project while retention is not built (jig's `protocol/store/README.md`, 6.1).  the playground's project allows only the scripted model: the scripted account's uses name the project alone (`temper-engine-domain/src/route/playground.rs`, `Reply::Uses`), but nothing limits the project's models to the scripted one, so an account of a real provider the deployment holds may serve it.  the first run into the playground and out by its reset, and a reset the box's death cut (`bin/temper-box/tests/support/playground.rs`, `end_and_check` and `interrupt_reset`, and `support/boxed.rs`, `run_ctl_resuming_playground`, have no test); and GitHub gone on after a restart, on the fake backends.  the App's statuses and actions, write: skein's `skein-forge/src/github/setup.rs` asks both as read.  a GitHub `Connect` on a clients' listener at port 0 is not refused (`temper-engine-domain-forge/src/connect.rs`, `make`). 6: no run on record shows a managed Forgejo's hook delivering: the process story asserts its hook's URL alone (`bin/temper-box/tests/support/forgejo_elsewhere.rs`, `assert_hook`), and the request `managed-listener` of 2026-11-06 leaves delivery to dev.fabrig.ai's next deployment.  that request asks that a hook that can't deliver be shown to leave polling working, and `a_managed_forge_whose_hook_delivers_nothing_stays_connected_and_keeps_reading` (`temper-service/src/tests/setup_admin.rs`) holds the forge connected, nothing ended and no read of a hook's deliveries, but for all its name asserts no read. Polling past such a hook stands on `a_watched_pulls_ci_at_the_same_head_wakes_its_change` (`temper/tests/engine/forge/tests/suite/simulation.rs`) alone, polling with no hook heard: the fake-box walks that landed on its Forgejo elsewhere, whose hooks are off (`temper-fake/src/forge.rs`, `hooks: None`), went to lower worlds by the request `walks-lower` of 2026-11-08 (`../../testing/process-tier.md`), and the two that stay land nothing.  the walk the user's request of 2026-11-03 ends with, a fresh box through the reference case's steps 1 to 6 with no override and no settings change beyond the reference's charters and template: its QA rung, `rm3`, was holding on findings in repair on 2026-11-04, so a fresh box's scale stands on its tests alone (`temper/tests/engine/domain/tests/suite/fabrig_scale.rs`, 80 chats and a tree of 384; `bin/temper-box/tests/suite/startup.rs`, the start) and the measured box below.
- **Departs:**  serve writes no group's file: the box link sends `start N owned` or `start N fake`, the box writes `<data>/box/groups/N` holding the kind alone, and the box and `temper-box forge fetch`, `prepare`, `run` and `remove` compose forge N's setup entry and port from `application.machine` by that kind (`temper-service/src/route/box_link.rs`, `start`; `temper-box-machine/src/pipe.rs`, `Command::StartKind`; `temper-box-machine/src/group_records.rs`; `temper-forge-setup/src/machine.rs`, `group_file` and `entry`; `temper-engine-shell/src/forge.rs`, `entry`). The code records no reason; at most one owned and one playground forge are kept at once, so each kind has the machine's one setup.  the scripted model's `otherwise` sends the person to Accounts to connect a model, where the banner's "set up temper" is the way out (`temper-init/template/temper.json`, `application.demo.llm.otherwise`).  init's fallback lines name "forge 3's person" (`crates/temper-box-shell/src/init.rs`). 10: `Connect` carries no bot: every forge's record is made with the bot `temper`, `temper@temper.invalid` (`temper-engine-domain-forge/src/make.rs`, `bot`), so two deployments on one managed forge share a bot login.  the real loop's `suite::sign_in::a_sign_in_beside_a_browsers_idle_issuer_sockets_completes_and_past_the_cap_is_turned_away` (`tests/real-loop/tests/suite/sign_in.rs`) holds sign-in and turn-away past the cap, and `suite::fake::the_oauth_fake_logs_each_connection_it_turns_away_at_its_cap` (`bin/temper-box/tests/suite/fake.rs`) holds each rejection's log. The real-loop story opens the browser's idle issuer sockets at the authorize hop, that hop's own socket kept and a spare beside it, where the user's request of 2026-11-05 opens two just before the visit, because on a lowered cap two held before the visit would refuse the visit itself, where a browser's fail the token POST.  the first link opened at once: `the_first_link_opened_at_once_waits_and_signs_in` (`bin/temper-box/tests/suite/box_listeners.rs`) drops the connection it opens on reading the link, signs in by `temper-box open` on a connection of its own, and passes whether or not serve listened yet, saying which on its stderr. So no test holds a connection waiting in the backlog and then signing in, where the request `box-link-first` of 2026-11-05 has a browser's early connection do so. Its plan's cut took being early as the machine's luck: what the tests hold is that the socket accepts from the block on, and the block before any spawn (`temper-box-machine/src/tests/host.rs`, `the_block_is_the_terminal_before_any_spawn`).
