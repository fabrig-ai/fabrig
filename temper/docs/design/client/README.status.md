# README: status

2026-11-05. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 3.4, 5.5: the composer's
  supplied default, jig's
  `resolved_default`, which neither jig's `Me` nor temper carries. 3.5:
  `Board { project }` with its reordering, and `Changes { project }`: no
  address in `temper-client-domain`'s `address.rs`. 3.5, 5.13, 10:
  `Proposal { proposal }` and the fuller page's sections, with a review's
  "Change something"; ctl supplies the review groups and running-work effect
  (`tools.md`, 5.2). 3.5, 5.3, 5.13: How work runs' controls and its
  session's watch and reads, the rail and tab bar by kind 7, `drafts_offered`,
  and a proposal of settings' card in a chat.
  3.5, 4.6, 8: Forget, on the Forges page and as the FORGE
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
- **Departs:** 3.5, With no project yet, and 5.3: where the document
  shows the welcome, with no project, the banner's place for it reads "No project yet" followed by the new
  project's form, "New project" and "Create" (`temper-client-view`'s
  `frame.rs`, `switcher`), where 5.3 ends the switcher's list with that
  form and 3.5 leaves the first project to the playground's start or
  the welcome's Ready. The code records no reason; it lets a person make
  the project Ready would offer before Ready, the case of the draft's open question
  (`../draft/first-run-projects.status.md`).
