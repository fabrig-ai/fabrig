# README: status

2026-11-05. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 3.5, 3.6, 5.8, 5.11, 5.14, 10: a goal's page, its acts and Start goal as these sections say them: plan `ui-goal-page`; a tracked task still draws the old page (`temper-client-view`'s `pages/task.rs`).
- **Not yet:** 3.4, 5.5: the composer's
  supplied default, jig's
  `resolved_default`, which neither jig's `Me` nor temper carries.
  5.3: the banner's "Changes", on a page that is no project's:
  the request of 2026-11-05 asked for the rail, and its mock has no banner.
  3.5, *Goals*: a project whose `me` lists no goal template opens no watch
  (`temper-client-domain`'s `pages/goals.rs`, `query`), so its page draws the
  kicker and the lede alone, with no heading and no filters, where the section
  says "No goals yet."; a project with no tracked template has a lede that
  stops at "landed work.", which the section has no words for. 3.5, 5.13, 10:
  `HowWorkRuns`, `Proposal { proposal }` and a proposal's card in a chat,
  with their tests, and with them How work runs' "Configure temper" entry and
  a review's "Change something"; ctl supplies the review groups and running-work effect
  (`tools.md`, 5.2). 3.5, 4.6, 8: Forget, on the Forges page and as the FORGE
  request `Forget`: no `forget` in `temper-client-protocol-wire`'s schema
  or in `temper-service`'s routing. 3.5: Ready's model line names no
  email for a ChatGPT sign-in, whose account's display is empty, since
  the engine keeps no email: it waits on
  `jig/docs/design/draft/account-facts.md`, 6, its second step.
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
  The playground offered on Forge's and Model's foot while a real
  account alone is kept; `temper-client-view`'s `pages/welcome.rs`
  `main` offers it only while the store is empty. FORGE in "FORGE · not
  finished" as the forge's name; `forge/connect.rs` `unfinished` prints
  "FORGE" itself. "Go on" after a restart: the row's `app` is the forge
  domain's memory alone (`temper-engine-domain-forge`'s `domain.rs`
  `apps`), so a GitHub forge found unfinished after a restart offers
  "Start over" alone. The Model step showing an account added before
  as ready, where it shows `ConnectModel`'s Choose.
  3.5, With no project
  yet, where the document shows the welcome: its heading, words and acts; `pages/chats.rs` draws the pane
  alone, the new chat's bar with no status and the list under "Chats".
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
- **Design departs:** 3.5, *Goals*, from the request of 2026-11-05: "6 / 9" is
  on the task line (jig's `client/domain/contract.md`, 5.1), where the request
  offered the goal's document or the tree's read, and a row reads its report
  alone, not also its tasks by `parent`: each place needed the same stored
  counter, and the line needs no read per row. The lede's ways run in template
  number order, as the request's words have it, so the reference's read "task
  by task, plan, tasks, land whole", where its mock, the target, draws them the
  other way round: number order is what a project's settings decide.
