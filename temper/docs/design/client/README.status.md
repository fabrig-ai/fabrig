# README: status

2026-11-05. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 10: a browser walk from a draft's sheet through planning, a live
  report and delegate tasks, Pause, Resume, a message and Stop. The fake box's
  model (`temper-fake/src/backends.rs`, `model_document`) writes a report only
  for the first chat's goal, task 2, then finishes it without delegates. The
  browser can open that ended goal and its report, but cannot exercise those
  live states with this script.
- **Departs:** 3.5, 5.14: Goals' "Start one from a draft" goes to Chats with
  the Drafts filter, rather than opening a sheet with a Draft field;
  `temper-client-domain`'s `pages/start_goal.rs` ignores `OpenStart` without
  a supplied draft. The sheet with a supplied draft remains. This is the
  route accepted by `ui-goal-page`'s `goal-browser` answer of 2026-11-05;
  `ui-chat-draft` owns its draft-to-goal continuation. On the fake box's
  stock project, Chats offers no Drafts filter or draft-start control.
- **Not yet:** 3.5, *Away*: its client domain and view. 3.5, *Home on any project*: the stock owner pool may
  reserve the whole project period, leaving no funding for the status
  agent (`temper-service`'s `stock.rs`, `project_policy`;
  `jig-domain-core`'s `connector_repair_authority`). Returning then keeps
  the computed sentence. The fake-box proof reduces the owner's pool
  allowance to leave period funding, as the root's status world does.
  3.4, 5.5: the composer's
  supplied default, jig's
  `resolved_default`, which neither jig's `Me` nor temper carries. 3.5:
  `Board { project }` with its reordering: no
  address in `temper-client-domain`'s `address.rs`.
  5.3, 5.13: the Away switch in the rail and phone's top bar, and
  "While you're away, agents may decide", await Away's `SetAway`.
  5.13: a person with a project's role alone can read kinds 6 and 7,
  but cannot change them on the tab: jig's settings inventory requires
  a deployment role (`jig-domain-core`'s `documents/settings.rs`, `make`),
  so `temper-client-domain`'s `forge/ways.rs`, `act`, has no newest
  revision and sends nothing. The controls appear enabled and give no
  refusal. A project-scoped settings inventory needs a contract in jig
  before the tab can use it.
  5.13: `pages/chats.rs`'s `drafts_offered` reads kind 7, but the chat's
  draft offer has no view calling it yet. A goal's start preselecting the
  stored way awaits a goal-start page; `pages/goals.rs` draws the list alone.
  3.5, 5.13, 10: `Proposal { proposal }` and the fuller page's sections, with a review's
  "Change something"; ctl supplies the review groups and running-work effect (`tools.md`, 5.2).
  5.3: the banner's "Changes", on a page that is no project's:
  the request of 2026-11-05 asked for the rail, and its mock has no banner.
  3.5, *Goals*: a project whose `me` lists no goal template opens no watch
  (`temper-client-domain`'s `pages/goals.rs`, `query`), so its page draws the
  kicker and the lede alone, with no heading and no filters, where the section
  says "No goals yet."; a project with no tracked template has a lede that
  stops at "landed work.", which the section has no words for.
  3.5, 4.6, 8: Forget, on the Forges page and as the FORGE
  request `Forget`: no `forget` in `temper-client-protocol-wire`'s schema
  or in `temper-service`'s routing. 3.5: Ready's model line names no
  email for a ChatGPT sign-in, whose account's display is empty, since
  the engine keeps no email: it waits on
  `jig/docs/design/draft/account-facts.md`, 6, its second step.
  3.5, *Ready*: two endings leave nothing under "Model none yet"
  (`temper-client-view`'s `pages/connect_model.rs`, `welcome_end`), and no
  test holds either: a sign-in `Made` or `Renewed`, not declined, with no
  account usable, since 5.5 has no words for it and no walk has ended so
  (section 11); and any ending where `me` offers no ChatGPT plan's sign-in,
  since the function finds that way before it draws the ending's words.
  4.4: the picker, which the adoption page does not draw
  (`temper-client-view`'s `forge/page.rs`), so Home's GitHub step's
  "Adopt" opens the form with no picker, and its line counts no
  repositories, where the mock's does.
  5.5: a plan's first use, its card in the conversation. 3.5, 3.6, 10:
  Delete on a project's settings, confirmed by its name, and jig's
  `RemoveProject` by `shapes::remove_project`, with the chat
  template's grace: no such act in `temper-client-domain`. 3.5, The welcome:
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
  3.5, With no project
  yet, where the document shows the welcome: its heading, words and acts; `pages/chats.rs` draws its page
  alone. There too, with a welcome or without one, `NewChat` at its address
  outside the playground's start: `temper-client-view`'s `pages/new_chat.rs`
  draws its layout where the section has it give way, its line calling the
  missing project "Project".
  4.7: the "not installed" outcome, "temper's App is not installed on
  OWNER.", with "Install on another account": `temper-service/src/family.rs`
  maps `forge::AdoptionRefusal::NotInstalled` to `ForgeReason::NoAccess`;
  the client wire has no not-installed reason, pinned as a known gap by
  `temper-service/src/tests/setup_github.rs`,
  `unaccepted_and_other_account_adoptions_keep_the_wire_reason_gap`;
  no reason is recorded.
- **Departs:** 3.5, Settings: "Configure temper" chooses among the rows
  `temper-client-domain`'s `Chats::open_configuring` is delivered, whose
  live watch holds a window of `limits.client.domain.rows` by latest
  activity, so a configuring chat not closed but past that window isn't
  seen and the link opens `Configure { project }`; `configure-temper-name`'s
  landing accepted it, raising no bound for one link.
  3.5, With no project yet, and 5.3: where the document
  shows the welcome, with no project, the banner's place for it reads "No project yet" followed by the new
  project's form, "New project" and "Create" (`temper-client-view`'s
  `frame.rs`, `switcher`), where 5.3 ends the switcher's list with that
  form and 3.5 leaves the first project to the playground's start or
  the welcome's Ready. The code records no reason; it lets a person make
  the project Ready would offer before Ready, the case of the draft's open question
  (`../draft/first-run-projects.status.md`).
  3.5, 4.6, Changes: the queue's first says "checking" at whichever step of
  its turn, and "checks started" gives when that step began
  (`temper-engine-domain-forge`'s `views/changes.rs`, `Change.since`), where
  the request of 2026-11-05 has when its checks began: nothing keeps that time.
  4.6: `changes_view` reads every `ChangeRow`'s repository with `?`, and no
  step erases a closed change's row (`../domain/after-landing.status.md`), so
  once a forge's removal erases a repository with a closed change, no project
  has the document; seen by reading, no test holds it and no reason is recorded.
  3.5, *Goals*: `goals::row` is `pub(crate)` (`temper-client-view`'s
  `pages/goals.rs`), where this section has it public, because its `Bindings`
  is the view's own and Home, its next reader, is in the same crate.
  3.5, *Needs you*: a filter's `kind` and `goal` narrow the open cards alone:
  `temper-client-view`'s `pages/needs.rs`, `main`, asks `Session::answered` by
  project, and an `Answered` keeps neither, so under such a filter the answered
  lines and the head's count hold answers the filter leaves out; no reason is
  recorded.
  3.5, *A new chat* and *A project with no repository*: `NewChat`'s line says
  the project has no repository for every `Repository` but `In`
  (`temper-client-view`'s `pages/new_chat.rs`, `main`), so also with the
  `forges` watch refused, ended or not delivered yet, where the section has
  nothing said. The request of 2026-11-05 gives that line two cases and no
  words for not knowing; no test holds it and no reason is recorded.
  3.5, *The rows*: an account's sources are a list under their level-3
  heading, each with its state and acts, and a sign-in under way a paragraph
  (`temper-client-view`'s `pages/accounts/rows.rs`, `draw`), where the section
  has a row's facts as `.setting` rows, because the rows keep their sources,
  observations and forms in document order.
- **Design departs:** 3.5, *A chat's conversation*, its cards, from the request
  `ui-chats` of 2026-11-05, which names a proposal, a held task and a question's
  answer as cards in the conversation: a chat's page holds proposals alone
  (`temper-client-domain`'s `pages/chat.rs`, `cards`), since its held delegate
  is its one line (5.7), and the answer the one line under "You" the same
  section keeps. No reason is recorded.
- **Design departs:** 5.9, and `browser.md`, 2.5, from the request `ui-chats` of
  2026-11-05, decided by its plan's cut: the request retires `look::CHAT` and
  `CHATBAR`, and they stand renamed `AGENT_PAGE` and `TASKBAR`, with `ENTRY` and
  `COMPOSER_AREA` kept, because an agent's task's page still draws them and the
  request keeps to chats' pages.
- **Design departs:** 3.5, *Needs you*, *What the client keeps*, and 5.3, from
  the request of 2026-11-05 for them, decided by its plan's cut, the request
  leaving that judgement to it. The rail's watch opens while a project's frame shows, not once signed
  in: the rail alone reads it, and opened at sign-in it would stale all three
  journeys' recordings. Bare `/inbox` is `/`, the Inbox's unfiltered address
  (`browser.md`, 2.3). The dots are a figure, not `role=img`: skein-ui's
  `Role::Image` is an image, with no children. Money reads "S of L used", not "$S
  of $L": the contract's units aren't dollars, and the request also asks the task
  page's form, which has none. Leave held takes its reason in a disclosure, as
  Reject does, where the request names no text area: `Decision::LeftHeld` needs
  one. A `Choice:` line past `CUE_BYTES` is no choice (`../domain/agent.md`, 8.2),
  where the request counts every such line, since its button could send nothing.
  And " · later" shows on *Needs you*'s mono line as on Home's, where the request
  and the mock have it on Home's card alone: the one function draws both, and no
  reason is recorded.
- **Design departs:** 3.5, *The flows' paths* and *Builders*, from the request
  `ui-settings-tabs` of 2026-11-05, decided by its plan's cut. The terminal
  draws no path, where the request has each flow add one: the same request
  keeps the terminal's prints' meaning, a path would shift every number on the
  flows, and the terminal's rail holds "Settings". With hosts kept and none
  connected, `.empty` stands above their "Not connected" lines, where the
  request gives none connected `.empty` and no words for that case, so that no
  state is dropped.
- **Design departs:** 3.5, *Goals*, from the request of 2026-11-05: "6 / 9" is
  on the task line (jig's `client/domain/contract.md`, 5.1), where the request
  offered the goal's document or the tree's read, and a row reads its report
  alone, not also its tasks by `parent`: each place needed the same stored
  counter, and the line needs no read per row. The lede's ways run in template
  number order, as the request's words have it, so the reference's read "task
  by task, plan, tasks, land whole", where its mock, the target, draws them the
  other way round: number order is what a project's settings decide.
