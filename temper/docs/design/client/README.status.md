# README: status

2026-11-03. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 3.4, 5.5: the composer's supplied default, jig's
  `resolved_default`, which neither jig's `Me` nor temper carries. 3.5:
  `Board { project }` with its reordering, and `Changes { project }`: no
  address in `temper-client-domain`'s `address.rs`. 3.5, 5.13, 10:
  `HowWorkRuns`, `Proposal { proposal }` and a proposal's card in a chat,
  with their tests. 3.5, 4.6, 8: Forget, on the Forges page and as the FORGE
  request `Forget`: no `forget` in `temper-client-protocol-wire`'s schema
  or in `temper-service`'s routing.
  3.5, The welcome: whole once a real forge's first repository is
  adopted, not once the person has a project
  (`temper-client-domain`'s `pages/welcome.rs`, `decision` and
  `first_adopted`); the Forge step's project field and the client's
  adoption of the first repository (`temper-client-view`'s
  `forge/connect.rs`, `setup_fields` and `github_picker`;
  `temper-client-domain`'s `forge/forges.rs`, `maybe_adopt`); Ready's
  "Open Default", where `temper-client-view`'s `pages/welcome.rs`
  `ready` waits for that adoption.
  3.5, A project with no repository: the chats page says nothing of it
  (`temper-client-view`'s `pages/chats.rs`). 4.4: the form opened
  filled for a set up's first repository adopted nowhere
  (`temper-client-domain`'s `forge/form.rs`, `form`, starts at forge 1
  with owner, name and landing empty).
  4.4: the picker, which the adoption page does not draw
  (`temper-client-view`'s `forge/page.rs`).
  4.6, 8: `SetUp`, `FirstAsk` and the row's `first` carry a project,
  `SetUpProject`, and `SetUpRefusal` keeps `new_project`
  (`temper-client-protocol-wire/schema/client-wire-v1.schema`).
  3.5, The first chat: the offer "Tell temper how you work" on a chats
  page with no chat; no such act in `temper-client-domain`'s `pages.rs`
  and no offer in `temper-client-view`'s `pages/chats.rs`.
  5.3, Projects in the banner: the switcher's Rename and new project:
  no such acts in `temper-client-domain`'s `pages.rs`, and the banner
  draws one button per project (`temper-client-view`'s `frame.rs`,
  `project_buttons`).
  5.5: a plan's first use, its card in the conversation. 3.5, 3.6, 10:
  Delete on a project's settings, confirmed by its name, and jig's
  `RemoveProject` by `shapes::remove_project`, with the chat
  template's grace: no project's settings and no such act in
  `temper-client-domain`. 3.5, The welcome: for a person whose role
  does not hold the deployment's forges, the model alone deciding;
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
  as ready, where it shows `ConnectModel`'s Choose. 3.5, With no project
  yet: its heading, words and acts; `pages/chats.rs` draws the list
  alone. 5.6: the playground's project named "Playground" by the
  phrasebook, not by the name its record holds (`temper-client-view`'s
  `pages.rs`, `project_name`).
