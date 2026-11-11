# records: status

2026-11-11. Where the tree's code lags or departs from [records.md](records.md).

- **Not yet:** the whole document: the first-usable-model real-box walk remains `../pages/accounts.status.md`, 2.
- **Not yet:** the subject `approvals`: `ApprovalRow` has no `signed`,
  `can_sign` or `can_sign_more`, and the schema no `ApprovalPerson`
  [in flight: change 847].
- **Not yet:** 
  the Board address and reordering remains `../engine.status.md` (Not yet).
   the project-only settings-role gap remains `../pages/settings.status.md`, 2.
   the Home settings-tab links remain `../pages/settings.status.md`, 2.
  `../pages/README.md`, *Goals*: the empty Goals-page gap remains `../pages/work.status.md`, 1.
  the whole document: Forget, on the Forges page and as the FORGE
  request `Forget`: no `forget` in `temper-client-protocol-wire`'s schema
  or in `temper-service`'s routing.
  `../pages/README.md`, *Ready*: the unhandled Ready endings remain `../pages/welcome.status.md`, 1.
   the plan's first-use card remains `../pages/accounts.status.md`, 2.
  the project's Delete act remains `../pages/settings.status.md`, 1. `../pages/README.md`, The welcome:
  the welcome role-scope gap remains `../pages/welcome.status.md`, 1.
  FORGE in "FORGE · not
  finished" as the forge's name; `forge/connect.rs` `unfinished` prints
  "FORGE" itself. "Go on" after a restart: the row's `app` is the forge
  domain's memory alone (`temper-engine-domain-forge`'s `domain.rs`
  `apps`), so a GitHub forge found unfinished after a restart offers
  "Start over" alone.
   the App-not-installed reason remains `family.status.md`, 6.
- **Departs:** `../pages/README.md`, *The first chat*, and `../view.md`, 3: the rail's "Ask temper", the
  tab bar's "Ask" and To begin's "Start" choose among the rows
  `temper-client-domain`'s `Chats::open_asking` is delivered, whose
  live watch holds a window of `limits.client.domain.rows` by latest
  activity, so an Ask temper chat not closed but past that window isn't
  seen and they open `Ask { project }`; before the chats' rows arrive, the rail's link opens the page before its first words; the landings of `configure-temper-name`
  and `ask-temper-place` accepted it, raising no bound for one link.
  `../pages/README.md`, `records.md`, Changes: the queue's first says "checking" at whichever step of
  its turn, and "checks started" gives when that step began
  (`temper-engine-domain-forge`'s `views/changes.rs`, `Change.since`), where
  the request of 2026-11-05 has when its checks began: nothing keeps that time.
  the whole document: `changes_view` reads every `ChangeRow`'s repository with `?`, and no
  step erases a closed change's row (`../../domain/after-landing.status.md`), so
  once a forge's removal erases a repository with a closed change, no project
  has the document; seen by reading, no test holds it and no reason is recorded.
  `../pages/README.md`, *Goals*: `goals::row` is `pub(crate)` (`temper-client-view`'s
  `pages/goals.rs`), where this section has it public, because its `Bindings`
  is the view's own and Home reads it in the same crate.
  `../pages/README.md`, *Needs you*: a filter's `kind` and `goal` narrow the open cards alone:
  `temper-client-view`'s `pages/needs.rs`, `main`, asks `Session::answered` by
  project, and an `Answered` keeps neither, so under such a filter the answered
  lines and the head's count hold answers the filter leaves out; no reason is
  recorded.
  `../pages/README.md`, *A new chat* and *A project with no repository*: `NewChat`'s line says
  the project has no repository for every `Repository` but `In`
  (`temper-client-view`'s `pages/new_chat.rs`, `main`), so also with the
  `forges` watch refused, ended or not delivered yet, where the section has
  nothing said. The request of 2026-11-05 gives that line two cases and no
  words for not knowing; no test holds it and no reason is recorded.
  `../pages/README.md`, *The rows*: an account's sources are a list under their level-3
  heading, each with its state and acts, and a sign-in under way a paragraph
  (`temper-client-view`'s `pages/accounts/rows.rs`, `draw`), where the section
  has a row's facts as `.setting` rows, because the rows keep their sources,
  observations and forms in document order.

The document and `../pages/settings.md`, section 2 and `../pages/chat/conversation.md`, section 3 have no known gap. The wire carries
`BranchPath`, the subject `ways` with its `WaysPanel` and the read `View`;
Ask temper's page watches its project's `ways` while open, and its domain
sends "Use this" and "Keep it" as one `ChangeSettings` of the read's items,
with the project's policy joined where a way's `delegation` holds, offers
"Yours" where no way is current, no act where `may_change` is false, and
"Keep it" only while To begin's first step is left. A turn's and a chat
document's top-level `view` blocks are split out, read through `View` and
redrawn at a newer revision, drawn as temper's path drawing kind with a
stop a line as the terminal's fallback. `PathStep` holds `design` and
`close_out`, a goal's coordinator's, and the wire a `TemplateGoal`;
clients draw those stops by their words, as every other, and no gallery
state draws either.

The document and `../pages/settings.md`, section 2 and `../pages/chat/conversation.md`, section 3 have no known gap but the departure below
in `look-round-25`'s repairs of `ask-temper-ways` and of How work runs:
the wire's `PathStep` holds `conflict` and `catch_up` and no `update`,
its `Stop` `needed` and `over`, and `View` `roles`, a card a role,
which a reply draws; a stop made only when needed is drawn outlined, its
small words over its own, and the fallback says ", when needed" and the
small words in brackets; who acts is the mark alone, an agent's star
and its rises in ink, a person's ring and its rises the signal's,
code's square grey, each rise a fade of skein's (skein's `ui/scenes.md`,
2), the legend with an agent when needed; and in a box under 360 the
narrow path is a scene 360 wide scaled to the box, as the request's
picture is. How work runs draws its path with Ask temper's drawing,
legend, tabs, card and act, shared in the view's `pages/ways.rs`, and
Roles, a disclosure a role of the view `roles`, read as the page opens
and again at a newer revision. The view's `tests/ask.rs`, `tests/how.rs`
and `tests/views.rs`, the fuzzy suite's
`parts_and_drawing_follow_only_the_current_facts` and the gallery's
`ask-ways-*` and `how` to `night-how` hold them. Their render check and
the page's walk remain `../README.md`, section 5's after-landing work.

The document and `../tui/box.md` have no known implementation gap in the TUI in
front of a box, by the look request `tui-box-front` (its account
`0030c4f597`): the forges document's `linked`, set at jig's
`LinkSignedIn` and false after a restart, and `welcome::signed_in`; the
handed link on descriptor 3 and the box's lines on descriptor 4,
`BoxLine` read and printed back; `Start.front` and `Report::Front`; the
welcome before and after the sign-in, its steps read-only at
`welcome::decision`'s step, with the forge under way; the playground's
header, banner and quiet line; the quit question by `work_runs`, a give
up's note and `/log`, offered in front alone; and the place "box" or
"box:PORT" by `place(address, true)`. Goldens at 80 by 24, the state,
view, theme, host, run and domain rows, and the joint world's TUI stories
(`temper/tests/client/tests/suite/tui.rs`) hold them; the theme's rows
map the new tokens in both schemes, which establishes no rendered look.
The quiet line names a step as the box names it ("forgejo-3
restarting"), where the request's frame 05 has "forge restarting", and
`/log` shows the box's own lines, as `../tui/box.md` has them
(`../../draft/tui.status.md`). The journey `box-welcome` waits for
`again` (`../tui/journeys.status.md`, 8.3), and the real terminal's check,
Linux and macOS, is `../README.md`, section 5's after-landing work, owned by temper/tui.

- **Design departs:** *The program's environment*, from the request `box-small-2`
  of 2026-11-06, item 1, decided by its plan's cut, which takes `resolve` as all or
  nothing: with no `HOME` and an XDG variable placing some of the directories, not all,
  `program_environment` (`temper-engine-shell/src/serve.rs`) knows no default and names
  all three, one at its own default among them, where the request has each only where it
  differs. The command finds the same directories either way, and no test holds this
  case: `no_home_names_all_three` gives no XDG variable. Seen by reading.

- **Design departs:** *The words of a command for this machine*, from the request
  `box-small` of 2026-11-06, item 1, which has these words in any other command handed
  to the person for this machine: `temper-box forge run`'s refusal
  (`temper-engine-shell/src/forge.rs`, `run_refusal`) says `temper-box forge prepare` and each
  directory whole, and the terminal client says `temper-box open`
  (`temper-client-shell-terminal/src/terminal.rs`), so with another `temper-box` first on
  `PATH` each names that one. That plan's cut names the pages' commands alone, and no
  reason is recorded. Seen by reading.
