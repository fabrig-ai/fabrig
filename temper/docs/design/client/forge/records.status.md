# records: status

2026-11-11. Where the tree's code lags or departs from [records.md](records.md).

- **Not yet:** the whole document: the walk the request `first-usable-model` of
  2026-11-06 asks: none on record has a real box with one account, unedited, start
  a chat that answers, or a box with no usable account say why it waits. A real
  `temper-box program-login` needs a person's sign-in, so such a walk starts from a login
  already signed in, or a key, and pastes the printed command only as far as the
  program's prompt. No test runs that command through `temper-box program-login`:
  `temper-box-shell`'s `program_login/tests.rs` runs it by `sh -c` against a stub, so that it
  reaches the sign-in for that box's `programs/NAME` is seen by reading.
- **Not yet:** 
  `Board { project }` with its reordering: no
  address in `temper-client-domain`'s `address.rs`.
   a person with a project's role alone can read kinds 6 and 7,
  but cannot change them on the tab: jig's settings inventory requires
  a deployment role (`jig-domain-core`'s `documents/settings.rs`, `make`),
  so `temper-client-domain`'s `forge/ways.rs`, `act`, has no newest
  revision and sends nothing. The controls appear enabled and give no
  refusal, where the request of 2026-11-05 has a person who is no owner refused
  in jig's words, as one with a deployment role is. With no document to say a
  setting changed, that person reads them again only as the project opens or
  the link comes back. A project-scoped settings inventory needs a contract in
  jig before the tab can use it.
   Home draws neither "Show goals" nor "See the settings", so
  nothing opens the tab by them.
  `../pages/README.md`, *Goals*: a project whose `me` lists no goal template opens no watch
  (`temper-client-domain`'s `pages/goals.rs`, `query`), so its page draws the
  kicker and the lede alone, with no heading and no filters, where the section
  says "No goals yet."; a project with no tracked template has a lede that
  stops at "landed work.", which the section has no words for.
  the whole document: Forget, on the Forges page and as the FORGE
  request `Forget`: no `forget` in `temper-client-protocol-wire`'s schema
  or in `temper-service`'s routing.
  `../pages/README.md`, *Ready*: two endings leave nothing under "Model none yet"
  (`temper-client-view`'s `pages/connect_model.rs`, `welcome_end`), and no
  test holds either: a sign-in `Made` or `Renewed`, not declined, with no
  account usable, since `../pages/accounts.md`, 2 has no words for it and no walk has ended so
  (`../README.md`, section 5); and any ending where `me` offers no ChatGPT plan's sign-in,
  since the function finds that way before it draws the ending's words.
   a plan's first use, its card in the conversation. 
  Delete on a project's settings, confirmed by its name, and jig's
  `RemoveProject` by `shapes::remove_project`, with the chat
  template's grace: no such act in `temper-client-domain`. `../pages/README.md`, The welcome:
  for a person whose role does not hold the deployment's forges, the model
  alone deciding;
  `temper-client-domain`'s `pages/welcome.rs` `decision` reads a
  refused `forges` watch as no forge, so a usable model opens Forge.
  FORGE in "FORGE · not
  finished" as the forge's name; `forge/connect.rs` `unfinished` prints
  "FORGE" itself. "Go on" after a restart: the row's `app` is the forge
  domain's memory alone (`temper-engine-domain-forge`'s `domain.rs`
  `apps`), so a GitHub forge found unfinished after a restart offers
  "Start over" alone.
   the "not installed" outcome, "temper's App is not installed on
  OWNER.", with "Install on another account": `temper-service/src/family.rs`
  maps `forge::AdoptionRefusal::NotInstalled` to `ForgeReason::NoAccess`;
  the client wire has no not-installed reason, so ctl offers repository access
  and Forgejo collaborator-admin recovery rather than installation recovery.
  This gap is pinned by
  `temper-service/src/tests/setup_github.rs`,
  `unaccepted_and_other_account_adoptions_keep_the_wire_reason_gap`;
  no reason is recorded.
- **Departs:** `../pages/README.md`, *The first chat*, and `../view.md`, 3: the rail's "Ask temper", the
  tab bar's "Ask" and To begin's "Start" choose among the rows
  `temper-client-domain`'s `Chats::open_asking` is delivered, whose
  live watch holds a window of `limits.client.domain.rows` by latest
  activity, so an Ask temper chat not closed but past that window isn't
  seen and they open `Ask { project }`; the landings of `configure-temper-name`
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
