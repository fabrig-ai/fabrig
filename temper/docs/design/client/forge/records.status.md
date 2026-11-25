# records: status

2026-11-21. Where the tree's code lags or departs from [records.md](records.md).




- **Not yet:** *`OpenChange`* and *A project's changes*: `into` left
  empty, `landing` and the `default_branch` flag mean the forge's
  default branch rather than the repository's landing branch [in flight:
  change 8087].


- **Not yet:** *A project's ways, the subject `ways`*: `WaysPanel`'s
  `rules`, a `PathRules` of `PathRule` rows, on the wire [in flight:
  change 871].
- **Not yet:** *`SetDraft`* and *A project's landed drafts, the subject
  `drafts`*: no such request, outcome, subject or panel on the wire
  [in flight: change 662].
- **Not yet:** 
  the Board address and reordering remains `../engine.status.md` (Not yet).
   the project-only settings-role gap remains `../pages/settings.status.md`, 2.
   the Home settings-tab links remain `../pages/settings.status.md`, 2.
  `../pages/README.md`, *Goals*: the empty Goals-page gap remains `../pages/work.status.md`, 1.
  the whole document: Forget, on the Forges page and as the FORGE
  request `Forget`: no `forget` in `temper-client-protocol-wire`'s schema
  or in `temper-service`'s routing.
  `../pages/README.md`, *Ready*: the unhandled Ready endings remain `../pages/welcome.status.md`, 1.
   the plan's first-use card remains `../pages/accounts-words.status.md`, 1.
  `../pages/README.md`, The welcome:
  the welcome role-scope gap remains `../pages/welcome.status.md`, 1.
  FORGE in "FORGE · not
  finished" as the forge's name; `forge/connect.rs` `unfinished` prints
  "FORGE" itself. "Go on" after a restart: the row's `app` is the forge
  domain's memory alone (`temper-engine-domain-forge`'s `domain.rs`
  `apps`), so a GitHub forge found unfinished after a restart offers
  "Start over" alone.
   the App-not-installed reason remains `family.status.md`, 6.
- **Departs:** `../pages/README.md`, *The first chat*, and `../view-navigation.md`, 1: the rail's "Ask temper", the
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

- **Not yet:** the subject `approvals`: `ApprovalRow`'s `size` and `ApprovalSize` on the wire [in flight: change 533].
- **Not yet:** the subject `approvals`: `size` filled from the change's approval evidence [in flight: change 534].
- **Not yet:** the whole document: the first-usable-model real-box walk remains `../pages/accounts-words.status.md`, 1.
