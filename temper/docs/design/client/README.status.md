# README: status

2026-11-05. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 3.5, The Forge step with a forge: `forge/connect.rs`'s `welcome` says one line and "View". 3.5, *A model usable*, *A way's page*'s Key and Paste, *The sign-in under way*, *Finishing a paste*: the welcome draws `ConnectModel`'s cards, "‹ All ways", "LLM account sign-in started" and "Pending". 3.5, *A way's page*'s Claude: the welcome declares a name and stops at "Declared". 3.5, *Ready*: Model comes first until both are done; the line's size, its spinning ring, "Forge none yet"'s link at the row's end, Model's words, the note under the form and the foot. 3.5, *Forgejo's sheet*, *Its copies*: copies and nothing say "Not available yet"; `SetUp` carries an org and a name, in the store, on the wire and in the engine. 3.5, Home's second step: its copies, "Add a repository" and its sheet. 3.4, 5.5: the composer's
  supplied default, jig's
  `resolved_default`, which neither jig's `Me` nor temper carries. 3.5:
  `Board { project }` with its reordering, and `Changes { project }`: no
  address in `temper-client-domain`'s `address.rs`. 3.5, 5.13, 10:
  `HowWorkRuns`, `Proposal { proposal }` and a proposal's card in a chat,
  with their tests; ctl supplies the review groups and running-work effect
  (`tools.md`, 5.2). 3.5, 4.6, 8: Forget, on the Forges page and as the FORGE
  request `Forget`: no `forget` in `temper-client-protocol-wire`'s schema
  or in `temper-service`'s routing. 3.5: Ready's model line names no
  email for a ChatGPT sign-in, whose account's display is empty, since
  the engine keeps no email: it waits on
  `jig/docs/design/draft/account-facts.md`, 6, its second step.
  3.5, Settings: the forge and account lines await Settings' design;
  "Open Forgejo ↗" is reached through Forges, one step beyond Settings,
  and the model line has no other place yet.
  3.5, Home: "landed" on a chat's line waits on the client's Home
  chat-row reader: the list gives the chat's state, not what its
  delegates landed. It needs one read per row; Home uses the supplied
  chat state, including "closed", until that reader is built.
  4.4: Home's GitHub step's repository picker and reached count, and
  the picker, which the adoption page does not draw
  (`temper-client-view`'s `forge/page.rs`).
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
